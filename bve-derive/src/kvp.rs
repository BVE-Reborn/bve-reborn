#![allow(clippy::default_trait_access)] // Needed by darling

use crate::helpers::combine_token_streams;
use darling::FromField;
use proc_macro2::Ident;
use quote::quote;
use syn::export::TokenStream;
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
            true => quote! {
                parsed.#ident.push(<#ty as crate::parse::kvp::FromKVPSection>::from_kvp_section(section))
            },
            false => quote! {
                parsed.#ident = <#ty as crate::parse::kvp::FromKVPSection>::from_kvp_section(section)
            },
        };
        quote! {
            #primary #aliases => #operation,
        }
    }));

    quote! (
        impl crate::parse::kvp::FromKVPFile for #ident {
            fn from_kvp_file(file: &crate::parse::kvp::KVPFile<'_>) -> Self {
                use crate::parse::kvp::FromKVPSection;
                let mut parsed = Self::default();

                for section in &file.sections {
                    match section.name {
                        #matches
                        _ => {}
                    }
                }

                parsed
            }
        }
    )
    .into()
}

pub fn kvp_section(item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as ItemStruct);

    let ident = &item.ident;

    let fields = parse_fields(&item);

    let matches = combine_token_streams(fields.iter().map(|field| {
        let ty = field.ty.clone();
        let ident = field.ident.clone().expect("Fields must have names");
        let primary = match &field.bare {
            true => quote! {crate::parse::kvp::ValueData::Value{ value }},
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
        let operation = match field.vec {
            true => quote! {{
                let optional = <#ty as crate::parse::kvp::FromKVPValue>::from_kvp_value(value);
                if let Some(inner) = optional {
                    parsed.#ident.push(inner);
                }
            }},
            false => quote! {{
                let optional = <#ty as crate::parse::kvp::FromKVPValue>::from_kvp_value(value);
                if let Some(inner) = optional {
                    parsed.#ident = inner;
                }
            }},
        };
        quote! {
            #primary #aliases => #operation,
        }
    }));

    quote! (
        impl crate::parse::kvp::FromKVPSection for #ident {
            fn from_kvp_section(section: &crate::parse::kvp::KVPSection<'_>) -> Self {
                let mut parsed = Self::default();

                for field in &section.fields {
                    match field.data {
                        #matches
                        _ => {}
                    }
                }

                parsed
            }
        }
    )
    .into()
}
