//! Macros for generating the [`FromKVPFile`](../../bve/parse/kvp/trait.FromKVPFile.html) and
//! [`FromKVPSection`](../../bve/parse/kvp/trait.FromKVPSection.html) traits.
//!
//! This consists of a routine that parses the fields from the struct,
//! then emitting a loop over the sections/fields in the file and matching
//! against the field names.  

#![allow(clippy::default_trait_access)] // Needed by darling

use crate::helpers::combine_token_streams;
use darling::{FromField, FromVariant};
use itertools::Itertools;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    export::{TokenStream, TokenStream2},
    GenericArgument, ItemEnum, ItemStruct, PathArguments, Type,
};

#[allow(clippy::needless_pass_by_value)] // Needed for type deduction
fn split_aliases(input: String) -> Vec<String> {
    if input.is_empty() {
        vec![]
    } else {
        input.split(';').map(str::trim).map(str::to_lowercase).collect()
    }
}

fn lowercase(input: Option<String>) -> Option<String> {
    input.map(|s| s.to_lowercase())
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum FieldKind {
    Normal,
    Vec,
    Hash,
}

impl Default for FieldKind {
    fn default() -> Self {
        Self::Normal
    }
}

/// Struct field with possible attributes as returned by [`parse_fields`]
#[derive(Debug, FromField)]
#[darling(attributes(kvp))]
struct Field {
    ident: Option<Ident>,
    ty: Type,
    #[darling(skip)]
    kind: FieldKind,
    #[darling(default)]
    bare: bool,
    #[darling(default)]
    variadic: bool,
    #[darling(map = "lowercase", default)]
    rename: Option<String>,
    #[darling(map = "split_aliases", default)]
    alias: Vec<String>,
}

fn parse_fields(item: &ItemStruct) -> Vec<Field> {
    let fields = item.fields.iter().flat_map(Field::from_field);
    let fields: Vec<Field> = fields
        .map(|mut field: Field| {
            if let Type::Path(path) = field.ty.clone() {
                let segments = path.path.segments.last().expect("No path segments found");
                let last_path_str = segments.ident.to_string();
                if last_path_str == "Vec" && !field.variadic {
                    field.kind = FieldKind::Vec;
                    match &segments.arguments {
                        PathArguments::AngleBracketed(angled) => {
                            match angled.args.first().expect("Generic must have args") {
                                GenericArgument::Type(ty) => field.ty = ty.clone(),
                                _ => unreachable!("Vec takes a single type argument"),
                            }
                        }
                        _ => unreachable!("Vec takes a single angle bracketed argument"),
                    }
                }
                if last_path_str == "HashMap" {
                    field.kind = FieldKind::Hash;
                    match &segments.arguments {
                        PathArguments::AngleBracketed(angled) => {
                            match angled.args.last().expect("Generic must have args") {
                                GenericArgument::Type(ty) => field.ty = ty.clone(),
                                _ => unreachable!("HashMap takes two type arguments"),
                            }
                        }
                        _ => unreachable!("HashMap takes two angle bracketed arguments"),
                    }
                }
            };
            field
        })
        .collect();

    fields
}

fn generate_pretty_print_impls<'a>(iter: impl IntoIterator<Item = &'a Field> + 'a) -> TokenStream2 {
    combine_token_streams(iter.into_iter().map(|field| {
        let ident = field.ident.clone().expect("Fields must have names");
        let name_plus_colon: String = field.rename.as_ref().map_or_else(
            || ident.to_string().chars().filter(|&c| c != '_').collect(),
            String::clone,
        ) + ": ";

        let ty = field.ty.clone();

        let real_ty = match field.kind {
            FieldKind::Normal => quote! {#ty},
            FieldKind::Vec => quote! {Vec<#ty>},
            FieldKind::Hash => {
                quote! {std::collections::HashMap<u64, #ty>}
            }
        };

        quote! {
            crate::parse::util::indent(indent, out)?;
            write!(out, #name_plus_colon)?;
            <#real_ty as crate::parse::PrettyPrintResult>::fmt(&self.#ident, indent + 1, out)?;
        }
    }))
}

pub fn kvp_file(item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as ItemStruct);

    let ident = &item.ident;
    let ident_str = ident.to_string();
    let ident_str_colon = ident_str + ":";

    let fields = parse_fields(&item);

    // Error Checking

    assert!(
        fields.iter().filter(|f| f.bare).count() <= 1,
        "Cannot have more than 1 bare field"
    );

    let matches = combine_token_streams(fields.iter().map(|field| {
        let ty = field.ty.clone();
        let ident = field.ident.clone().expect("Fields must have names");

        let non_bare_ident: String = field.rename.as_ref().map_or_else(
            || ident.to_string().chars().filter(|&c| c != '_').collect(),
            String::clone,
        );
        let non_bare_ident_len = non_bare_ident.len();

        let primary = match (&field.bare, &field.kind) {
            // The bare section just uses None as a name
            (true, _) => quote! {None},
            // Named sections are matched directly
            (false, FieldKind::Hash) => {
                quote! {
                    Some(section_name)
                        if section_name.starts_with(#non_bare_ident) // Starts with the name we want
                        && section_name.len() > #non_bare_ident_len // Has at least 1 more character than the name (there must be a number)
                        && !section_name[#non_bare_ident_len..].trim().chars().any(|c| !c.is_digit(10)) // All of the other characters are chars
                }
            }
            (false, _) => {
                quote! {
                    Some(#non_bare_ident)
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
        let operation = match field.kind {
            FieldKind::Vec => quote! {{
                let (section_ty, section_warnings) = <#ty as crate::parse::kvp::FromKVPSection>::from_kvp_section(section);
                parsed.#ident.push(section_ty);
                // All section warnings need to be pushed onto the end of the file warnings
                warnings.extend(section_warnings);
            }},
            FieldKind::Hash => quote! {{
                let (section_ty, section_warnings) = <#ty as crate::parse::kvp::FromKVPSection>::from_kvp_section(section);
                parsed.#ident.insert(<u64 as std::str::FromStr>::from_str(&section_name[#non_bare_ident_len..].trim()).expect("Unable to parse section name id"), section_ty);
                // All section warnings need to be pushed onto the end of the file warnings
                warnings.extend(section_warnings);
            }},
            FieldKind::Normal => quote! {{
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

    let pretty_print = generate_pretty_print_impls(fields.iter());

    quote! (
        #[automatically_derived]
        #[allow(clippy::used_underscore_binding)]
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
        #[automatically_derived]
        #[allow(clippy::used_underscore_binding)]
        impl crate::parse::PrettyPrintResult for #ident {
            fn fmt(&self, indent: usize, out: &mut dyn ::std::io::Write) -> ::std::io::Result<()> {
                writeln!(out, #ident_str_colon)?;
                let indent = indent + 1;
                #pretty_print
                Ok(())
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

    let exists_bare_and_vec = fields.iter().filter(|f| f.bare && f.kind == FieldKind::Vec).count() >= 1;
    let more_than_one_bare_field = fields.iter().filter(|f| f.bare).count() > 1;
    if exists_bare_and_vec && more_than_one_bare_field {
        panic!("If there are any fields that are bare and of type Vec<T>, there must be only one bare field");
    }

    let exists_bare_and_hash = fields.iter().filter(|f| f.bare && f.kind == FieldKind::Hash).count() >= 1;
    if exists_bare_and_hash {
        panic!("Hash fields cannot be bare");
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
                let ts = if field.kind == FieldKind::Vec {
                    quote! {crate::parse::kvp::ValueData::Value{ value }}
                } else if field.kind == FieldKind::Normal {
                    quote! {crate::parse::kvp::ValueData::Value{ value } if bare_counter == #bare_field_counter}
                } else {
                    unreachable!("Only regular and Vec fields are allowed to be bare");
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
                if field.kind == FieldKind::Hash {
                    quote! {
                        crate::parse::kvp::ValueData::KeyValuePair{ key, value }
                    }
                } else {
                    quote! {
                        crate::parse::kvp::ValueData::KeyValuePair{ key: #lower, value }
                    }
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
        let operation = match field.kind {
            FieldKind::Vec => quote! {{
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
            FieldKind::Hash => quote! {{
                let key_optional = <u64 as crate::parse::kvp::FromKVPValue>::from_kvp_value(key);
                let value_optional = <#ty as crate::parse::kvp::FromKVPValue>::from_kvp_value(value);

                if key_optional.is_some() && value_optional.is_some() {
                    parsed.#ident.insert(key_optional.unwrap(), value_optional.unwrap());
                } else {
                    if !key_optional.is_some() {
                        warnings.push(crate::parse::kvp::KVPGenericWarning{
                            span: field.span,
                            kind: crate::parse::kvp::KVPGenericWarningKind::UnknownField {
                                name: String::from(key),
                            }
                        })
                    }
                    if !value_optional.is_some() {
                        warnings.push(crate::parse::kvp::KVPGenericWarning{
                            span: field.span,
                            kind: crate::parse::kvp::KVPGenericWarningKind::InvalidValue {
                                value: String::from(value),
                            }
                        })
                    }
                }

                #bare_operation
            }},
            FieldKind::Normal => quote! {{
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

    let pretty_print = generate_pretty_print_impls(fields.iter());

    quote! (
        #[automatically_derived]
        #[allow(clippy::used_underscore_binding)]
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
                            kind: crate::parse::kvp::KVPGenericWarningKind::TooManyFields {
                                idx: bare_counter,
                                max: #bare_field_counter,
                            }
                        }),
                    }
                }

                (parsed, warnings)
            }
        }
        #[automatically_derived]
        #[allow(clippy::used_underscore_binding)]
        impl crate::parse::PrettyPrintResult for #ident {
            fn fmt(&self, indent: usize, out: &mut dyn ::std::io::Write) -> ::std::io::Result<()> {
                writeln!(out)?;
                #pretty_print
                Ok(())
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

    let conversion = combine_token_streams(fields.iter().map(|field| {
        let ident = field.ident.clone().expect("Fields must have names");
        let ty = field.ty.clone();

        quote! {
            #ident: <#ty as crate::parse::kvp::FromKVPValue>::from_kvp_value(iterator.next()?)?,
        }
    }));

    let pretty_print = generate_pretty_print_impls(fields.iter());

    quote! (
        #[automatically_derived]
        #[allow(clippy::used_underscore_binding)]
        impl crate::parse::kvp::FromKVPValue for #ident {
            fn from_kvp_value(value: &str) -> Option<Self> {
                let mut iterator = value.split(',').map(str::trim);

                Some(#ident {
                    #conversion
                })
            }
        }
        #[automatically_derived]
        #[allow(clippy::used_underscore_binding)]
        impl crate::parse::PrettyPrintResult for #ident {
            fn fmt(&self, indent: usize, out: &mut dyn ::std::io::Write) -> ::std::io::Result<()> {
                writeln!(out)?;
                #pretty_print
                Ok(())
            }
        }
    )
    .into()
}

/// Enum variant with possible attributes as used by [`kvp_enum_numbers`]
#[derive(Debug, FromVariant)]
#[darling(attributes(kvp))]
struct Variant {
    ident: Ident,
    #[darling(default)]
    default: bool,
    #[darling(map = "split_aliases", default)]
    alias: Vec<String>,
    #[darling(default)]
    index: Option<i64>,
}

fn generate_pretty_print_impls_variant<'a>(iter: impl IntoIterator<Item = &'a Variant> + 'a) -> TokenStream2 {
    combine_token_streams(iter.into_iter().map(|variant| {
        let ident = variant.ident.clone();
        let ident_str = ident.to_string();

        quote! {
            Self::#ident => writeln!(out, #ident_str),
        }
    }))
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

    let per_field_tokens: Vec<_> = fields
        .iter()
        .map(|variant| {
            let ident = variant.ident.clone();

            if let Some(replacement_idx) = variant.index {
                idx = replacement_idx;
            }

            let new_idx = idx + 1;
            let idx = std::mem::replace(&mut idx, new_idx);

            (
                quote! {
                    #idx => Self::#ident,
                },
                if variant.alias.is_empty() {
                    TokenStream2::new()
                } else {
                    let conditions =
                        combine_token_streams(variant.alias.iter().map(|s| quote! { #s }).intersperse(quote! {|}));
                    quote! {
                        #conditions => #idx,
                    }
                },
            )
        })
        .collect();
    let matches = combine_token_streams(per_field_tokens.iter().map(|(m, _)| m.clone()));
    let recovery = combine_token_streams(per_field_tokens.into_iter().map(|(_, r)| r));

    let pretty_print = generate_pretty_print_impls_variant(fields.iter());

    quote! (
        #[automatically_derived]
        #[allow(clippy::used_underscore_binding, unreachable_code)]
        impl crate::parse::kvp::FromKVPValue for #ident {
            fn from_kvp_value(value: &str) -> Option<Self> {
                let number = <i64 as crate::parse::kvp::FromKVPValue>::from_kvp_value(value).or_else(|| Some(match value {
                    #recovery
                    _ => return None,
                }))?;

                Some(match number {
                    #matches
                    _ => return None,
                })
            }
        }

        #default_impl


        #[automatically_derived]
        #[allow(clippy::used_underscore_binding)]
        impl crate::parse::PrettyPrintResult for #ident {
            fn fmt(&self, indent: usize, out: &mut dyn ::std::io::Write) -> ::std::io::Result<()> {
                match self {
                    #pretty_print
                }?;
                Ok(())
            }
        }
    )
    .into()
}
