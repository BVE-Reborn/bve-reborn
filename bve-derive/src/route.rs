use crate::helpers::combine_token_streams;
use darling::FromField;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use syn::{Expr, ItemStruct, Type};

#[derive(Debug, FromField)]
#[darling(attributes(command))]
#[allow(clippy::struct_excessive_bools)]
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
        let defaulted = f.default.map_or_else(|| quote::quote! {?}, |string| {
            let expr: Expr = syn::parse_str(&string).expect("Could not parse default expression");

            quote::quote! {
                .unwrap_or_else(|_| #expr)
            }
        });
        if index {
            assert!(!f.variadic, "Fields can't be indices and variadic");
            assert!(!f.optional, "Fields can't be indices and optional");
            let idx = index_count;
            index_count += 1;
            let len = index_count;
            quote::quote! {
                #ident: {
                    let value: Result<#ty, crate::parse::route::errors::CommandCreationError> = try {
                        if command.indices.len() >= #len {
                            let index = command.indices[#idx].ok_or_else(|| crate::parse::route::errors::CommandCreationError::MissingIndex { command: command.to_string(), index: #idx })?;
                            ::std::convert::TryFrom::try_from(index).map_err(|_| crate::parse::route::errors::CommandCreationError::InvalidIndex { command: command.to_string(), index: #idx })?
                        } else {
                            Err(crate::parse::route::errors::CommandCreationError::MissingIndex { command: command.to_string(), index: #idx })?
                        }
                    };
                    value #defaulted
                },
            }
        } else if f.suffix {
            quote::quote! {
                #ident: {
                    let suffix = command.suffix.as_ref().ok_or_else(|| crate::parse::route::errors::CommandCreationError::MissingSuffix { command: command.to_string() })?;
                    ::std::str::FromStr::from_str(suffix).map_err(|_| crate::parse::route::errors::CommandCreationError::InvalidSuffix { command: command.to_string() })?
                },
            }
        } else if f.ignore {
            quote::quote! {
                #ident: ::std::default::Default::default(),
            }
        } else if f.variadic {
            quote::quote! {
                #ident: crate::parse::route::ir::FromVariadicRouteArgument::from_variadic_route_argument(&command) #defaulted,
            }
        } else {
            let idx = argument_count;
            argument_count += 1;
            let len = argument_count;
            if optional {
                quote::quote! {
                    #ident: {
                        let value: #ty = {
                            if command.indices.len() >= #len {
                                ::std::str::FromStr::from_str(&command.arguments[#idx]).ok()
                            } else {
                                None
                            }
                        };
                        value
                    },
                }
            }
            else {
                quote::quote! {
                    #ident: {
                        let value: Result<#ty, crate::parse::route::errors::CommandCreationError> = try {
                            if command.arguments.len() >= #len {
                                ::std::str::FromStr::from_str(&command.arguments[#idx]).map_err(|_| crate::parse::route::errors::CommandCreationError::InvalidArgument { command: command.to_string(), index: #idx })?
                            } else {
                                Err(crate::parse::route::errors::CommandCreationError::MissingArgument { command: command.to_string(), index: #idx })?
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
            fn from_route_command(command: crate::parse::route::parser::Command) -> Result<Self, crate::parse::route::errors::CommandCreationError>
            where
                Self: Sized
            {
                Ok(Self {
                    #members
                })
            }
        }
    };
    output.into()
}
