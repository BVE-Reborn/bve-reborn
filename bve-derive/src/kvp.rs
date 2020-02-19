use crate::helpers::combine_token_streams;
use darling::FromField;
use proc_macro2::Ident;
use quote::ToTokens;
use syn::export::TokenStream;
use syn::{GenericArgument, ItemStruct, PathArguments, Type};

fn split_aliases(input: String) -> Vec<String> {
    if input.is_empty() {
        vec![]
    } else {
        input.split(";").map(str::trim).map(String::from).collect()
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
    #[darling(map = "split_aliases", default)]
    alias: Vec<String>,
}

fn parse_fields(item: ItemStruct) -> Vec<Field> {
    let fields = item.fields.iter().flat_map(Field::from_field);
    let fields: Vec<Field> = fields
        .map(|mut field: Field| {
            match &field.ty {
                Type::Path(path) => {
                    if path.path.segments.last().unwrap().ident.to_string() == "Vec" {
                        field.vec = true;
                        match &path.path.segments.last().unwrap().arguments {
                            PathArguments::AngleBracketed(angled) => match angled.args.first().unwrap() {
                                GenericArgument::Type(ty) => field.ty = ty.clone(),
                                _ => unimplemented!(),
                            },
                            _ => unimplemented!(),
                        }
                    }
                }
                _ => {}
            };
            field
        })
        .collect();

    assert!(
        fields.iter().filter(|f| f.bare).count() <= 1,
        "Cannot have more than 1 bare field"
    );

    fields
}

pub fn kvp_file(item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as ItemStruct);

    let fields = parse_fields(item);

    TokenStream::new()
}
