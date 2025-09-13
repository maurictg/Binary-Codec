extern crate proc_macro;

use quote::{format_ident, quote};
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, Attribute, Data, DeriveInput, Fields,
    Lit, PathArguments, Type,
};

type FieldReference<'a> = (&'a syn::Ident, Option<usize>);

#[proc_macro_derive(
    ToBytes,
    attributes(
        length_determined_by,
        toggled_by,
        bits,
        dynamic,
        dynamic_len,
        variant_by,
        no_disc_prefix
    )
)]
pub fn generate_code_to_bytes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    generate_code_binary_serializer(false, input)
}

#[proc_macro_derive(
    FromBytes,
    attributes(
        length_determined_by,
        toggled_by,
        bits,
        dynamic,
        dynamic_len,
        variant_by,
        no_disc_prefix
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

        let mut length_determining_field = None;
        let mut toggled_by_field = None;
        let mut variant_by_field = None;
        let mut bits_count_type = None;
        let mut is_dynamic = false;
        let mut dynamic_length_depth = None;

        // Search attributes for length/toggle declarations
        for attr in field.attrs.iter() {
            // #[length_determined_by = "other_field"] attribute
            // or: #[length_determined_by = "other_field.2"] for using index of array/Vec
            if attr.path().is_ident("length_determined_by") {
                length_determining_field = Some(get_field_name_from_attribute(
                    "length_determined_by",
                    attr,
                    fields,
                    field_name,
                ))
            }

            // #[toggled_by = "other_field"] attribute
            // or: #[toggled_by = "other_field.2"] by index of array/Vec
            if attr.path().is_ident("toggled_by") {
                toggled_by_field = Some(get_field_name_from_attribute(
                    "toggled_by",
                    attr,
                    fields,
                    field_name,
                ))
            }

            // #[variant_by = "other_field"] attribute
            // or: #[variant_by = "other_field.2"] by index of array/Vec
            if attr.path().is_ident("variant_by") {
                variant_by_field = Some(get_field_name_from_attribute(
                    "variant_by",
                    attr,
                    fields,
                    field_name,
                ))
            }

            // #[bits = n] attribute
            if attr.path().is_ident("bits") {
                let bits_count = get_int_value_from_attribute("bits", attr, field_name);
                bits_count_type = Some(bits_count as u8);
            }

            // #[dynamic] attribute. If put on an integer, serialize as dyn_int
            if attr.path().is_ident("dynamic") {
                is_dynamic = true;
            }

            // #[dynamic_len] attribute. If put on object, Vec or String: prefix with dyn_int length
            // If you want a Vec to inherit it, use #[dynamic_len(1)] on the Vec to inherit to 1st element
            if attr.path().is_ident("dynamic_len") {
                // Accept #[dynamic_len] or #[dynamic_len(value)] and extract integer if present
                let dynamic_len_value: Option<usize> = get_int_value_from_attribute_2(attr)
                    .or_else(|| Some(1));

                dynamic_length_depth = dynamic_len_value;
            }
        }

        // Compose code to handle field
        let before = if read {
            quote! {}
        } else {
            quote! {
                let _p_val = &self.#field_name;
            }
        };

        let after = if read {
            quote! {
                let #field_name = _p_val;
            }
        } else {
            quote! {}
        };

        let handle_field = generate_code_for_handling_field(
            read,
            field_type,
            field_name,
            bits_count_type,
            is_dynamic,
            dynamic_length_depth,
            length_determining_field,
            toggled_by_field,
            variant_by_field,
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
            fn from_bytes_internal(_p_bytes: &[u8], _p_pos: &mut usize, _p_bits: &mut u8, _p_config: &binary_codec::SerializationConfig) -> Result<Self, #error_type> {
                #(#field_serializations)*

                Ok(Self {
                    #(#vars),*
                })
            }

            pub fn from_bytes(bytes: &[u8], config: &binary_codec::SerializationConfig) -> Result<Self, #error_type> {
                let mut bits = 0;
                let mut pos = 0;
                Self::from_bytes_internal(bytes, &mut pos, &mut bits, config)
            }
        }
    } else {
        // write bytes code
        quote! {
            fn to_bytes_internal(&self, _p_bytes: &mut Vec<u8>, _p_pos: &mut usize, _p_bits: &mut u8, _p_config: &binary_codec::SerializationConfig) -> Result<(), #error_type> {
                #(#field_serializations)*
                Ok(())
            }

            pub fn to_bytes(&self, config: &binary_codec::SerializationConfig) -> Result<Vec<u8>, #error_type> {
                let mut bytes = Vec::new();
                let mut bits = 0;
                let mut pos = 0;
                self.to_bytes_internal(&mut bytes, &mut pos, &mut bits, config)?;
                Ok(bytes)
            }
        }
    };

    quote! {
        impl #struct_name {
            #serializer_code
        }
    }
    .into()
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
        if attr.path().is_ident("no_disc_prefix") {
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
                binary_codec::encodings::FixedInt::write(_p_disc, _p_bytes, _p_pos, _p_bits)?;
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
            impl #enum_name {
                fn from_bytes_internal_with_disc(_p_disc: u8, _p_bytes: &[u8], _p_pos: &mut usize, _p_bits: &mut u8, _p_config: &binary_codec::SerializationConfig) -> Result<Self, #error_type> {
                    match _p_disc {
                        #(#variants,)*
                        _ => Err(#error_type::UnknownDiscriminant(_p_disc)),
                    }
                }

                fn from_bytes_internal(bytes: &[u8], pos: &mut usize, bits: &mut u8, config: &binary_codec::SerializationConfig) -> Result<Self, #error_type> {
                    let _p_disc: u8 = binary_codec::encodings::FixedInt::read(bytes, pos, bits)?;
                    Self::from_bytes_internal_with_disc(_p_disc, bytes, pos, bits, config)
                }

                pub fn from_bytes(bytes: &[u8], config: &binary_codec::SerializationConfig) -> Result<Self, #error_type> {
                    let mut pos = 0;
                    let mut bits = 0;
                    Self::from_bytes_internal(bytes, &mut pos, &mut bits, config)
                }
            }
        }
        .into()
    } else {
        quote! {
            impl #enum_name {
                fn to_bytes_internal(&self, _p_bytes: &mut Vec<u8>, _p_pos: &mut usize, _p_bits: &mut u8, _p_config: &binary_codec::SerializationConfig) -> Result<(), #error_type> {
                    match self {
                        #(#variants)*
                    }
                    Ok(())
                }

                pub fn to_bytes(&self, config: &binary_codec::SerializationConfig) -> Result<Vec<u8>, #error_type> {
                    let mut bytes = Vec::new();
                    let mut pos = 0;
                    let mut bits = 0;
                    self.to_bytes_internal(&mut bytes, &mut pos, &mut bits, config)?;
                    Ok(bytes)
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
            false,
            None,
            None,
            None,
            None,
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
    bits_count_type: Option<u8>,
    is_dynamic: bool,
    dynamic_length_depth: Option<usize>,
    length_determining_field: Option<FieldReference>,
    toggled_by_field: Option<FieldReference>,
    variant_by_field: Option<FieldReference>,
    level: usize,
) -> proc_macro2::TokenStream {
    if let Type::Path(path) = field_type {
        let path = &path.path;

        if let Some(ident) = path.get_ident() {
            let ident_name = ident.to_string();
            // println!(
            //     "Found single segment ident '{:?}: {}'",
            //     field_name, ident_name
            // );

            // Single segment without arguments
            match ident_name.as_str() {
                "bool" => {
                    if read {
                        quote! { let _p_val = binary_codec::serializers::read_bool(_p_bytes, _p_pos, _p_bits)?; }
                    } else {
                        quote! { binary_codec::serializers::write_bool(*_p_val, _p_bytes, _p_pos, _p_bits)?; }
                    }
                }
                "i8" => {
                    if let Some(bits_count) = bits_count_type.as_ref() {
                        if *bits_count < 1 || *bits_count > 7 {
                            panic!("Bits count should be between 1 and 7");
                        }

                        if read {
                            quote! { let _p_val = binary_codec::serializers::read_small_dynamic_signed(_p_bytes, _p_pos, _p_bits, #bits_count)?; }
                        } else {
                            quote! { binary_codec::serializers::write_small_dynamic_signed(*_p_val, _p_bytes, _p_pos, _p_bits, #bits_count)?; }
                        }
                    } else {
                        if read {
                            quote! {
                                let _p_val = binary_codec::encodings::read_zigzag(_p_bytes, _p_pos, _p_bits)?;
                            }
                        } else {
                            quote! {
                                binary_codec::encodings::write_zigzag(*_p_val, _p_bytes, _p_pos, _p_bits)?;
                            }
                        }
                    }
                }
                "u8" => {
                    if let Some(bits_count) = bits_count_type.as_ref() {
                        if *bits_count < 1 || *bits_count > 7 {
                            panic!("Bits count should be between 1 and 7");
                        }

                        if read {
                            quote! { let _p_val = binary_codec::serializers::read_small_dynamic_unsigned(_p_bytes, _p_pos, _p_bits, #bits_count)?; }
                        } else {
                            quote! { binary_codec::serializers::write_small_dynamic_unsigned(*_p_val, _p_bytes, _p_pos, _p_bits, #bits_count)?; }
                        }
                    } else {
                        if read {
                            quote! {
                                let _p_val = binary_codec::encodings::FixedInt::read(_p_bytes, _p_pos, _p_bits)?;
                            }
                        } else {
                            quote! {
                                binary_codec::encodings::FixedInt::write(*_p_val, _p_bytes, _p_pos, _p_bits)?;
                            }
                        }
                    }
                }
                "u16" | "u32" | "u64" | "u128" => {
                    if is_dynamic {
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
                                let _p_val = binary_codec::encodings::FixedInt::read(_p_bytes, _p_pos, _p_bits)?;
                            }
                        } else {
                            quote! {
                                binary_codec::encodings::FixedInt::write(*_p_val, _p_bytes, _p_pos, _p_bits)?;
                            }
                        }
                    }
                }
                "i16" | "i32" | "i64" | "i128" => {
                    if is_dynamic {
                        let dynint: proc_macro2::TokenStream = generate_dynint(read);
                        if read {
                            quote! {
                                #dynint
                                let _p_val: #ident = binary_codec::encodings::ZigZag::to_signed(_p_dyn);
                            }
                        } else {
                            quote! {
                                let _p_dyn = binary_codec::encodings::ZigZag::to_unsigned(*_p_val) as u128;
                                #dynint
                            }
                        }
                    } else {
                        if read {
                            quote! {
                                let _p_val = binary_codec::encodings::read_zigzag(_p_bytes, _p_pos, _p_bits)?;
                            }
                        } else {
                            quote! {
                                binary_codec::encodings::write_zigzag(*_p_val, _p_bytes, _p_pos, _p_bits)?;
                            }
                        }
                    }
                }
                "String" => {
                    // Read and write for String based on two strategies:
                    // 1. using length_determining_field like we do for options's toggled_by. Cast the field to usize
                    // 2. using space left if has_dynamic_len is not set
                    // 3. Try to read space from dyn_int.

                    let (len_specified, dynamic_len) = generate_dynamic_length(
                        read,
                        length_determining_field,
                        dynamic_length_depth,
                        quote! { _string },
                    );

                    if read {
                        if len_specified {
                            quote! {
                                #dynamic_len
                                let _string = &_p_bytes[*_p_pos..*_p_pos + _p_len];
                                let _p_val = String::from_utf8(_string.to_vec()).expect("Invalid string");
                                *_p_pos += _string.len();
                                *_p_bits = 0; // A string should have full _p_bytes, and start with a full byte
                            }
                        } else {
                            quote! {
                                let _string = &_p_bytes[*_p_pos..];
                                let _p_val = String::from_utf8(_string.to_vec()).expect("Invalid string");
                                *_p_pos += _string.len();
                                *_p_bits = 0; // A string should have full _p_bytes, and start with a full byte
                            }
                        }
                    } else {
                        quote! {
                            let _string = _p_val.as_bytes();
                            #dynamic_len
                            _p_bytes.extend_from_slice(_string);
                            *_p_pos += _string.len();
                            *_p_bits = 0;
                        }
                    }
                }
                _ => {
                    // Other types: try to call to_bytes(config) or from_bytes()
                    // It is possible to have length determined
                    let (len_specified, dynamic_len) = generate_dynamic_length(
                        read,
                        length_determining_field,
                        dynamic_length_depth,
                        quote! { _p_slice },
                    );

                    if read {
                        let read_code = if let Some(variant_by) = variant_by_field {
                            let variant_by = get_reference_accessor(variant_by);
                            quote! {
                                let _p_disc = #variant_by;
                                let _p_val = #field_type::from_bytes_internal_with_disc(_p_disc, _p_slice, &mut _s_pos, _p_bits, _p_config)?;
                            }
                        } else {
                            quote! {
                                let _p_val = #field_type::from_bytes_internal(_p_slice, &mut _s_pos, _p_bits, _p_config)?;
                            }
                        };

                        let handle = if len_specified {
                            quote! {
                                #dynamic_len
                                let __s_pos = if *_p_bits != 0 && *_p_pos != 0 {
                                    *_p_pos - 1
                                } else {
                                    *_p_pos
                                };
                                let _p_slice = &_p_bytes[__s_pos..__s_pos + _p_len];
                            }
                        } else {
                            quote! {
                                let __s_pos = if *_p_bits != 0 && *_p_pos != 0 {
                                    *_p_pos - 1
                                } else {
                                 *_p_pos
                                };
                                let _p_slice = &_p_bytes[__s_pos..];
                            }
                        };

                        // It MIGHT be that the next objects reads bits from the last byte
                        quote! {
                            #handle
                            let mut _s_pos = 0;
                            #read_code
                            *_p_pos += _s_pos;
                        }
                    } else {
                        if len_specified {
                            quote! {
                                let mut _s_pos = 0;
                                let mut _vec: Vec<u8> = Vec::new();
                                _p_val.to_bytes_internal(&mut _vec, &mut _s_pos, _p_bits, _p_config)?;
                                let _p_slice = &_vec;
                                #dynamic_len
                                _p_bytes.extend_from_slice(_p_slice);
                                *_p_pos += _s_pos;
                            }
                        } else {
                            quote! {
                                _p_val.to_bytes_internal(_p_bytes, _p_pos, _p_bits, _p_config)?;
                            }
                        }
                    }
                }
            }
        } else {
            // Multiple segments, or arguments
            if path.segments.len() == 1 {
                let ident = &path.segments[0].ident;
                let ident_name = ident.to_string();

                // println!(
                //     "Found multi segment ident '{:?}: {}'",
                //     field_name, ident_name
                // );

                match ident_name.as_ref() {
                    "Option" => {
                        let inner_type = get_inner_type(path).expect("Option missing inner type");
                        let handle = generate_code_for_handling_field(
                            read,
                            inner_type,
                            field_name,
                            bits_count_type,
                            is_dynamic,
                            dynamic_length_depth,
                            length_determining_field,
                            None,
                            variant_by_field,
                            level + 1,
                        );
                        let option_name: syn::Ident = format_ident!("__option_{}", level);

                        if let Some(toggled_by) = toggled_by_field {
                            let toggled_by = get_reference_accessor(toggled_by);
                            // If toggled_by is set, read or write it
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
                                    if self.#toggled_by {
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
                                    if *_p_pos < _p_bytes.len() {
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
                            bits_count_type,
                            is_dynamic,
                            dynamic_length_depth.map(|d| d - 1),
                            None,
                            None,
                            None,
                            level + 1,
                        );

                        let (len_specified, dynamic_len) = generate_dynamic_length(
                            read,
                            length_determining_field,
                            dynamic_length_depth,
                            quote! { _p_val },
                        );

                        if read {
                            if len_specified {
                                quote! {
                                    #dynamic_len
                                    let mut #vec_name = Vec::<#inner_type>::with_capacity(_p_len);
                                    for _ in 0.._p_len {
                                        #handle
                                        #vec_name.push(_p_val);
                                    }
                                    let _p_val = #vec_name;
                                }
                            } else {
                                quote! {
                                    let mut #vec_name = Vec::<#inner_type>::new();
                                    while *_p_pos < _p_bytes.len() {
                                        #handle
                                        #vec_name.push(_p_val);
                                    }
                                    let _p_val = #vec_name;
                                }
                            }
                        } else {
                            quote! {
                                #dynamic_len
                                for _p_val in _p_val {
                                    #handle
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
                            bits_count_type,
                            is_dynamic,
                            dynamic_length_depth.map(|d| d - 1),
                            None,
                            None,
                            None,
                            level + 1,
                        );

                        let handle_value = generate_code_for_handling_field(
                            read,
                            value_type,
                            field_name,
                            bits_count_type,
                            is_dynamic,
                            dynamic_length_depth.map(|d| d - 1),
                            None,
                            None,
                            None,
                            level + 1,
                        );

                        let (len_specified, dynamic_len) = generate_dynamic_length(
                            read,
                            length_determining_field,
                            dynamic_length_depth,
                            quote! { _p_val },
                        );

                        if read {
                            if len_specified {
                                quote! {
                                    #dynamic_len
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
                                    while *_p_pos < _p_bytes.len() {
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
                            quote! {
                                #dynamic_len
                                for (key, value) in _p_val {
                                    let _p_val = key;
                                    #handle_key
                                    let _p_val = value;
                                    #handle_value
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

        // println!("Found array '{:?}' with length: {}", field_name, len);

        let array_type = &array.elem;
        let handle = generate_code_for_handling_field(
            read,
            array_type,
            field_name,
            bits_count_type,
            is_dynamic,
            dynamic_length_depth,
            None,
            None,
            None,
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

fn get_string_value_from_attribute(
    attribute_name: &str,
    attr: &Attribute,
    field_name: &syn::Ident,
) -> String {
    if let syn::Meta::NameValue(name_value) = &attr.meta {
        if let syn::Expr::Lit(lit_expr) = &name_value.value {
            if let Lit::Str(lit_str) = &lit_expr.lit {
                lit_str.value()
            } else {
                panic!(
                    "Expected a string for {} above '{}'",
                    attribute_name, field_name
                );
            }
        } else {
            panic!(
                "Expected field name for {} above '{}'",
                attribute_name, field_name
            );
        }
    } else {
        panic!(
            "Expected '{}' {} to specify field name",
            attribute_name, field_name
        );
    }
}

fn get_int_value_from_attribute(
    attribute_name: &str,
    attr: &Attribute,
    field_name: &syn::Ident,
) -> i32 {
    if let syn::Meta::NameValue(name_value) = &attr.meta {
        if let syn::Expr::Lit(lit_expr) = &name_value.value {
            if let Lit::Int(lit_str) = &lit_expr.lit {
                lit_str.base10_parse().expect("Not a valid int value")
            } else {
                panic!(
                    "Expected a int for {} above '{}'",
                    attribute_name, field_name
                );
            }
        } else {
            panic!(
                "Expected field name for {} above '{}'",
                attribute_name, field_name
            );
        }
    } else {
        panic!(
            "Expected '{}' {} to specify field name",
            attribute_name, field_name
        );
    }
}

fn get_int_value_from_attribute_2(attr: &Attribute) -> Option<usize> {
    match &attr.meta {
        syn::Meta::Path(_) => {
            None
        }
        syn::Meta::List(list_value) => {
            // #[dynamic_len(value)]
            for token in list_value.tokens.clone().into_iter() {
                if let proc_macro2::TokenTree::Literal(lit) = token {
                    if let Ok(val) = lit.to_string().parse::<usize>() {
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

fn get_field_name_from_attribute<'a>(
    attribute_name: &str,
    attr: &Attribute,
    fields: &'a Fields,
    field_name: &syn::Ident,
) -> (&'a syn::Ident, Option<usize>) {
    let field_name = get_string_value_from_attribute(attribute_name, attr, field_name);
    let mut index: Option<usize> = None;

    let field_name = if field_name.contains('.') {
        let parts: Vec<&str> = field_name.split('.').collect();
        if parts.len() != 2 {
            panic!(
                "Invalid field name '{}' for attribute '{}', expected 'field_name.index'",
                field_name, attribute_name
            );
        }

        index = parts[1].parse().ok();
        parts[0].to_string()
    } else {
        field_name
    };

    let determining_field = fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|i| i == &field_name))
        .expect(&format!("Referenced field '{}' not found", field_name));

    let field = determining_field.ident.as_ref().expect(&format!(
        "Referenced field '{}' has no name, which is not supported",
        field_name
    ));

    (field, index)
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

fn get_reference_accessor(field_reference: FieldReference) -> proc_macro2::TokenStream {
    let name = field_reference.0;
    if let Some(index) = field_reference.1 {
        quote! { #name[#index] }
    } else {
        quote! { #name }
    }
}

fn generate_dynint(read: bool) -> proc_macro2::TokenStream {
    if read {
        quote! {
            let (_p_dyn, _bytes_read) = binary_codec::dyn_int::read_from_slice(&_p_bytes[*_p_pos..])?;
            *_p_pos += _bytes_read;
            *_p_bits = 0;
        }
    } else {
        quote! {
            let _p_enc = binary_codec::dyn_int::encode(_p_dyn);
            _p_bytes.extend_from_slice(&_p_enc);
            *_p_pos += _p_enc.len();
            *_p_bits = 0;
        }
    }
}

/**
 * Generate code writing or reading dynamic integer, or reading and validating length determining field in struct
 * If the length is specified this produces:
 * read:
 * let _p_len : usize = ...;
 */
fn generate_dynamic_length(
    read: bool,
    length_determining_field: Option<(&syn::Ident, Option<usize>)>,
    dynamic_length_depth: Option<usize>,
    item: proc_macro2::TokenStream,
) -> (bool, proc_macro2::TokenStream) {
    let dynint = generate_dynint(read);
    if read {
        if let Some(length_determining_field) = length_determining_field {
            let length_determining_field = get_reference_accessor(length_determining_field);
            (
                true,
                quote! {
                    let _p_len = #length_determining_field as usize;
                },
            )
        } else {
            if dynamic_length_depth.is_some_and(|v| v > 0) {
                (
                    true,
                    quote! {
                        #dynint
                        let _p_len = _p_dyn as usize;
                    },
                )
            } else {
                (false, quote! {})
            }
        }
    } else {
        if let Some(length_determining_field) = length_determining_field {
            let length_determining_field = get_reference_accessor(length_determining_field);
            (
                true,
                quote! {
                    let expected_len = self.#length_determining_field as usize;
                    if #item.len() != expected_len {
                        return Err(binary_codec::SerializationError::UnexpectedLength(expected_len, #item.len()));
                    }
                },
            )
        } else {
            if dynamic_length_depth.is_some_and(|v| v > 0) {
                (
                    true,
                    quote! {
                        let _p_dyn = #item.len() as u128;
                        #dynint
                    },
                )
            } else {
                (false, quote! {})
            }
        }
    }
}
