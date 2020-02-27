//! Macros for generating the [`FromKVPFile`](../../bve/parse/kvp/trait.FromKVPFile.html) and
//! [`FromKVPSection`](../../bve/parse/kvp/trait.FromKVPSection.html) traits.
//!
//! This consists of a routine that parses the fields from the struct,
//! then emitting a loop over the sections/fields in the file and matching
//! against the field names.  

#![allow(clippy::default_trait_access)] // Needed by darling

use crate::helpers::combine_token_streams;
use darling::{FromField, FromVariant};
use proc_macro2::Ident;
use quote::quote;
use syn::export::{TokenStream, TokenStream2};
use syn::{GenericArgument, ItemEnum, ItemStruct, PathArguments, Type};

#[allow(clippy::needless_pass_by_value)] // Needed for type deduction
fn split_aliases(input: String) -> Vec<String> {
    if input.is_empty() {
        vec![]
    } else {
        input.split(';').map(str::trim).map(String::from).collect()
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(kvp))]
struct Field {
    ident: Option<Ident>,
    ty: Type,
    #[darling(skip)]
    vec: bool,
    #[darling(default)]
    bare: bool,
    #[darling(default)]
    variadic: bool,
    #[darling(default)]
    rename: Option<String>,
    #[darling(map = "split_aliases", default)]
    alias: Vec<String>,
}

fn parse_fields(item: &ItemStruct) -> Vec<Field> {
    let fields = item.fields.iter().flat_map(Field::from_field);
    let fields: Vec<Field> = fields
        .map(|mut field: Field| {
            if let Type::Path(path) = &field.ty {
                let segments = path.path.segments.last().expect("No path segments found");
                let last_path_str = segments.ident.to_string();
                if last_path_str == "Vec" && !field.variadic {
                    field.vec = true;
                    match &segments.arguments {
                        PathArguments::AngleBracketed(angled) => {
                            match angled.args.first().expect("Generic must have args") {
                                GenericArgument::Type(ty) => field.ty = ty.clone(),
                                _ => unimplemented!(),
                            }
                        }
                        _ => unimplemented!(),
                    }
                }
            };
            field
        })
        .collect();

    fields
}

pub fn kvp_file(item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as ItemStruct);

    let ident = &item.ident;

    let fields = parse_fields(&item);

    // Error Checking

    assert!(
        fields.iter().filter(|f| f.bare).count() <= 1,
        "Cannot have more than 1 bare field"
    );

    let matches = combine_token_streams(fields.iter().map(|field| {
        let ty = field.ty.clone();
        let ident = field.ident.clone().expect("Fields must have names");
        let primary = match &field.bare {
            // The bare section just uses None as a name
            true => quote! {None},
            // Named sections are matched directly
            false => {
                let lower: String = field.rename.as_ref().map_or_else(
                    || ident.to_string().chars().filter(|&c| c != '_').collect(),
                    String::clone,
                );
                quote! {
                    Some(#lower)
                }
            }
        };
        // Enumerate all aliases to append them on the match arm
        let aliases = combine_token_streams(field.alias.iter().map(|alias| {
            quote! {
                | Some(#alias)
            }
        }));
        // Vec fields need to be pushed onto, whereas regular fields need to be assigned.
        let operation = match field.vec {
            true => quote! {{
                let (section_ty, section_warnings) = <#ty as crate::parse::kvp::FromKVPSection>::from_kvp_section(section);
                parsed.#ident.push(section_ty);
                // All section warnings need to be pushed onto the end of the file warnings
                warnings.extend(section_warnings);
            }},
            false => quote! {{
                let (section_ty, section_warnings) = <#ty as crate::parse::kvp::FromKVPSection>::from_kvp_section(section);
                parsed.#ident = section_ty;
                warnings.extend(section_warnings);
            }},
        };
        // Hey look ma, a match arm
        quote! {
            #primary #aliases => #operation,
        }
    }));

    quote! (
        #[automatically_derived]
        impl crate::parse::kvp::FromKVPFile for #ident {
            type Warnings = crate::parse::kvp::KVPGenericWarning;
            fn from_kvp_file(file: &crate::parse::kvp::KVPFile<'_>) -> (Self, Vec<Self::Warnings>) {
                use crate::parse::kvp::FromKVPSection;
                let mut parsed = Self::default();
                let mut warnings = Vec::new();

                for section in &file.sections {
                    #[allow(unreachable_patterns)] // The error catching arms are sometimes redundant
                    match section.name {
                        #matches
                        // Unknown named section
                        Some(name) => warnings.push(crate::parse::kvp::KVPGenericWarning{
                            span: section.span,
                            kind: crate::parse::kvp::KVPGenericWarningKind::UnknownSection {
                                name: String::from(name),
                            }
                        }),
                        // Unknown header section
                        // The header section is always there, so we only care if there's stuff in it
                        None if !section.fields.is_empty() => warnings.push(crate::parse::kvp::KVPGenericWarning{
                            span: section.span,
                            kind: crate::parse::kvp::KVPGenericWarningKind::UnknownSection {
                                name: String::from("<file header>"),
                            }
                        }),
                        // Empty header section is fine
                        None => {}
                    }
                }

                (parsed, warnings)
            }
        }
    )
    .into()
}

pub fn kvp_section(item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as ItemStruct);

    let ident = &item.ident;

    let fields = parse_fields(&item);

    // Error Checking

    let exists_bare_and_vec = fields.iter().filter(|f| f.bare && f.vec).count() >= 1;
    let more_than_one_bare_field = fields.iter().filter(|f| f.bare).count() > 1;
    if exists_bare_and_vec && more_than_one_bare_field {
        panic!("If there are any fields that are bare and of type Vec<T>, there must be only one bare field");
    }

    fields
        .iter()
        .for_each(|f| assert!(!(f.bare && !f.alias.is_empty()), "Bare fields can't have aliases"));

    // Used to assign bare fields indexes as we go. The runtime also keeps track of the current bare field index,
    // matching its index with the index we assign each field.
    let mut bare_field_counter = 0_u64;

    let matches = combine_token_streams(fields.iter().map(|field| {
        let ident = field.ident.clone().expect("Fields must have names");

        let ty = field.ty.clone();
        let primary = match &field.bare {
            // Bare fields must check the counter, and come in the form of ValueData::Value
            true => {
                // If a bare field is a vector, then it is the only bare field, and all values
                // should unconditionally shoved into it.
                let ts = if field.vec {
                    quote! {crate::parse::kvp::ValueData::Value{ value }}
                } else {
                    quote! {crate::parse::kvp::ValueData::Value{ value } if bare_counter == #bare_field_counter}
                };
                bare_field_counter += 1;
                ts
            }
            // Bare fields must check the key name, and come in the form of ValueData::KeyValuePair
            false => {
                let lower: String = field.rename.as_ref().map_or_else(
                    || ident.to_string().chars().filter(|&c| c != '_').collect(),
                    String::clone,
                );
                quote! {
                    crate::parse::kvp::ValueData::KeyValuePair{ key: #lower, value }
                }
            }
        };
        // If there are aliases, iterate over the possibilities
        let aliases = combine_token_streams(field.alias.iter().map(|alias| {
            quote! {
                | crate::parse::kvp::ValueData::KeyValuePair{ key: #alias, value }
            }
        }));
        // If the field is bare, we must also increment the bare_counter in the body
        let bare_operation = if field.bare {
            quote! {bare_counter += 1;}
        } else {
            TokenStream2::new()
        };
        // Vec fields need to be pushed onto, whereas regular fields need to be assigned.
        let operation = match field.vec {
            true => quote! {{
                let optional = <#ty as crate::parse::kvp::FromKVPValue>::from_kvp_value(value);
                // Push a warning if the conversion from a value fails
                if let Some(inner) = optional {
                    parsed.#ident.push(inner);
                } else {
                    warnings.push(crate::parse::kvp::KVPGenericWarning{
                        span: field.span,
                        kind: crate::parse::kvp::KVPGenericWarningKind::InvalidValue {
                            value: String::from(value),
                        }
                    })
                };
                #bare_operation
            }},
            false => quote! {{
                let optional = <#ty as crate::parse::kvp::FromKVPValue>::from_kvp_value(value);
                if let Some(inner) = optional {
                    parsed.#ident = inner;
                } else {
                    warnings.push(crate::parse::kvp::KVPGenericWarning{
                        span: field.span,
                        kind: crate::parse::kvp::KVPGenericWarningKind::InvalidValue {
                            value: String::from(value),
                        }
                    })
                };
                #bare_operation
            }},
        };
        // Hey look ma, a match arm
        quote! {
            #primary #aliases => #operation,
        }
    }));

    quote! (
        #[automatically_derived]
        impl crate::parse::kvp::FromKVPSection for #ident {
            type Warnings = crate::parse::kvp::KVPGenericWarning;
            fn from_kvp_section(section: &crate::parse::kvp::KVPSection<'_>) -> (Self, Vec<Self::Warnings>) {
                let mut parsed = Self::default();
                let mut warnings = Vec::new();
                let mut bare_counter = 0_u64;

                for field in &section.fields {
                    #[allow(unreachable_patterns)] // The error catching arms are sometimes redundant
                    match field.data {
                        #matches
                        // Unknown kvp
                        crate::parse::kvp::ValueData::KeyValuePair{ key, .. } => warnings.push(crate::parse::kvp::KVPGenericWarning{
                            span: field.span,
                            kind: crate::parse::kvp::KVPGenericWarningKind::UnknownField {
                                name: String::from(key),
                            }
                        }),
                        // We have more bare fields than we have places to put them, complain
                        crate::parse::kvp::ValueData::Value{ .. } => warnings.push(crate::parse::kvp::KVPGenericWarning{
                            span: field.span,
                            kind: crate::parse::kvp::KVPGenericWarningKind::UnknownField {
                                name: format!("<bare field {} greater than {} field count>", bare_counter + 1, #bare_field_counter),
                            }
                        }),
                    }
                }

                (parsed, warnings)
            }
        }
    )
    .into()
}

pub fn kvp_value(item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as ItemStruct);

    let ident = &item.ident;

    // Strictly speaking, we don't need that much functionality, but it gets us the core info we need
    let fields = parse_fields(&item);

    let conversion = combine_token_streams(fields.into_iter().map(|field| {
        let ident = field.ident.clone().expect("Fields must have names");
        let ty = field.ty;

        quote! {
            #ident: <#ty as crate::parse::kvp::FromKVPValue>::from_kvp_value(iterator.next()?)?,
        }
    }));

    quote! (
        #[automatically_derived]
        impl crate::parse::kvp::FromKVPValue for #ident {
            fn from_kvp_value(value: &str) -> Option<Self> {
                let mut iterator = value.split(',').map(str::trim);

                Some(#ident {
                    #conversion
                })
            }
        }
    )
    .into()
}

#[derive(Debug, FromVariant)]
#[darling(attributes(kvp))]
struct Variant {
    ident: Ident,
    #[darling(default)]
    default: bool,
    #[darling(default)]
    index: Option<i64>,
}

pub fn kvp_enum_numbers(item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as ItemEnum);

    let ident = item.ident;

    let fields: Vec<Variant> = item
        .variants
        .iter()
        .map(Variant::from_variant)
        .filter_map(Result::ok)
        .collect();

    let default_count = fields.iter().filter(|v| v.default).count();
    assert!(default_count <= 1, "Must not have more than one default field");

    let default_impl = if default_count == 1 {
        let default_ident = fields
            .iter()
            .find(|v| v.default)
            .expect("Must have a default value")
            .ident
            .clone();
        quote! {
            #[automatically_derived]
            impl std::default::Default for #ident {
                fn default() -> Self {
                    Self::#default_ident
                }
            }
        }
    } else {
        quote! {}
    };

    let mut idx = 0_i64;

    let matches = combine_token_streams(fields.iter().map(|variant| {
        let ident = variant.ident.clone();

        if let Some(replacement_idx) = variant.index {
            idx = replacement_idx;
        }

        let new_idx = idx + 1;
        let idx = std::mem::replace(&mut idx, new_idx);

        quote! {
            #idx => Self::#ident,
        }
    }));

    quote! (
        #[automatically_derived]
        impl crate::parse::kvp::FromKVPValue for #ident {
            fn from_kvp_value(value: &str) -> Option<Self> {
                let number = <i64 as crate::parse::kvp::FromKVPValue>::from_kvp_value(value)?;

                Some(match number {
                    #matches
                    _ => return None,
                })
            }
        }

        #default_impl
    )
    .into()
}
