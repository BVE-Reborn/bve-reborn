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
use proc_macro2::{Group, Ident, Literal};
use quote::{format_ident, quote};
use syn::export::{ToTokens, TokenStream2};
use syn::{Attribute, ExprPath, GenericArgument, PathArguments, Type, TypePath, Visibility};

#[derive(Debug)]
struct Field {
    attributes: Vec<Attribute>,
    rename: Option<Literal>,
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

fn get_first_generic_argument(name_vec: &[String], path: &TypePath) -> TokenStream2 {
    let valid = name_vec.len() >= 2;
    let second_name = if valid { name_vec.get(name_vec.len() - 2) } else { None };
    match second_name.map(String::as_str) {
        Some("ColorU8R") | Some("ColorU8RG") | Some("ColorU8RGB") | Some("ColorU8RGBA") => quote!(u8),
        Some("ColorU16R") | Some("ColorU16RG") | Some("ColorU16RGB") | Some("ColorU16RGBA") => quote!(u16),
        Some("ColorF32R") | Some("ColorF32RG") | Some("ColorF32RGB") | Some("ColorF32RGBA") => quote!(f32),
        _ => match &path.path.segments.last().expect("Type path must exist.").arguments {
            PathArguments::AngleBracketed(arg) => match &arg.args[0] {
                GenericArgument::Type(t) => quote!(#t),
                _ => panic!("Vector1 generic argument must be a type"),
            },
            _ => panic!("Vector1 must have generic arguments"),
        },
    }
}

fn process_rename(
    new_name: &Ident,
    inner_type: &TokenStream2,
    rename: &Option<Literal>,
) -> (TokenStream2, TokenStream2, TokenStream2) {
    match rename {
        Some(s) => {
            let l_parsed =
                syn::parse2::<syn::LitStr>(s.to_token_stream()).expect("Argument to rename must be string literal.");
            let default_call: ExprPath = l_parsed.parse().expect("Argument to rename must be a valid ExprPath.");

            let new_type = quote!(Option<#inner_type>);
            let conversion = quote!(match proxy.#new_name {
                Some(v) => v,
                None => #default_call().unwrap(),
            });
            let attribute = quote!(#[serde(default = #s)]);
            (new_type, conversion, attribute)
        }
        None => {
            let new_type = quote!(#inner_type);
            let conversion = quote!(proxy.#new_name);
            let attribute = TokenStream2::new();
            (new_type, conversion, attribute)
        }
    }
}

fn generate_proxy_object(name: &Ident, fields: &[Field]) -> TokenStream2 {
    let new_data = fields
        .iter()
        .map(|field| match &field.ty.0 {
            vec if vec.last().map(String::as_str) == Some("Vector1") => {
                let original_name = &field.name;
                let inner_type = get_first_generic_argument(vec, &field.ty.1);
                let x_new = format_ident!("{}_x", original_name);
                let attributes = combine_attributes(&field.attributes);
                let (x_inner, x_conversion, x_attribute) = process_rename(&x_new, &inner_type, &field.rename);
                let proxy_fields = quote! {
                    #attributes
                    #x_attribute
                    #x_new: #x_inner,
                };
                let from_fields = quote! {
                    #original_name: ::cgmath::Vector1::new(#x_conversion),
                };
                (proxy_fields, from_fields)
            }
            vec if vec.last().map(String::as_str) == Some("Vector2") => {
                let inner_type = get_first_generic_argument(vec, &field.ty.1);
                let original_name = &field.name;
                let x_new = format_ident!("{}_x", original_name);
                let y_new = format_ident!("{}_y", original_name);
                let attributes = combine_attributes(&field.attributes);
                let (x_inner, x_conversion, x_attribute) = process_rename(&x_new, &inner_type.clone(), &field.rename);
                let (y_inner, y_conversion, y_attribute) = process_rename(&y_new, &inner_type, &field.rename);
                let proxy_fields = quote! {
                    #attributes
                    #x_attribute
                    #x_new: #x_inner,

                    #attributes
                    #y_attribute
                    #y_new: #y_inner,
                };
                let from_fields = quote! {
                    #original_name: ::cgmath::Vector2::new(#x_conversion, #y_conversion),
                };
                (proxy_fields, from_fields)
            }
            vec if vec.last().map(String::as_str) == Some("Vector3") => {
                let original_name = &field.name;
                let inner_type = get_first_generic_argument(vec, &field.ty.1);
                let x_new = format_ident!("{}_x", original_name);
                let y_new = format_ident!("{}_y", original_name);
                let z_new = format_ident!("{}_z", original_name);
                let attributes = combine_attributes(&field.attributes);
                let (x_inner, x_conversion, x_attribute) = process_rename(&x_new, &inner_type.clone(), &field.rename);
                let (y_inner, y_conversion, y_attribute) = process_rename(&y_new, &inner_type.clone(), &field.rename);
                let (z_inner, z_conversion, z_attribute) = process_rename(&z_new, &inner_type, &field.rename);
                let proxy_fields = quote! {
                    #attributes
                    #x_attribute
                    #x_new: #x_inner,

                    #attributes
                    #y_attribute
                    #y_new: #y_inner,

                    #attributes
                    #z_attribute
                    #z_new: #z_inner,
                };
                let from_fields = quote! {
                    #original_name: ::cgmath::Vector3::new(#x_conversion, #y_conversion, #z_conversion),
                };
                (proxy_fields, from_fields)
            }
            vec if vec.last().map(String::as_str) == Some("Vector4") => {
                let original_name = &field.name;
                let inner_type = get_first_generic_argument(vec, &field.ty.1);
                let x_new = format_ident!("{}_x", original_name);
                let y_new = format_ident!("{}_y", original_name);
                let z_new = format_ident!("{}_z", original_name);
                let w_new = format_ident!("{}_w", original_name);
                let attributes = combine_attributes(&field.attributes);
                let (x_inner, x_conversion, x_attribute) = process_rename(&x_new, &inner_type.clone(), &field.rename);
                let (y_inner, y_conversion, y_attribute) = process_rename(&y_new, &inner_type.clone(), &field.rename);
                let (z_inner, z_conversion, z_attribute) = process_rename(&z_new, &inner_type.clone(), &field.rename);
                let (w_inner, w_conversion, w_attribute) = process_rename(&w_new, &inner_type, &field.rename);
                let proxy_fields = quote! {
                    #attributes
                    #x_attribute
                    #x_new: #x_inner,

                    #attributes
                    #y_attribute
                    #y_new: #y_inner,

                    #attributes
                    #z_attribute
                    #z_new: #z_inner,

                    #attributes
                    #w_attribute
                    #w_new: #w_inner,
                };
                let from_fields = quote! {
                    #original_name: ::cgmath::Vector4::new(#x_conversion, #y_conversion, #z_conversion, #w_conversion),
                };
                (proxy_fields, from_fields)
            }
            _ => {
                let original_name = &field.name;
                let attributes = combine_attributes(&field.attributes);
                let ty = &field.ty.1;
                let (inner, conversion, attribute) =
                    process_rename(original_name, &ty.to_token_stream(), &field.rename);
                let proxy_fields = quote! {
                    #attributes
                    #attribute
                    #original_name: #inner,
                };
                let from_fields = quote! {
                    #original_name: #conversion,
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

fn find_default_attribute(mut attributes: Vec<Attribute>) -> (Option<Literal>, Vec<Attribute>) {
    let mut rename = None;
    let mut rename_buffer = Vec::new();
    for attr in attributes.drain(0..) {
        match attr
            .path
            .segments
            .first()
            .map(|s| s.ident.to_string())
            .as_ref()
            .map(String::as_str)
        {
            Some("default") => {
                rename = Some(
                    syn::parse2::<Literal>(
                        syn::parse2::<Group>(attr.tokens)
                            .expect("expected group in default")
                            .stream(),
                    )
                    .expect("expected string in default"),
                )
            }
            _ => rename_buffer.push(attr),
        }
    }
    (rename, rename_buffer)
}

#[proc_macro_attribute]
pub fn serde_vector_proxy(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut parsed = syn::parse_macro_input!(item as syn::ItemStruct);

    let mut fields = Vec::new();

    for field in &mut parsed.fields {
        let attributes = field.attrs.clone();
        let (rename, attributes) = find_default_attribute(attributes);
        field.attrs = attributes.clone();
        let viability = field.vis.clone();
        let name = field.ident.clone().expect("Shits gotta have a name.");
        let ty: (Vec<String>, TypePath) = if let Type::Path(p) = &field.ty {
            let mut type_segment_name = Vec::new();
            for segment in &p.path.segments {
                type_segment_name.push(segment.ident.to_string());
            }
            match type_segment_name.last().map(String::as_str) {
                Some("ColorU8R") | Some("ColorU16R") | Some("ColorF32R") => type_segment_name.push("Vector1".into()),
                Some("ColorU8RG") | Some("ColorU16RG") | Some("ColorF32RG") => type_segment_name.push("Vector2".into()),
                Some("ColorU8RGB") | Some("ColorU16RGB") | Some("ColorF32RGB") => {
                    type_segment_name.push("Vector3".into())
                }
                Some("ColorU8RGBA") | Some("ColorU16RGBA") | Some("ColorF32RGBA") => {
                    type_segment_name.push("Vector4".into())
                }
                _ => {}
            }
            (type_segment_name, (*p).clone())
        } else {
            panic!("Why is this anything but a Type::Path?");
        };
        fields.push(Field {
            attributes,
            rename,
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
