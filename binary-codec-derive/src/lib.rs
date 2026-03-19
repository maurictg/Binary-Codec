extern crate proc_macro;

use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Fields, Lit, PathArguments, Type, parse_macro_input,
    punctuated::Punctuated, token::Comma,
};

#[proc_macro_derive(
    ToBytes,
    attributes(
        bits,
        skip_bits,
        dyn_int,
        dyn_length,
        key_dyn_length,
        val_dyn_length,
        toggles,
        toggled_by,
        toggled_by_variant,
        length_for,
        length_by,
        variant_for,
        variant_by,
        multi_enum,
        no_discriminator,
        discriminator_bits,
        codec_error,
        codec_ser_error,
        codec_de_error,
    )
)]
pub fn generate_code_to_bytes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    generate_code_binary_serializer(false, input)
}

#[proc_macro_derive(
    FromBytes,
    attributes(
        bits,
        skip_bits,
        dyn_int,
        key_dyn_length,
        val_dyn_length,
        dyn_length,
        toggles,
        toggled_by,
        toggled_by_variant,
        length_for,
        length_by,
        variant_for,
        variant_by,
        multi_enum,
        no_discriminator,
        discriminator_bits,
        codec_error,
        codec_ser_error,
        codec_de_error,
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
        _ => panic!("ToBytes can only be used on structs or enums"),
    }
}

fn generate_field_serializer(
    read: bool,
    field_ident: &proc_macro2::Ident,
    field_type: &syn::Type,
    field: &syn::Field,
    is_enum: bool,
) -> proc_macro2::TokenStream {
    let single_ident_type_name = if let Type::Path(path) = field_type {
        if path.path.segments.len() == 1 {
            Some(path.path.segments[0].ident.to_string())
        } else {
            None
        }
    } else {
        None
    };

    let mut toggle_key = None;
    // allow multiple variant_for / length_for entries
    let mut variant_keys: Vec<String> = Vec::new();
    let mut length_keys: Vec<String> = Vec::new();
    let mut toggled_by_variant = None;
    let mut toggled_by = None;
    let mut variant_by = None;
    let mut length_by = None;
    let mut is_dynamic_int = false;
    let mut has_dynamic_length = false;
    let mut bits_count = None;
    let mut skip_bits = None;
    let mut key_dyn_length = false;
    let mut val_dyn_length = false;
    let mut multi_enum = false;

    // Search attributes for length/toggle declarations
    for attr in field.attrs.iter() {
        let ident = attr.path().get_ident().map(|i| i.clone().to_string());
        match ident.as_deref() {
            Some("dyn_int") => is_dynamic_int = true,
            Some("dyn_length") => has_dynamic_length = true,
            Some("key_dyn_length") => key_dyn_length = true,
            Some("val_dyn_length") => val_dyn_length = true,
            Some("multi_enum") => multi_enum = true,
            Some("toggles") => toggle_key = get_string_value_from_attribute(attr),
            Some("variant_for") => {
                if let Some(v) = get_string_value_from_attribute(attr) {
                    variant_keys.push(v);
                }
            }
            Some("length_for") => {
                if let Some(v) = get_string_value_from_attribute(attr) {
                    length_keys.push(v);
                }
            }
            Some("toggled_by") => toggled_by = get_string_value_from_attribute(attr),
            Some("toggled_by_variant") => {
                toggled_by_variant = get_string_value_from_attribute(attr)
            }
            Some("variant_by") => variant_by = get_string_value_from_attribute(attr),
            Some("length_by") => length_by = get_string_value_from_attribute(attr),
            Some("bits") => bits_count = get_int_value_from_attribute(attr).map(|b| b as u8),
            Some("skip_bits") => skip_bits = get_int_value_from_attribute(attr).map(|b| b as u8),
            _ => {} // None => continue
        }
    }

    let val_reference = if matches!(single_ident_type_name, Some(s) if s == String::from("RefCell"))
    {
        if read {
            quote! {
                *#field_ident.borrow()
            }
        } else {
            quote! {
                *_p_val.borrow()
            }
        }
    } else {
        if read {
            quote! {
                _p_val
            }
        } else {
            quote! {
                *_p_val
            }
        }
    };

    // Runtime toggle_key
    let toggles = if let Some(key) = toggle_key {
        quote! {
            _p_config.set_toggle(#key, #val_reference);
        }
    } else {
        quote! {}
    };

    // Runtime length_for keys (support multiple)
    let length_calls: Vec<proc_macro2::TokenStream> = length_keys
        .iter()
        .map(|k| quote! { _p_config.set_length(#k, #val_reference as usize); })
        .collect();

    let length = if !length_calls.is_empty() {
        quote! { #(#length_calls)* }
    } else {
        quote! {}
    };

    // Runtime variant_for keys (support multiple)
    let variant_calls: Vec<proc_macro2::TokenStream> = variant_keys
        .iter()
        .map(|k| quote! { _p_config.set_variant(#k, #val_reference as u8); })
        .collect();

    let variant = if !variant_calls.is_empty() {
        quote! { #(#variant_calls)* }
    } else {
        quote! {}
    };

    // Compose code to handle field
    let f_ident = if is_enum {
        quote! { #field_ident }
    } else {
        quote! { &self.#field_ident }
    };

    let before = if read {
        quote! {}
    } else {
        quote! {
            let _p_val = #f_ident;
            #toggles
            #length
            #variant
        }
    };

    let after = if read {
        quote! {
            let #field_ident = _p_val;
            #toggles
            #length
            #variant
        }
    } else {
        quote! {}
    };

    let skip_bits_code = if let Some(skip) = skip_bits && skip > 0 {
        if read {
            quote! {
                let _ = _p_stream.read_small(#skip)?;
            }
        } else {
            quote! {
                _p_stream.write_small(0, #skip);
            }
        }
    } else {
        quote! {}
    };

    let handle_field = generate_code_for_handling_field(
        read,
        field_type,
        field_ident,
        bits_count,
        toggled_by,
        toggled_by_variant,
        variant_by,
        length_by,
        is_dynamic_int,
        has_dynamic_length,
        key_dyn_length,
        val_dyn_length,
        multi_enum,
        false,
        0,
    );

    quote! {
        #before
        #skip_bits_code
        #handle_field
        #after
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
        generate_field_serializer(
            read,
            &field
                .ident
                .as_ref()
                .expect("binary-codec does not support fields without a name"),
            &field.ty,
            field,
            false,
        )
    });

    let error_type = generate_error_type(read, &ast.attrs);
    let serializer_code = if read {
        let vars = fields.iter().map(|f| f.ident.as_ref().unwrap());

        // read bytes code
        quote! {
            impl<T : Clone> binary_codec::BinaryDeserializer<T, #error_type> for #struct_name {
                fn read_bytes(
                    stream: &mut binary_codec::BitStreamReader,
                    config: Option<&mut binary_codec::SerializerConfig<T>>,
                ) -> Result<Self, #error_type> {
                    let mut _new_config = binary_codec::SerializerConfig::new(None);
                    let _p_config = config.unwrap_or(&mut _new_config);
                    let _p_stream = stream;

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
            impl<T : Clone> binary_codec::BinarySerializer<T, #error_type> for #struct_name {
                fn write_bytes(
                    &self,
                    stream: &mut binary_codec::BitStreamWriter,
                    config: Option<&mut binary_codec::SerializerConfig<T>>,
                ) -> Result<(), #error_type> {
                    let mut _new_config = binary_codec::SerializerConfig::new(None);
                    let _p_config = config.unwrap_or(&mut _new_config);
                    let _p_stream = stream;

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
    let error_type = generate_error_type(read, &ast.attrs);

    let mut no_disc_prefix = false;
    let mut disc_bits = None;

    // Search attributes for variant_by declarations
    for attr in ast.attrs.iter() {
        // #[no_disc_prefix] attribute
        if attr.path().is_ident("no_discriminator") {
            no_disc_prefix = true;
        }

        if attr.path().is_ident("discriminator_bits") {
            disc_bits = get_int_value_from_attribute(attr).map(|b| b as u8);
        }
    }

    if let Some(bits) = disc_bits {
        if no_disc_prefix {
            panic!("Cannot use discriminator_bits and no_discriminator together");
        }

        if bits < 1 || bits > 8 {
            panic!("discriminator_bits should be between 1 and 8");
        }
    }

    let mut configure_functions = Vec::new();

    // Compute discriminant values following Rust rules: explicit values are used,
    // unspecified values get previous + 1 (or 0 for the first unspecified).
    let mut disc_values: Vec<u8> = Vec::with_capacity(data_enum.variants.len());
    let mut last_val: Option<u8> = None;
    for variant in data_enum.variants.iter() {
        let val = if let Some((_, expr)) = &variant.discriminant {
            match expr {
                syn::Expr::Lit(syn::ExprLit {
                    lit: Lit::Int(lit_int),
                    ..
                }) => lit_int
                    .base10_parse::<u8>()
                    .expect("Invalid discriminant integer"),
                _ => panic!("Discriminant must be an integer literal"),
            }
        } else {
            match last_val {
                Some(v) => v + 1,
                None => 0,
            }
        };

        if val > u8::from(u8::MAX) {
            panic!("Discriminant value too large (must fit in u8)");
        }

        disc_values.push(val);
        last_val = Some(val);
    }

    // Create discriminant getter
    let disc_variants = data_enum
        .variants
        .iter()
        .enumerate()
        .map(|(i, variant)| {
            let var_ident = &variant.ident;
            let disc_value = disc_values[i];

            for attr in variant.attrs.iter() {
                if attr.path().is_ident("toggled_by") {
                    let field = get_string_value_from_attribute(attr)
                        .expect("toggled_by for multi_enum should have a value");
                    configure_functions.push(quote! {
                        _p_config.configure_multi_disc(stringify!(#enum_name), #disc_value, #field);
                    });
                }
            }

            match &variant.fields {
                Fields::Unit => quote! {
                    Self::#var_ident => #disc_value
                },
                Fields::Unnamed(_) => quote! {
                    Self::#var_ident(..) => #disc_value
                },
                Fields::Named(_) => quote! {
                    Self::#var_ident { .. } => #disc_value
                },
            }
        })
        .collect::<Vec<_>>();

    // Assign discriminant values starting from 0
    let serialization_variants = data_enum.variants.iter().enumerate().map(|(i, variant)| {
        let var_ident = &variant.ident;
        let disc_value = disc_values[i];
        let fields = &variant.fields;

        // TODO: problem might be that attrs are not used from the fields??.

        let write_disc = if no_disc_prefix {
            quote! {}
        } else {
            let disc_writer = if let Some(bits) = disc_bits {
                quote! {
                    _p_stream.write_small(_p_disc, #bits);
                }
            } else {
                quote! {
                    _p_stream.write_fixed_int(_p_disc);
                }
            };

            quote! {
                let _p_disc: u8 = #disc_value;
                #disc_writer
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
        let disc_reader = if let Some(bits) = disc_bits {
            quote! {
                _p_stream.read_small(#bits)?
            }
        } else {
            quote! {
                 _p_stream.read_fixed_int()?
            }
        };

        quote! {
            impl #enum_name {
                pub fn configure_multi_disc<T : Clone>(config: &mut binary_codec::SerializerConfig<T>) {
                    let _p_config = config;
                    #(#configure_functions)*
                }
            }

            impl<T : Clone> binary_codec::BinaryDeserializer<T, #error_type> for #enum_name {
                fn read_bytes(
                    stream: &mut binary_codec::BitStreamReader,
                    config: Option<&mut binary_codec::SerializerConfig<T>>,
                ) -> Result<Self, #error_type> {
                    let mut _new_config = binary_codec::SerializerConfig::new(None);
                    let _p_config = config.unwrap_or(&mut _new_config);
                    let _p_stream = stream;

                    let _p_disc = if let Some(disc) = _p_config.discriminator.take() {
                        disc
                    } else {
                        #disc_reader
                    };

                    match _p_disc {
                        #(#serialization_variants,)*
                        _ => Err(binary_codec::DeserializationError::UnknownDiscriminant(_p_disc).into()),
                    }
                }
            }
        }
        .into()
    } else {
        quote! {
            impl<T : Clone> binary_codec::BinarySerializer<T, #error_type> for #enum_name {
                fn write_bytes(
                    &self,
                    stream: &mut binary_codec::BitStreamWriter,
                    config: Option<&mut binary_codec::SerializerConfig<T>>,
                ) -> Result<(), #error_type> {
                    let mut _new_config = binary_codec::SerializerConfig::new(None);
                    let _p_config = config.unwrap_or(&mut _new_config);
                    #(#configure_functions)*
                    let _p_stream = stream;

                    match self {
                        #(#serialization_variants)*
                    }

                    Ok(())
                }
            }

            impl #enum_name {
                pub fn get_discriminator(&self) -> u8 {
                    match self {
                        #(#disc_variants,)*
                    }
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

        generate_field_serializer(read, &field_ident, field_type, f, true)
    });
    field_serializations.collect()
}

fn generate_code_for_handling_field(
    read: bool,
    field_type: &Type,
    field_name: &syn::Ident,
    bits_count: Option<u8>,
    toggled_by: Option<String>,
    toggled_by_variant: Option<String>,
    variant_by: Option<String>,
    length_by: Option<String>,
    is_dynamic_int: bool,
    has_dynamic_length: bool,
    key_dyn_length: bool,
    val_dyn_length: bool,
    multi_enum: bool,
    direct_collection_child: bool,
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
                        quote! { let _p_val = _p_stream.read_bit()?;}
                    } else {
                        quote! { _p_stream.write_bit(*_p_val); }
                    }
                }
                "i8" => {
                    if let Some(bits_count) = bits_count.as_ref() {
                        if *bits_count < 1 || *bits_count > 7 {
                            panic!("Bits count should be between 1 and 7");
                        }

                        if read {
                            quote! { let _p_val = binary_codec::ZigZag::to_signed(_p_stream.read_small(#bits_count)?); }
                        } else {
                            quote! { _p_stream.write_small(binary_codec::ZigZag::to_unsigned(*_p_val), #bits_count); }
                        }
                    } else {
                        if read {
                            quote! { let _p_val = _p_stream.read_fixed_int()?; }
                        } else {
                            quote! { _p_stream.write_fixed_int(*_p_val); }
                        }
                    }
                }
                "u8" => {
                    if let Some(bits_count) = bits_count.as_ref() {
                        if *bits_count < 1 || *bits_count > 7 {
                            panic!("Bits count should be between 1 and 7");
                        }

                        if read {
                            quote! { let _p_val = _p_stream.read_small(#bits_count)?; }
                        } else {
                            quote! { _p_stream.write_small(*_p_val, #bits_count); }
                        }
                    } else {
                        if read {
                            quote! { let _p_val = _p_stream.read_byte()?; }
                        } else {
                            quote! { _p_stream.write_byte(*_p_val); }
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
                            quote! { let _p_val = _p_stream.read_fixed_int()?; }
                        } else {
                            quote! { _p_stream.write_fixed_int(*_p_val); }
                        }
                    }
                }
                "i16" | "i32" | "i64" | "i128" => {
                    if is_dynamic_int {
                        let dynint: proc_macro2::TokenStream = generate_dynint(read);
                        if read {
                            quote! {
                                #dynint
                                let _p_val: #ident = binary_codec::ZigZag::to_signed(_p_dyn);
                            }
                        } else {
                            quote! {
                                let _p_dyn = binary_codec::ZigZag::to_unsigned(*_p_val) as u128;
                                #dynint
                            }
                        }
                    } else {
                        if read {
                            quote! { let _p_val = _p_stream.read_fixed_int()?; }
                        } else {
                            quote! { _p_stream.write_fixed_int(*_p_val); }
                        }
                    }
                }
                "f32" | "f64" => {
                    if read {
                        quote! { let _p_val = _p_stream.read_fixed_int()?; }
                    } else {
                        quote! { _p_stream.write_fixed_int(*_p_val); }
                    }
                }
                "String" => {
                    let size_key = generate_size_key(length_by, has_dynamic_length).1;

                    if read {
                        quote! {
                            let _p_val = binary_codec::utils::read_string(_p_stream, #size_key, _p_config)?;
                        }
                    } else {
                        quote! {
                            binary_codec::utils::write_string(_p_val, #size_key, _p_stream, _p_config)?;
                        }
                    }
                }
                _ => {
                    let size_key = generate_size_key(length_by, has_dynamic_length).1;

                    let variant_code = if variant_by.is_some() {
                        quote! {
                            _p_config.discriminator = _p_config.get_variant(#variant_by);
                        }
                    } else if multi_enum {
                        let config_multi = if !direct_collection_child {
                            quote! { #ident::configure_multi_disc(_p_config); }
                        } else {
                            quote! {}
                        };

                        quote! {
                            #config_multi
                            _p_config.discriminator = _p_config.get_next_multi_disc(stringify!(#field_name), #ident_name);
                        }
                    } else {
                        quote! {
                            _p_config.discriminator = None;
                        }
                    };

                    if read {
                        quote! {
                            #variant_code
                            let _p_val = binary_codec::utils::read_object(_p_stream, #size_key, _p_config)?;
                        }
                    } else {
                        quote! {
                            #variant_code
                            binary_codec::utils::write_object(_p_val, #size_key, _p_stream, _p_config)?;
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
                    "RefCell" => {
                        let inner_type = get_inner_type(path).expect("Option missing inner type");
                        let handle = generate_code_for_handling_field(
                            read,
                            inner_type,
                            field_name,
                            bits_count,
                            None,
                            None,
                            variant_by,
                            length_by,
                            is_dynamic_int,
                            has_dynamic_length,
                            key_dyn_length,
                            val_dyn_length,
                            multi_enum,
                            false,
                            level + 1,
                        );

                        if read {
                            quote! {
                                #handle
                                let _p_val = RefCell::new(_p_val);
                            }
                        } else {
                            quote! {
                                let _p_val = &*_p_val.borrow();
                                #handle
                            }
                        }
                    }
                    "Option" => {
                        let inner_type = get_inner_type(path).expect("Option missing inner type");
                        let handle = generate_code_for_handling_field(
                            read,
                            inner_type,
                            field_name,
                            bits_count,
                            None,
                            None,
                            variant_by,
                            length_by,
                            is_dynamic_int,
                            has_dynamic_length,
                            key_dyn_length,
                            val_dyn_length,
                            multi_enum,
                            false,
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
                        } else if let Some(toggled_by_variant) = toggled_by_variant {
                            // If toggled_by_variant is set, read or write it
                            let toggled_by = quote! {
                                _p_config.get_variant_toggle(#toggled_by_variant).unwrap_or(false)
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
                                        let _p_val = _p_val.as_ref().expect("Expected Some value, because toggled_by_variant field evalutates to true");
                                        #handle
                                    }
                                }
                            }
                        } else {
                            // If space available, read it, write it if not None
                            if read {
                                quote! {
                                    let mut #option_name: Option<#inner_type> = None;
                                    if _p_stream.bytes_left() > 0 {
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

                        // If inner type is u8, optimize to bulk read/write bytes
                        if let Type::Path(inner_path) = inner_type {
                            if let Some(inner_ident) = inner_path.path.get_ident() {
                                if inner_ident == "u8" {
                                    let (has_size, size_key) =
                                        generate_size_key(length_by, has_dynamic_length);

                                    if read {
                                        if has_size || multi_enum {
                                            // sized read
                                            let len_code = if multi_enum {
                                                quote! {
                                                    // multi_enum sized Vec<u8>
                                                    let _p_len = _p_config.get_multi_disc_size("u8");
                                                }
                                            } else {
                                                quote! {
                                                    let _p_len = binary_codec::utils::get_read_size(_p_stream, #size_key, _p_config)?;
                                                }
                                            };

                                            return quote! {
                                                #len_code
                                                let _p_val = _p_stream.read_bytes(_p_len)?.to_vec();
                                            };
                                        } else {
                                            // read all remaining bytes
                                            return quote! {
                                                let _p_len = _p_stream.bytes_left();
                                                let _p_val = _p_stream.read_bytes(_p_len)?.to_vec();
                                            };
                                        }
                                    } else {
                                        // write path: if sized, write size first
                                        let write_size = if has_size {
                                            quote! {
                                                let _p_len = _p_val.len();
                                                binary_codec::utils::write_size(_p_len, #size_key, _p_stream, _p_config)?;
                                            }
                                        } else {
                                            quote! {}
                                        };

                                        return quote! {
                                            #write_size
                                            _p_stream.write_bytes(_p_val);
                                        };
                                    }
                                }
                            }
                        }

                        // Fallback to element-wise handling for non-u8 inner types
                        let handle = generate_code_for_handling_field(
                            read,
                            inner_type,
                            field_name,
                            bits_count,
                            None,
                            None,
                            None,
                            None,
                            is_dynamic_int,
                            val_dyn_length,
                            false,
                            false,
                            multi_enum,
                            true,
                            level + 1,
                        );

                        let (has_size, size_key) = generate_size_key(length_by, has_dynamic_length);

                        let write_code = quote! {
                            for _p_val in _p_val {
                                #handle
                            }
                        };

                        if has_size || (read && multi_enum) {
                            if read {
                                let len_code = if multi_enum && let Type::Path(path) = inner_type {
                                    let enum_ident = path
                                        .path
                                        .get_ident()
                                        .expect("Expected ident for multi_enum inner type");
                                    quote! {
                                        #enum_ident::configure_multi_disc(_p_config);
                                        let _p_len = _p_config.get_multi_disc_size(stringify!(#enum_ident));
                                    }
                                } else {
                                    quote! {
                                        let _p_len = binary_codec::utils::get_read_size(_p_stream, #size_key, _p_config)?;
                                    }
                                };

                                quote! {
                                    #len_code
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
                                    binary_codec::utils::write_size(_p_len, #size_key, _p_stream, _p_config)?;
                                    #write_code
                                }
                            }
                        } else {
                            if read {
                                quote! {
                                    let mut #vec_name = Vec::<#inner_type>::new();
                                    while _p_stream.bytes_left() > 0 {
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
                            None,
                            is_dynamic_int,
                            key_dyn_length,
                            false,
                            false,
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
                            None,
                            is_dynamic_int,
                            val_dyn_length,
                            false,
                            false,
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
                                    let _p_len = binary_codec::utils::get_read_size(_p_stream, #size_key, _p_config)?;
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
                                    while _p_stream.bytes_left() > 0 {
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
                                    binary_codec::utils::write_size(_p_len, #size_key, _p_stream, _p_config)?;
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

        let array_type = &*array.elem;
        // Optimize [u8; N] to bulk read_bytes / write_bytes
        if let Type::Path(at_path) = array_type {
            if let Some(at_ident) = at_path.path.get_ident() {
                if at_ident == "u8" {
                    if read {
                        quote! {
                            let _p_slice = _p_stream.read_bytes(#len)?;
                            let _p_val = <[u8; #len]>::try_from(_p_slice).expect("Failed to convert slice to array");
                        }
                    } else {
                        quote! {
                            _p_stream.write_bytes(_p_val);
                        }
                    }
                } else {
                    let handle = generate_code_for_handling_field(
                        read,
                        array_type,
                        field_name,
                        bits_count,
                        None,
                        None,
                        None,
                        None,
                        is_dynamic_int,
                        val_dyn_length,
                        false,
                        false,
                        false,
                        true,
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
                }
            } else {
                // fallback to element handling
                let handle = generate_code_for_handling_field(
                    read,
                    array_type,
                    field_name,
                    bits_count,
                    None,
                    None,
                    None,
                    None,
                    is_dynamic_int,
                    val_dyn_length,
                    false,
                    false,
                    false,
                    true,
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
            }
        } else {
            panic!("Unsupported array element type");
        }
    } else {
        panic!("Field type of '{:?}' not supported", field_name);
    }
}

fn generate_error_type(read: bool, attrs: &[Attribute]) -> proc_macro2::TokenStream {
    if let Some(custom) = get_custom_error_type(read, attrs) {
        return custom;
    }

    if read {
        quote! { binary_codec::DeserializationError }
    } else {
        quote! { binary_codec::SerializationError }
    }
}

fn get_custom_error_type(read: bool, attrs: &[Attribute]) -> Option<proc_macro2::TokenStream> {
    let specific = if read {
        "codec_de_error"
    } else {
        "codec_ser_error"
    };

    let specific_value = attrs
        .iter()
        .find(|attr| attr.path().is_ident(specific))
        .and_then(get_string_value_from_attribute);

    if let Some(value) = specific_value {
        return Some(parse_error_type(&value));
    }

    let common_value = attrs
        .iter()
        .find(|attr| attr.path().is_ident("codec_error"))
        .and_then(get_string_value_from_attribute);

    common_value.map(|value| parse_error_type(&value))
}

fn parse_error_type(value: &str) -> proc_macro2::TokenStream {
    let ty: Type = syn::parse_str(value).expect("Invalid error type for codec_error");
    quote! { #ty }
}

fn generate_size_key(
    length_by: Option<String>,
    has_dynamic_length: bool,
) -> (bool, proc_macro2::TokenStream) {
    if let Some(length_by) = length_by.as_ref() {
        (true, quote! { Some(#length_by) })
    } else if has_dynamic_length {
        (true, quote! { Some("__dynamic") })
    } else {
        (false, quote! { None })
    }
}

fn get_string_value_from_attribute(attr: &Attribute) -> Option<String> {
    match &attr.meta {
        syn::Meta::Path(_) => None,
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
        syn::Meta::Path(_) => None,
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
            let _p_dyn = _p_stream.read_dyn_int()?;
        }
    } else {
        quote! {
            _p_stream.write_dyn_int(_p_dyn);
        }
    }
}
