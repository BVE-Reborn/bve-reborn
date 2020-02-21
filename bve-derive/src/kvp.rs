#![allow(clippy::default_trait_access)] // Needed by darling

use crate::helpers::combine_token_streams;
use darling::FromField;
use proc_macro2::Ident;
use quote::quote;
use syn::export::{TokenStream, TokenStream2};
use syn::{GenericArgument, ItemStruct, PathArguments, Type};

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

    assert!(
        fields.iter().filter(|f| f.bare).count() <= 1,
        "Cannot have more than 1 bare field"
    );

    let matches = combine_token_streams(fields.iter().map(|field| {
        let ty = field.ty.clone();
        let ident = field.ident.clone().expect("Fields must have names");
        let primary = match &field.bare {
            true => quote! {None},
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
        let aliases = combine_token_streams(field.alias.iter().map(|alias| {
            quote! {
                | Some(#alias)
            }
        }));
        let operation = match field.vec {
            true => quote! {{
                let (section_ty, section_warnings) = <#ty as crate::parse::kvp::FromKVPSection>::from_kvp_section(section);
                parsed.#ident.push(section_ty);
                warnings.extend(section_warnings);
            }},
            false => quote! {{
                let (section_ty, section_warnings) = <#ty as crate::parse::kvp::FromKVPSection>::from_kvp_section(section);
                parsed.#ident = section_ty;
                warnings.extend(section_warnings);
            }},
        };
        quote! {
            #primary #aliases => #operation,
        }
    }));

    quote! (
        impl crate::parse::kvp::FromKVPFile for #ident {
            type Warnings = crate::parse::kvp::KVPGenericWarning;
            fn from_kvp_file(file: &crate::parse::kvp::KVPFile<'_>) -> (Self, Vec<Self::Warnings>) {
                use crate::parse::kvp::FromKVPSection;
                let mut parsed = Self::default();
                let mut warnings = Vec::new();

                for section in &file.sections {
                    #[allow(unreachable_patterns)]
                    match section.name {
                        #matches
                        Some(name) => warnings.push(crate::parse::kvp::KVPGenericWarning{
                            span: section.span,
                            kind: crate::parse::kvp::KVPGenericWarningKind::UnknownSection {
                                name: String::from(name),
                            }
                        }),
                        // The header section is always there, so we only care if there's stuff in it
                        None if !section.fields.is_empty() => warnings.push(crate::parse::kvp::KVPGenericWarning{
                            span: section.span,
                            kind: crate::parse::kvp::KVPGenericWarningKind::UnknownSection {
                                name: String::from("<file header>"),
                            }
                        }),
                        // Empty header section
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
    let mut bare_field_counter = 0_u64;

    let matches = combine_token_streams(fields.iter().map(|field| {
        let ty = field.ty.clone();
        let ident = field.ident.clone().expect("Fields must have names");
        let primary = match &field.bare {
            true => {
                let ts = quote! {crate::parse::kvp::ValueData::Value{ value } if bare_counter == #bare_field_counter};
                bare_field_counter += 1;
                ts
            }
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
        let aliases = combine_token_streams(field.alias.iter().map(|alias| {
            quote! {
                | crate::parse::kvp::KVPInnerData::KeyValuePair{ key: #alias, value }
            }
        }));
        let bare_operation = if field.bare {
            quote! {bare_counter += 1;}
        } else {
            TokenStream2::new()
        };
        let operation = match field.vec {
            true => quote! {{
                let optional = <#ty as crate::parse::kvp::FromKVPValue>::from_kvp_value(value);
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
        quote! {
            #primary #aliases => #operation,
        }
    }));

    quote! (
        impl crate::parse::kvp::FromKVPSection for #ident {
            type Warnings = crate::parse::kvp::KVPGenericWarning;
            fn from_kvp_section(section: &crate::parse::kvp::KVPSection<'_>) -> (Self, Vec<Self::Warnings>) {
                let mut parsed = Self::default();
                let mut warnings = Vec::new();
                let mut bare_counter = 0_u64;

                for field in &section.fields {
                    match field.data {
                        #matches
                        crate::parse::kvp::ValueData::KeyValuePair{ key, .. } => warnings.push(crate::parse::kvp::KVPGenericWarning{
                            span: field.span,
                            kind: crate::parse::kvp::KVPGenericWarningKind::UnknownField {
                                name: String::from(key),
                            }
                        }),
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
