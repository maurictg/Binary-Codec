extern crate proc_macro;

use quote::{format_ident, quote};
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, Attribute, Data, DeriveInput, Fields,
    Lit, PathArguments, Type,
};

#[proc_macro_derive(
    ToBytes,
    attributes(
        bits,
        dyn_int,
        dyn_length,
        key_dyn_length,
        val_dyn_length,
        toggles,
        toggled_by,
        length_for,
        length_by,
        variant_for,
        variant_by,
        no_discriminator
    )
)]
pub fn generate_code_to_bytes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    generate_code_binary_serializer(false, input)
}

#[proc_macro_derive(
    FromBytes,
    attributes(
        bits,
        dyn_int,
        key_dyn_length,
        val_dyn_length,
        dyn_length,
        toggles,
        toggled_by,
        length_for,
        length_by,
        variant_for,
        variant_by,
        no_discriminator
    )
)]
pub fn generate_code_from_bytes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    generate_code_binary_serializer(true, input)
}

fn generate_code_binary_serializer(
    read: bool,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse code input (TokenStream) to AST
    let ast = parse_macro_input!(input as DeriveInput);

    match ast.data {
        Data::Struct(ref data) => generate_struct_serializer(read, &ast, data),
        Data::Enum(ref data) => generate_enum_serializer(read, &ast, data),
        _ => panic!("ToBytes can only be used on structs"),
    }
}

fn generate_struct_serializer(
    read: bool,
    ast: &DeriveInput,
    data_struct: &syn::DataStruct,
) -> proc_macro::TokenStream {
    let fields = &data_struct.fields;
    let struct_name = &ast.ident;

    // Iterate all fields in the struct
    let field_serializations = fields.iter().map(|field| {
        let field_name = field
            .ident
            .as_ref()
            .expect("ToBytes does not support fields without a name");

        let field_type = &field.ty;

        let mut toggle_key = None;
        let mut variant_key = None;
        let mut length_key = None;
        let mut toggled_by = None;
        let mut variant_by = None;
        let mut length_by = None;
        let mut is_dynamic_int = false;
        let mut has_dynamic_length = false;
        let mut bits_count = None;
        let mut key_dyn_length = false;
        let mut val_dyn_length = false;

        // Search attributes for length/toggle declarations
        for attr in field.attrs.iter() {
            let ident = attr.path().get_ident().map(|i| i.clone().to_string());
            match ident.as_deref() {
                Some("dyn_int") => is_dynamic_int = true,
                Some("dyn_length") => has_dynamic_length = true,
                Some("key_dyn_length") => key_dyn_length = true,
                Some("val_dyn_length") => val_dyn_length = true,
                Some("toggles") => toggle_key = get_string_value_from_attribute(attr),
                Some("variant_for") => variant_key = get_string_value_from_attribute(attr),
                Some("length_for") => length_key = get_string_value_from_attribute(attr),
                Some("toggled_by") => toggled_by = get_string_value_from_attribute(attr),
                Some("variant_by") => variant_by = get_string_value_from_attribute(attr),
                Some("length_by") => length_by = get_string_value_from_attribute(attr),
                Some("bits") => bits_count = get_int_value_from_attribute(attr).map(|b| b as u8),
                _ => {}
                // None => continue
            }
        }

        // Runtime toggle_key
        let toggles = if let Some(key) = toggle_key {
            if read {
                quote! {
                    _p_config.set_toggle(#key, _p_val);
                }
            } else {
                quote! {
                    _p_config.set_toggle(#key, *_p_val);
                }
            }
        } else { quote! {} };

        // Runtime length_key
        let length = if let Some(key) = length_key {
            if read {
                quote! {
                    _p_config.set_length(#key, _p_val as usize);
                }
            } else {
                quote! {
                    _p_config.set_length(#key, *_p_val as usize);
                }
            }
        } else { quote! {} };

        // Runtime variant_key
        let variant = if let Some(key) = variant_key {
            if read {
                quote! {
                    _p_config.set_variant(#key, _p_val as u8);
                }
            } else {
                quote! {
                    _p_config.set_variant(#key, *_p_val as u8);
                }
            }
        } else { quote! {} };

        // Compose code to handle field
        let before = if read {
            quote! {}
        } else {
            quote! {
                let _p_val = &self.#field_name;
                #toggles
                #length
                #variant
            }
        };

        let after = if read {
            quote! {
                let #field_name = _p_val;
                #toggles
                #length
                #variant
            }
        } else {
            quote! {}
        };

        let handle_field = generate_code_for_handling_field(
            read,
            field_type,
            field_name,
            bits_count,
            toggled_by,
            variant_by,
            length_by,
            is_dynamic_int,
            has_dynamic_length,
            key_dyn_length,
            val_dyn_length,
            0,
        );

        quote! {
            #before
            #handle_field
            #after
        }
    });

    let error_type = generate_error_type(read);
    let serializer_code = if read {
        let vars = fields.iter().map(|f| f.ident.as_ref().unwrap());

        // read bytes code
        quote! {
            impl binary_codec::BinaryDeserializer for #struct_name {
                fn from_bytes(bytes: &[u8], config: Option<&mut binary_codec::SerializerConfig>) -> Result<Self, #error_type> {
                    let mut _new_config = binary_codec::SerializerConfig::new();
                    let _p_config = config.unwrap_or(&mut _new_config);
                    let _p_bytes = bytes;
                    
                    #(#field_serializations)*

                    Ok(Self {
                        #(#vars),*
                    })
                }
            }
        }
    } else {
        // write bytes code
        quote! {
            impl binary_codec::BinarySerializer for #struct_name {
                fn to_bytes(&self, config: Option<&mut binary_codec::SerializerConfig>) -> Result<Vec<u8>, #error_type> {
                    let mut bytes = Vec::new();
                    Self::write_bytes(self, &mut bytes, config)?;
                    Ok(bytes)
                }

                fn write_bytes(&self, buffer: &mut Vec<u8>, config: Option<&mut binary_codec::SerializerConfig>) -> Result<(), #error_type> {
                    let mut _new_config = binary_codec::SerializerConfig::new();
                    let _p_config = config.unwrap_or(&mut _new_config);
                    let _p_bytes = buffer;

                    #(#field_serializations)*
                    Ok(())
                }
            }
        }
    };

    serializer_code.into()
}

fn generate_enum_serializer(
    read: bool,
    ast: &DeriveInput,
    data_enum: &syn::DataEnum,
) -> proc_macro::TokenStream {
    let enum_name = &ast.ident;
    let error_type = generate_error_type(read);

    let mut no_disc_prefix = false;

    // Search attributes for variant_by declarations
    for attr in ast.attrs.iter() {
        // #[no_disc_prefix] attribute
        if attr.path().is_ident("no_discriminator") {
            no_disc_prefix = true;
        }
    }

    // Assign discriminant values starting from 0
    let variants = data_enum.variants.iter().enumerate().map(|(i, variant)| {
        let var_ident = &variant.ident;
        let disc_value = i as u8; // Could be changed to u16/u32 if needed
        let fields = &variant.fields;

        let write_disc = if no_disc_prefix {
            quote! {}
        } else {
            quote! {
                let _p_disc: u8 = #disc_value;
                binary_codec::fixed_int::FixedInt::write(_p_disc, _p_bytes, _p_config)?;
            }
        };

        match fields {
            Fields::Unit => {
                if read {
                    quote! {
                        #disc_value => {
                            Ok(Self::#var_ident)
                        }
                    }
                } else {
                    quote! {
                        Self::#var_ident => {
                            #write_disc
                        }
                    }
                }
            }
            Fields::Unnamed(fields_unnamed) => {
                let field_count = fields_unnamed.unnamed.len();
                let idents: Vec<_> = (0..field_count).map(|i| format_ident!("f{}", i)).collect();
                let ident_refs: Vec<&syn::Ident> = idents.iter().collect();
                let field_serializations =
                    generate_enum_field_serializations(read, &ident_refs, &fields_unnamed.unnamed);
                if read {
                    quote! {
                        #disc_value => {
                            #(#field_serializations)*
                            Ok(Self::#var_ident(#(#idents),*))
                        }
                    }
                } else {
                    quote! {
                        Self::#var_ident(#(#idents),*) => {
                            #write_disc
                            #(#field_serializations)*
                        }
                    }
                }
            }
            Fields::Named(fields_named) => {
                let field_idents: Vec<_> = fields_named
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();

                let field_serializations =
                    generate_enum_field_serializations(read, &field_idents, &fields_named.named);

                if read {
                    quote! {
                        #disc_value => {
                            #(#field_serializations)*
                            Ok(Self::#var_ident { #(#field_idents),* })
                        }
                    }
                } else {
                    quote! {
                        Self::#var_ident { #(#field_idents),* } => {
                            #write_disc
                            #(#field_serializations)*
                        }
                    }
                }
            }
        }
    });

    if read {
        quote! {
            impl binary_codec::BinaryDeserializer for #enum_name {
                fn from_bytes(bytes: &[u8], config: Option<&mut binary_codec::SerializerConfig>) -> Result<Self, #error_type> {
                    let mut _new_config = binary_codec::SerializerConfig::new();
                    let _p_config = config.unwrap_or(&mut _new_config);
                    let _p_bytes = bytes;

                    let _p_disc = _p_config.discriminator.take().unwrap_or(
                        binary_codec::fixed_int::FixedInt::read(_p_bytes, _p_config)?
                    );

                    match _p_disc {
                        #(#variants,)*
                        _ => Err(#error_type::UnknownDiscriminant(_p_disc)),
                    }
                }
            }
        }
        .into()
    } else {
        quote! {
            impl binary_codec::BinarySerializer for #enum_name {
                fn to_bytes(&self, config: Option<&mut binary_codec::SerializerConfig>) -> Result<Vec<u8>, #error_type> {
                    let mut bytes = Vec::new();
                    Self::write_bytes(self, &mut bytes, config)?;
                    Ok(bytes)
                }

                fn write_bytes(&self, buffer: &mut Vec<u8>, config: Option<&mut binary_codec::SerializerConfig>) -> Result<(), #error_type> {
                    let mut _new_config = binary_codec::SerializerConfig::new();
                    let _p_config = config.unwrap_or(&mut _new_config);
                    let _p_bytes = buffer;

                    match self {
                        #(#variants)*
                    }

                    Ok(())
                }
            }
        }
        .into()
    }
}

fn generate_enum_field_serializations(
    read: bool,
    idents: &Vec<&syn::Ident>,
    fields: &Punctuated<syn::Field, Comma>,
) -> Vec<proc_macro2::TokenStream> {
    let field_serializations = fields.iter().enumerate().map(|(i, f)| {
        let field_type = &f.ty;
        let field_ident = &idents[i];

        let handle_field = generate_code_for_handling_field(
            read,
            field_type,
            field_ident,
            None,
            None,
            None,
            None,
            false,
            false,
            false,
            false,
            0,
        );

        if read {
            quote! {
                #handle_field
                let #field_ident = _p_val;
            }
        } else {
            quote! {
                let _p_val = #field_ident;
                #handle_field
            }
        }
    });
    field_serializations.collect()
}

fn generate_code_for_handling_field(
    read: bool,
    field_type: &Type,
    field_name: &syn::Ident,
    bits_count: Option<u8>,
    toggled_by: Option<String>,
    variant_by: Option<String>,
    length_by: Option<String>,
    is_dynamic_int: bool,
    has_dynamic_length: bool,
    key_dyn_length: bool,
    val_dyn_length: bool,
    level: usize,
) -> proc_macro2::TokenStream {
    if let Type::Path(path) = field_type {
        let path = &path.path;

        if let Some(ident) = path.get_ident() {
            let ident_name = ident.to_string();

            // Single segment without arguments
            match ident_name.as_str() {
                "bool" => {
                    if read {
                        quote! { let _p_val = binary_codec::dynamics::read_bool(_p_bytes, _p_config)?; }
                    } else {
                        quote! { binary_codec::dynamics::write_bool(*_p_val, _p_bytes, _p_config)?; }
                    }
                }
                "i8" => {
                    if let Some(bits_count) = bits_count.as_ref() {
                        if *bits_count < 1 || *bits_count > 7 {
                            panic!("Bits count should be between 1 and 7");
                        }

                        if read {
                            quote! { let _p_val = binary_codec::dynamics::read_small_dynamic_signed(_p_bytes, _p_config, #bits_count)?; }
                        } else {
                            quote! { binary_codec::dynamics::write_small_dynamic_signed(*_p_val, _p_bytes, _p_config, #bits_count)?; }
                        }
                    } else {
                        if read {
                            quote! {
                                let _p_val = binary_codec::dynamics::read_zigzag(_p_bytes, _p_config)?;
                            }
                        } else {
                            quote! {
                                binary_codec::dynamics::write_zigzag(*_p_val, _p_bytes, _p_config)?;
                            }
                        }
                    }
                }
                "u8" => {
                    if let Some(bits_count) = bits_count.as_ref() {
                        if *bits_count < 1 || *bits_count > 7 {
                            panic!("Bits count should be between 1 and 7");
                        }

                        if read {
                            quote! { let _p_val = binary_codec::dynamics::read_small_dynamic_unsigned(_p_bytes, _p_config, #bits_count)?; }
                        } else {
                            quote! { binary_codec::dynamics::write_small_dynamic_unsigned(*_p_val, _p_bytes, _p_config, #bits_count)?; }
                        }
                    } else {
                        if read {
                            quote! {
                                let _p_val = binary_codec::fixed_int::FixedInt::read(_p_bytes, _p_config)?;
                            }
                        } else {
                            quote! {
                                binary_codec::fixed_int::FixedInt::write(*_p_val, _p_bytes, _p_config)?;
                            }
                        }
                    }
                }
                "u16" | "u32" | "u64" | "u128" => {
                    if is_dynamic_int {
                        let dynint: proc_macro2::TokenStream = generate_dynint(read);
                        if read {
                            quote! {
                                #dynint
                                let _p_val = _p_dyn as #ident;
                            }
                        } else {
                            quote! {
                                let _p_dyn = *_p_val as u128;
                                #dynint
                            }
                        }
                    } else {
                        if read {
                            quote! {
                                let _p_val = binary_codec::fixed_int::FixedInt::read(_p_bytes, _p_config)?;
                            }
                        } else {
                            quote! {
                                binary_codec::fixed_int::FixedInt::write(*_p_val, _p_bytes, _p_config)?;
                            }
                        }
                    }
                }
                "i16" | "i32" | "i64" | "i128" => {
                    if is_dynamic_int {
                        let dynint: proc_macro2::TokenStream = generate_dynint(read);
                        if read {
                            quote! {
                                #dynint
                                let _p_val: #ident = binary_codec::fixed_int::ZigZag::to_signed(_p_dyn);
                            }
                        } else {
                            quote! {
                                let _p_dyn = binary_codec::fixed_int::ZigZag::to_unsigned(*_p_val) as u128;
                                #dynint
                            }
                        }
                    } else {
                        if read {
                            quote! {
                                let _p_val = binary_codec::fixed_int::read_zigzag(_p_bytes, _p_config)?;
                            }
                        } else {
                            quote! {
                                binary_codec::fixed_int::write_zigzag(*_p_val, _p_bytes, _p_config)?;
                            }
                        }
                    }
                }
                "String" => {
                    let size_key = generate_size_key(length_by, has_dynamic_length).1;

                    if read {
                        quote! {
                            let _p_val = binary_codec::variable::read_string(_p_bytes, #size_key, _p_config)?;
                        }
                    } else {
                        quote! {
                            binary_codec::variable::write_string(_p_val, #size_key, _p_bytes, _p_config)?;
                        }
                    }
                }
                _ => {
                    let size_key = generate_size_key(length_by, has_dynamic_length).1;

                    let variant_code = if variant_by.is_some() {
                        quote! { 
                            _p_config.discriminator = _p_config.get_variant(#variant_by);
                        }
                    } else { quote! {} };

                    if read {
                        quote! {
                            #variant_code
                            let _p_val = binary_codec::variable::read_object(_p_bytes, #size_key, _p_config)?;
                        }
                    } else {
                        quote! {
                            #variant_code
                            binary_codec::variable::write_object(_p_val, #size_key, _p_bytes, _p_config)?;
                        }
                    }
                }
            }
        } else {
            // Multiple segments, or arguments
            if path.segments.len() == 1 {
                let ident = &path.segments[0].ident;
                let ident_name = ident.to_string();

                match ident_name.as_ref() {
                    "Option" => {
                        let inner_type = get_inner_type(path).expect("Option missing inner type");
                        let handle = generate_code_for_handling_field(
                            read,
                            inner_type,
                            field_name,
                            bits_count,
                            None,
                            variant_by,
                            length_by,
                            is_dynamic_int,
                            has_dynamic_length,
                            key_dyn_length,
                            val_dyn_length,
                            level + 1,
                        );
                        let option_name: syn::Ident = format_ident!("__option_{}", level);

                        if let Some(toggled_by) = toggled_by {
                            // If toggled_by is set, read or write it
                            let toggled_by = quote! {
                                _p_config.get_toggle(#toggled_by).unwrap_or(false)
                            };

                            if read {
                                quote! {
                                    let mut #option_name: Option<#inner_type> = None;
                                    if #toggled_by {
                                        #handle
                                        #option_name = Some(_p_val);
                                    }
                                    let _p_val = #option_name;
                                }
                            } else {
                                quote! {
                                    if #toggled_by {
                                        let _p_val = _p_val.as_ref().expect("Expected Some value, because toggled_by field is true");
                                        #handle
                                    }
                                }
                            }
                        } else {
                            // If space available, read it, write it if not None
                            if read {
                                quote! {
                                    let mut #option_name: Option<#inner_type> = None;
                                    if _p_config.next_reset_bits_pos() < _p_bytes.len() {
                                        #handle
                                        #option_name = Some(_p_val);
                                    }
                                    let _p_val = #option_name;
                                }
                            } else {
                                quote! {
                                    if let Some(_p_val) = _p_val.as_ref() {
                                        #handle
                                    }
                                }
                            }
                        }
                    }
                    "Vec" => {
                        let vec_name = format_ident!("__val_{}", level);
                        let inner_type = get_inner_type(path).expect("Vec missing inner type");
                        let handle = generate_code_for_handling_field(
                            read,
                            inner_type,
                            field_name,
                            bits_count,
                            None,
                            None,
                            None,
                            is_dynamic_int,
                            val_dyn_length,
                            false,
                            false,
                            level + 1,
                        );

                        let (has_size, size_key) = generate_size_key(length_by, has_dynamic_length);

                        let write_code = quote! {
                            for _p_val in _p_val {
                                #handle
                            }
                        };

                        if has_size {
                            if read {
                                quote! {
                                    let _p_len = binary_codec::utils::get_read_size(_p_bytes, #size_key, _p_config)?;
                                    let mut #vec_name = Vec::<#inner_type>::with_capacity(_p_len);
                                    for _ in 0.._p_len {
                                        #handle
                                        #vec_name.push(_p_val);
                                    }
                                    let _p_val = #vec_name;
                                }
                            } else {
                                quote! {
                                    let _p_len = _p_val.len();
                                    binary_codec::utils::write_size(_p_len, #size_key, _p_bytes, _p_config)?;
                                    #write_code
                                }
                            }
                        } else {
                            if read {
                                quote! {
                                    let mut #vec_name = Vec::<#inner_type>::new();
                                    while _p_config.next_reset_bits_pos() < _p_bytes.len() {
                                        #handle
                                        #vec_name.push(_p_val);
                                    }
                                    let _p_val = #vec_name;
                                }
                            } else {
                                quote! {
                                    #write_code
                                }
                            }
                        }
                    }
                    "HashMap" => {
                        let (key_type, value_type) =
                            get_two_types(path).expect("Failed to get HashMap types");

                        let handle_key = generate_code_for_handling_field(
                            read,
                            key_type,
                            field_name,
                            None,
                            None,
                            None,
                            None,
                            is_dynamic_int,
                            key_dyn_length,
                            false,
                            false,
                            level + 1,
                        );

                        let handle_value = generate_code_for_handling_field(
                            read,
                            value_type,
                            field_name,
                            None,
                            None,
                            None,
                            None,
                            is_dynamic_int,
                            val_dyn_length,
                            false,
                            false,
                            level + 1,
                        );

                        let (has_size, size_key) = generate_size_key(length_by, has_dynamic_length);

                        let write_code = quote! {
                            for (key, value) in _p_val {
                                let _p_val = key;
                                #handle_key
                                let _p_val = value;
                                #handle_value
                            }
                        };

                        if read {
                            if has_size {
                                quote! {
                                    let _p_len = binary_codec::utils::get_read_size(_p_bytes, #size_key, _p_config)?;
                                    let mut _p_map = std::collections::HashMap::<#key_type, #value_type>::with_capacity(_p_len);
                                    for _ in 0.._p_len {
                                        let _p_key;
                                        #handle_key
                                        _p_key = _p_val;
                                        let _p_value;
                                        #handle_value
                                        _p_value = _p_val;
                                        _p_map.insert(_p_key, _p_value);
                                    }
                                    let _p_val = _p_map;
                                }
                            } else {
                                quote! {
                                    let mut _p_map = std::collections::HashMap::<#key_type, #value_type>::new();
                                    while _p_config.next_reset_bits_pos() < _p_bytes.len() {
                                        let _p_key;
                                        #handle_key
                                        _p_key = _p_val;
                                        let _p_value;
                                        #handle_value
                                        _p_value = _p_val;
                                        _p_map.insert(_p_key, _p_value);
                                    }
                                    let _p_val = _p_map;
                                }
                            }
                        } else {
                            if has_size {
                                quote! {
                                    let _p_len = _p_val.len();
                                    binary_codec::utils::write_size(_p_len, #size_key, _p_bytes, _p_config)?;
                                    #write_code
                                }
                            } else {
                                quote! {
                                    #write_code
                                }
                            }
                        }
                    }
                    _ => {
                        panic!("Type not implemented")
                    }
                }
            } else {
                panic!("Multi-segment paths are not supported");
            }
        }
    } else if let Type::Array(array) = field_type {
        let len: usize = if let syn::Expr::Lit(ref arr_len_lit) = array.len {
            if let Lit::Int(ref lit_int) = arr_len_lit.lit {
                lit_int
                    .base10_parse()
                    .expect("Failed to parse literal to usize")
            } else {
                panic!("Expected an int to determine array length");
            }
        } else {
            panic!("Expected literal to determine array length");
        };

        let array_type = &array.elem;
        let handle = generate_code_for_handling_field(
            read,
            array_type,
            field_name,
            bits_count,
            None,
            None,
            None,
            is_dynamic_int,
            val_dyn_length,
            false,
            false,
            level + 1,
        );

        let array_name = format_ident!("__val_{}", level);

        if read {
            quote! {
                let mut #array_name = Vec::<#array_type>::with_capacity(#len);
                for _ in 0..#len {
                    #handle;
                    #array_name.push(_p_val);
                }
                let _p_val = TryInto::<[#array_type; #len]>::try_into(#array_name).expect("Failed to convert Vec to array");
            }
        } else {
            quote! {
                for _p_val in _p_val {
                    #handle
                }
            }
        }
    } else {
        panic!("Field type of '{:?}' not supported", field_name);
    }
}

fn generate_error_type(read: bool) -> proc_macro2::TokenStream {
    if read {
        quote! { binary_codec::DeserializationError }
    } else {
        quote! { binary_codec::SerializationError }
    }
}

fn generate_size_key(length_by: Option<String>, has_dynamic_length: bool) -> (bool, proc_macro2::TokenStream) {
    if let Some(length_by) = length_by.as_ref() {
        (true, quote! { Some(#length_by) })
    } else if has_dynamic_length {
        (true, quote! { Some("__dynamic") })
    } else {
        (false, quote! { None })
    }
}

fn get_string_value_from_attribute(
    attr: &Attribute
) -> Option<String> {
    match &attr.meta {
        syn::Meta::Path(_) => {
            None
        }
        syn::Meta::List(list_value) => {
            // #[myattribute("value")]
            for token in list_value.tokens.clone().into_iter() {
                if let proc_macro2::TokenTree::Literal(lit) = token {
                    return Some(lit.to_string().trim_matches('"').to_string());
                }
            }

            None
        }
        syn::Meta::NameValue(name_value) => {
            if let syn::Expr::Lit(lit_expr) = &name_value.value {
                if let Lit::Str(lit_str) = &lit_expr.lit {
                    return Some(lit_str.value());
                }
            }

            None
        }
    }
}

fn get_int_value_from_attribute(attr: &Attribute) -> Option<i32> {
    match &attr.meta {
        syn::Meta::Path(_) => {
            None
        }
        syn::Meta::List(list_value) => {
            // #[myattribute(value)]
            for token in list_value.tokens.clone().into_iter() {
                if let proc_macro2::TokenTree::Literal(lit) = token {
                    if let Ok(val) = lit.to_string().parse::<i32>() {
                        return Some(val);
                    }
                }
            }

            None
        }
        syn::Meta::NameValue(name_value) => {
            if let syn::Expr::Lit(lit_expr) = &name_value.value {
                if let Lit::Int(lit_int) = &lit_expr.lit {
                    return Some(lit_int.base10_parse().expect("Not a valid int value"));
                }
            }

            None
        }
    }
}

fn get_inner_type(path: &syn::Path) -> Option<&syn::Type> {
    if let Some(PathArguments::AngleBracketed(args)) =
        path.segments.last().map(|seg| &seg.arguments)
    {
        if let Some(arg) = args.args.first() {
            if let syn::GenericArgument::Type(inner_type) = arg {
                return Some(inner_type);
            }
        }
    }

    None
}

fn get_two_types(path: &syn::Path) -> Option<(&syn::Type, &syn::Type)> {
    if let Some(PathArguments::AngleBracketed(args)) =
        path.segments.last().map(|seg| &seg.arguments)
    {
        let mut types = args.args.iter().filter_map(|arg| {
            if let syn::GenericArgument::Type(inner_type) = arg {
                Some(inner_type)
            } else {
                None
            }
        });

        if let (Some(t1), Some(t2)) = (types.next(), types.next()) {
            return Some((t1, t2));
        }
    }

    None
}

fn generate_dynint(read: bool) -> proc_macro2::TokenStream {
    if read {
        quote! {
            let _p_dyn = binary_codec::dyn_int::read_dynint(_p_bytes, _p_config)?;
        }
    } else {
        quote! {
            binary_codec::dyn_int::write_dynint(_p_dyn, _p_bytes, _p_config)?;
        }
    }
}