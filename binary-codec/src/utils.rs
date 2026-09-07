use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6};

use crate::{
    BinaryDeserializer, BinarySerializer, BitStreamReader, BitStreamWriter, DeserializationError,
    SerializationError, SerializerConfig,
};

pub fn get_read_size<'a, T: Clone, E: From<DeserializationError>>(
    stream: &mut BitStreamReader,
    size_key: Option<&str>,
    config: &mut SerializerConfig<T>,
) -> Result<usize, E> {
    let size = if let Some(size_key) = size_key {
        if size_key == "__dynamic" {
            return stream
                .read_dyn_int()
                .map(|v| v as usize)
                .map_err(Into::into);
        }

        config.get_length(size_key).unwrap_or(stream.bytes_left())
    } else {
        stream.bytes_left()
    };

    Ok(size)
}

pub fn write_size<T: Clone, E: From<SerializationError>>(
    size: usize,
    size_key: Option<&str>,
    stream: &mut BitStreamWriter,
    config: &mut SerializerConfig<T>,
) -> Result<(), E> {
    if let Some(size_key) = size_key {
        if size_key == "__dynamic" {
            stream.write_dyn_int(size as u128);
            return Ok(());
        }

        if let Some(expected) = config.get_length(size_key) {
            if expected != size {
                return Err(
                    SerializationError::UnexpectedLength(expected as u64, size as u64).into(),
                );
            }
        }
    }

    Ok(())
}

pub fn read_string<T: Clone, E: From<DeserializationError>>(
    stream: &mut BitStreamReader,
    size_key: Option<&str>,
    config: &mut SerializerConfig<T>,
) -> Result<String, E> {
    let len = get_read_size(stream, size_key, config)?;
    let slice = stream.read_bytes(len).map_err(Into::into)?;
    let string = String::from_utf8(slice.to_vec()).expect("Not valid UTF-8 bytes to create string");

    Ok(string)
}

pub fn write_string<T: Clone, E: From<SerializationError>>(
    value: &str,
    size_key: Option<&str>,
    stream: &mut BitStreamWriter,
    config: &mut SerializerConfig<T>,
) -> Result<(), E> {
    write_size(value.len(), size_key, stream, config)?;
    stream.write_bytes(&value.as_bytes());

    Ok(())
}

pub fn read_object<T, U, E>(
    stream: &mut BitStreamReader,
    size_key: Option<&str>,
    config: &mut SerializerConfig<U>,
) -> Result<T, E>
where
    T: BinaryDeserializer<U, E>,
    U: Clone,
    E: From<DeserializationError>,
{
    let len = get_read_size::<U, E>(stream, size_key, config)?;

    // If exact size of buffer is available, don't slice
    if stream.bytes_left() == len {
        T::read_bytes(stream, Some(config))
    } else {
        // Create an isolated slice, because it could be that the object uses a dynamically sized buffer based on bytes left
        let slice = stream.read_bytes(len).map_err(Into::into)?;
        let mut isolated_reader = BitStreamReader::new(slice);

        T::read_bytes(&mut isolated_reader, Some(config))
    }
}

pub fn write_object<T, U, E>(
    value: &T,
    size_key: Option<&str>,
    stream: &mut BitStreamWriter,
    config: &mut SerializerConfig<U>,
) -> Result<(), E>
where
    T: BinarySerializer<U, E>,
    U: Clone,
    E: From<SerializationError>,
{
    // If length name is provided, we need to ensure the length matches
    // So we write it to a different buffer
    if size_key.is_some() {
        let mut buffer = Vec::new();
        let mut temp_stream = BitStreamWriter::new(&mut buffer);
        value.write_bytes(&mut temp_stream, Some(config))?;
        write_size::<U, E>(buffer.len(), size_key, stream, config)?;
        stream.write_bytes(&buffer);
        Ok(())
    } else {
        value.write_bytes(stream, Some(config))
    }
}

pub fn read_socketaddr_v4(
    stream: &mut BitStreamReader,
) -> Result<std::net::SocketAddrV4, DeserializationError> {
    let octets: [u8; 4] = stream.read_bytes(4)?.try_into().unwrap();
    let port = stream.read_fixed_int()?;
    let ip = Ipv4Addr::from_octets(octets);
    Ok(SocketAddrV4::new(ip, port))
}

pub fn write_socketaddr_v4(value: &std::net::SocketAddrV4, stream: &mut BitStreamWriter) {
    stream.write_bytes(&value.ip().octets());
    stream.write_fixed_int(value.port());
}

pub fn read_socketaddr_v6(
    stream: &mut BitStreamReader,
) -> Result<std::net::SocketAddrV6, DeserializationError> {
    let octets: [u8; 16] = stream.read_bytes(16)?.try_into().unwrap();
    let port = stream.read_fixed_int()?;
    let ip = Ipv6Addr::from_octets(octets);
    Ok(SocketAddrV6::new(ip, port, 0, 0))
}

pub fn write_socketaddr_v6(value: &std::net::SocketAddrV6, stream: &mut BitStreamWriter) {
    stream.write_bytes(&value.ip().octets());
    stream.write_fixed_int(value.port());
}
