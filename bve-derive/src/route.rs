use crate::helpers::combine_token_streams;
use darling::FromField;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use syn::ItemStruct;

#[derive(Debug, FromField)]
#[darling(attributes(command))]
struct Field {
    ident: Option<Ident>,
    #[darling(default)]
    index: bool,
    #[darling(default)]
    default: bool,
    #[darling(default)]
    suffix: bool,
    #[darling(default)]
    variadic: bool,
}

pub fn from_route_command(stream: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(stream as ItemStruct);

    let fields = item.fields.iter().flat_map(Field::from_field);
    let ident = item.ident.clone();

    let mut index_count = 0_usize;
    let mut argument_count = 0_usize;
    let members = combine_token_streams(fields.map(|f: Field| {
        let ident = f.ident;
        let index = f.index;
        if index {
            assert!(!f.variadic, "Structs can't be indices and variadic");
            let idx = index_count;
            index_count += 1;
            let len = index_count;
            quote::quote! {
                #ident: if command.indices.len() >= #len {
                    ::std::convert::TryFrom::try_from(command.indices[#idx]).ok()?
                } else {
                    return None;
                },
            }
        } else if f.suffix {
            quote::quote! {
                #ident: ::std::str::FromStr::from_str(command.suffix?).ok()?,
            }
        } else if f.default {
            quote::quote! {
                #ident: ::std::default::Default::default(),
            }
        } else {
            if f.variadic {
                quote::quote! {
                    #ident: crate::parse::route::ir::FromVariadicRouteArgument::from_variadic_route_argument(&command.arguments).ok()?,
                }
            } else {
                let idx = argument_count;
                argument_count += 1;
                let len = argument_count;
                quote::quote! {
                    #ident: if command.arguments.len() >= #len {
                        ::std::str::FromStr::from_str(command.arguments[#idx]).ok()?
                    } else {
                        return None;
                    },
                }
            }
        }
    }));

    let output = quote::quote! {
        #[automatically_derived]
        impl crate::parse::route::ir::FromRouteCommand for #ident {
            fn from_route_command(command: crate::parse::route::parser::Command<'_>) -> Option<Self>
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
