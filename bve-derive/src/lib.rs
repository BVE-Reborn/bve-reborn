// Rust warnings
#![warn(unused)]
#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![forbid(unsafe_code)]
// Clippy warnings
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::restriction)]
// Annoying regular clippy warnings
#![allow(clippy::cast_sign_loss)] // Annoying
#![allow(clippy::cast_precision_loss)] // Annoying
#![allow(clippy::cast_possible_truncation)] // Annoying
#![allow(clippy::cognitive_complexity)] // This is dumb
// Annoying/irrelevant clippy Restrictions
#![allow(clippy::decimal_literal_representation)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::float_cmp_const)]
#![allow(clippy::implicit_return)]
#![allow(clippy::indexing_slicing)] // Proc macros are error prone
#![allow(clippy::integer_arithmetic)]
#![allow(clippy::integer_division)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::shadow_reuse)]
#![allow(clippy::shadow_same)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::wildcard_enum_match_arm)]

extern crate proc_macro;

use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::export::{ToTokens, TokenStream2};
use syn::{Attribute, GenericArgument, PathArguments, Type, TypePath, Visibility};

#[derive(Debug)]
struct Field {
    attributes: Vec<Attribute>,
    visibility: Visibility,
    name: Ident,
    ty: (Vec<String>, TypePath),
}

fn combine_token_streams<I: IntoIterator<Item = TokenStream2>>(streams: I) -> TokenStream2 {
    streams
        .into_iter()
        .fold1(|mut l, r| {
            l.extend(r);
            l
        })
        .unwrap_or_else(TokenStream2::new)
}

fn combine_attributes(attributes: &[Attribute]) -> TokenStream2 {
    attributes
        .iter()
        .map(Attribute::to_token_stream)
        .fold1(|mut l, r| {
            l.extend(r);
            l
        })
        .unwrap_or_else(TokenStream2::new)
}

fn get_first_generic_argument(path: &TypePath) -> &Type {
    match &path.path.segments.last().expect("Type path must exist.").arguments {
        PathArguments::AngleBracketed(arg) => match &arg.args[0] {
            GenericArgument::Type(t) => t,
            _ => panic!("Vector1 generic argument must be a type"),
        },
        _ => panic!("Vector1 must have generic arguments"),
    }
}

fn generate_proxy_object(name: &Ident, fields: &[Field]) -> TokenStream2 {
    let new_data = fields
        .iter()
        .map(|field| match &field.ty.0 {
            vec if vec.last().map(String::as_str) == Some("Vector1") => {
                let original_name = &field.name;
                let inner_type = get_first_generic_argument(&field.ty.1);
                let x_new = format_ident!("{}_x", original_name);
                let attributes = combine_attributes(&field.attributes);
                let proxy_fields = quote! {
                    #attributes
                    #x_new: #inner_type,
                };
                let from_fields = quote! {
                    #original_name: ::cgmath::Vector1::new(proxy.#x_new),
                };
                (proxy_fields, from_fields)
            }
            vec if vec.last().map(String::as_str) == Some("Vector2") => {
                let inner_type = get_first_generic_argument(&field.ty.1);
                let original_name = &field.name;
                let x_new = format_ident!("{}_x", original_name);
                let y_new = format_ident!("{}_y", original_name);
                let attributes = combine_attributes(&field.attributes);
                let proxy_fields = quote! {
                    #attributes
                    #x_new: #inner_type,
                    #attributes
                    #y_new: #inner_type,
                };
                let from_fields = quote! {
                    #original_name: ::cgmath::Vector2::new(proxy.#x_new, proxy.#y_new),
                };
                (proxy_fields, from_fields)
            }
            vec if vec.last().map(String::as_str) == Some("Vector3") => {
                let original_name = &field.name;
                let inner_type = get_first_generic_argument(&field.ty.1);
                let x_new = format_ident!("{}_x", original_name);
                let y_new = format_ident!("{}_y", original_name);
                let z_new = format_ident!("{}_z", original_name);
                let attributes = combine_attributes(&field.attributes);
                let proxy_fields = quote! {
                    #attributes
                    #x_new: #inner_type,
                    #attributes
                    #y_new: #inner_type,
                    #attributes
                    #z_new: #inner_type,
                };
                let from_fields = quote! {
                    #original_name: ::cgmath::Vector3::new(proxy.#x_new, proxy.#y_new, proxy.#z_new),
                };
                (proxy_fields, from_fields)
            }
            vec if vec.last().map(String::as_str) == Some("Vector4") => {
                let original_name = &field.name;
                let inner_type = get_first_generic_argument(&field.ty.1);
                let x_new = format_ident!("{}_x", original_name);
                let y_new = format_ident!("{}_y", original_name);
                let z_new = format_ident!("{}_z", original_name);
                let w_new = format_ident!("{}_w", original_name);
                let attributes = combine_attributes(&field.attributes);
                let proxy_fields = quote! {
                    #attributes
                    #x_new: #inner_type,
                    #attributes
                    #y_new: #inner_type,
                    #attributes
                    #z_new: #inner_type,
                    #attributes
                    #w_new: #inner_type,
                };
                let from_fields = quote! {
                    #original_name: ::cgmath::Vector4::new(proxy.#x_new, proxy.#y_new, proxy.#z_new, proxy.#w_new),
                };
                (proxy_fields, from_fields)
            }
            _ => {
                let original_name = &field.name;
                let attributes = combine_attributes(&field.attributes);
                let ty = &field.ty.1;
                let proxy_fields = quote! {
                    #attributes
                    #original_name: #ty
                };
                let from_fields = quote! {
                    #original_name: proxy.#original_name,
                };
                (proxy_fields, from_fields)
            }
        })
        .collect_vec();

    let proxy_fields = combine_token_streams(new_data.iter().map(|(proxy_field, _)| proxy_field.clone()));
    let from_fields = combine_token_streams(new_data.into_iter().map(|(_, from_field)| from_field));

    let proxy_name = format_ident!("{}{}", name, "SerdeProxy");

    quote!(
        #[derive(Deserialize)]
        struct #proxy_name {
            #proxy_fields
        }

        impl ::std::convert::From<#proxy_name> for #name {
            #[inline]
            fn from(proxy: #proxy_name) -> #name {
                #name {
                    #from_fields
                }
            }
        }
    )
}

#[proc_macro_attribute]
pub fn serde_vector_proxy(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(item as syn::ItemStruct);

    let mut fields = Vec::new();

    for field in &parsed.fields {
        let attributes = field.attrs.clone();
        let viability = field.vis.clone();
        let name = field.ident.clone().expect("Shits gotta have a name.");
        let ty: (Vec<String>, TypePath) = if let Type::Path(p) = &field.ty {
            let mut type_segment_name = Vec::new();
            for segment in &p.path.segments {
                type_segment_name.push(segment.ident.to_string());
            }
            (type_segment_name, (*p).clone())
        } else {
            panic!("Why is this anything but a Type::Path?");
        };
        fields.push(Field {
            attributes,
            visibility: viability,
            name,
            ty,
        })
    }

    let proxy = generate_proxy_object(&parsed.ident, &fields);
    let proxy_name = format!("{}SerdeProxy", &parsed.ident);

    let current = quote!(
        #proxy

        #[derive(Debug, Clone, PartialEq, Deserialize)]
        #[serde(from = #proxy_name)]
        #parsed
    );

    current.into()
}
