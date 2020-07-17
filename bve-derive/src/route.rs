use crate::helpers::combine_token_streams;
use darling::FromField;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use syn::{export::TokenStream2, Expr, ItemStruct, Type};

#[derive(Debug, FromField)]
#[darling(attributes(command))]
struct Field {
    ident: Option<Ident>,
    ty: Type,
    #[darling(default)]
    index: bool,
    #[darling(default)]
    ignore: bool,
    #[darling(default)]
    suffix: bool,
    #[darling(default)]
    variadic: bool,
    #[darling(default)]
    optional: bool,
    #[darling(default)]
    default: Option<String>,
}

pub fn from_route_command(stream: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(stream as ItemStruct);

    let fields = item.fields.iter().flat_map(Field::from_field);
    let ident = item.ident.clone();

    let mut index_count = 0_usize;
    let mut argument_count = 0_usize;
    let members = combine_token_streams(fields.map(|f: Field| {
        let ident = f.ident;
        let ty = f.ty.clone();
        let index = f.index;
        let optional = f.optional;
        let optional_type = if f.optional {
            quote::quote! {#ty}
        } else {
            quote::quote! {Option<#ty>}
        };
        let defaulted = f.default.map_or_else(|| if optional {
            TokenStream2::new()
        } else {
            quote::quote! {?}
        }, |string| {
            let expr: Expr = syn::parse_str(&string).unwrap();

            quote::quote! {
                .unwrap_or_else(|| #expr)
            }
        });
        if index {
            assert!(!f.variadic, "Structs can't be indices and variadic");
            let idx = index_count;
            index_count += 1;
            let len = index_count;
            quote::quote! {
                #ident: {
                    let value: #optional_type  = try {
                        if command.indices.len() >= #len {
                            ::std::convert::TryFrom::try_from(command.indices[#idx]?).ok()?
                        } else {
                            None?
                        }
                    };
                    value #defaulted
                },
            }
        } else if f.suffix {
            quote::quote! {
                #ident: ::std::str::FromStr::from_str(&command.suffix?).ok()?,
            }
        } else if f.ignore {
            quote::quote! {
                #ident: ::std::default::Default::default(),
            }
        } else {
            if f.variadic {
                quote::quote! {
                    #ident: crate::parse::route::ir::FromVariadicRouteArgument::from_variadic_route_argument(&command.arguments).ok() #defaulted,
                }
            } else {
                let idx = argument_count;
                argument_count += 1;
                let len = argument_count;
                quote::quote! {
                    #ident: {
                        let value: #optional_type = try {
                            if command.indices.len() >= #len {
                                ::std::str::FromStr::from_str(&command.arguments[#idx]).ok()?
                            } else {
                                None?
                            }
                        };
                        value #defaulted
                    },
                }
            }
        }
    }));

    let output = quote::quote! {
        #[automatically_derived]
        impl crate::parse::route::ir::FromRouteCommand for #ident {
            fn from_route_command(command: crate::parse::route::parser::Command) -> Option<Self>
            where
                Self: Sized
            {
                Some(Self {
                    #members
                })
            }
        }
    };
    output.into()
}
