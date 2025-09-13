use crate::DeserializationError;

/// Gives encoded size in bytes
///
/// # Arguments
/// * `nr` - number to encode
pub fn encoded_size(nr: u128) -> usize {
    let mut res = 0;
    let mut nr = nr;
    while nr > 0 {
        nr /= 128;
        res += 1;
    }
    res
}

/// Encodes a number into a vector of bytes.
///
/// # Arguments
/// * `nr` - number to encode
pub fn encode(nr: u128) -> Vec<u8> {
    let mut res = Vec::new();
    let mut nr = nr;
    while nr > 0 {
        let mut encoded = nr % 128;
        nr /= 128;
        if nr > 0 {
            encoded |= 128;
        }
        res.push(encoded as u8);
    }
    res
}

/// Decodes a number from a slice of bytes.
///
/// # Arguments
/// * `data` - slice of bytes to decode
pub fn decode(data: &[u8]) -> u128 {
    let mut num = 0;
    let mut multiplier = 1;
    for byte in data {
        num += (*byte as u128 & 127) * multiplier;
        multiplier *= 128;
    }
    num
}

/// Decodes a number from a slice of bytes when size of encoded number is unknown, returning the number and the number of bytes read.
///
/// # Arguments
/// * `data` - slice of bytes to decode number from
///
/// # Returns
/// * (number, bytes read)
pub fn read_from_slice(data: &[u8]) -> Result<(u128, usize), DeserializationError> {
    let mut idx = 0;
    loop {
        if idx > data.len() - 1 {
            break Err(DeserializationError::NotEnoughBytes(1));
        }

        if (data[idx] & 1 << 7) == 0 {
            break Ok((decode(&data[..=idx]), idx + 1));
        }

        idx += 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_encode_decode_number() {
        let number = 1234567890;
        let encoded = encode(number);
        let decoded = decode(&encoded);
        assert_eq!(number, decoded);
        assert_eq!(5, encoded.len()); // 1234567890 ~ 2^31, 7 bits per byte = 7 * 5 = 35
    }

    #[test]
    fn can_decode_number() {
        let nr = &[216u8, 4];
        let res = decode(nr);
        assert_eq!(600, res);
    }

    #[test]
    fn can_decode_number_from_larger_slice() {
        let nr = &[216u8, 4, 234, 19, 74];
        let res = read_from_slice(nr).unwrap();
        assert_eq!((600, 2), res);
    }

    #[test]
    fn can_decode_number_in_4_bytes() {
        let max_nr = 268435455; // max number in 4 bytes
        let encoded = encode(max_nr);
        assert_eq!(4, encoded.len());
    }

    #[test]
    fn cant_decode_bignr_in_4_bytes() {
        let max_nr = 268435456;
        let encoded = encode(max_nr);
        assert_ne!(4, encoded.len());
    }

    #[test]
    fn cant_decode_slice_that_lies() {
        let slice = &[0b10111110]; // slice notes there is a second byte (7th bit, right-to-left), but there's not
        let decoded = read_from_slice(slice);
        assert!(decoded.is_err());
    }

    #[test]
    fn can_encode_nr_lt_128_in_1_byte() {
        let encoded = encode(127);
        assert_eq!(1, encoded.len());
    }

    #[test]
    fn can_guess_encoded_size() {
        let one_byte = 127;
        assert_eq!(1, encoded_size(one_byte));

        let two_bytes = 128;
        assert_eq!(2, encoded_size(two_bytes));

        let four_bytes = 268435455;
        assert_eq!(4, encoded_size(four_bytes));
    }
}
