use crate::helpers::combine_token_streams;
use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::{Group, Ident, Literal};
use quote::{format_ident, quote};
use syn::{
    export::{ToTokens, TokenStream2},
    Attribute, ExprPath, GenericArgument, PathArguments, Type, TypePath, Visibility,
};

struct PrettyPrintField {
    name: Ident,
    ty: TypePath,
}

#[derive(Debug, Clone)]
struct Field {
    attributes: Vec<Attribute>,
    default: Option<Literal>,
    visibility: Visibility,
    name: Ident,
    ty: (Vec<String>, TypePath),
}

impl From<Field> for PrettyPrintField {
    fn from(f: Field) -> Self {
        Self {
            name: f.name,
            ty: f.ty.1,
        }
    }
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

/// Replace paths like `f32` with their proxy like `LooseFloat<f32>`
fn process_explicit_type_proxy_path(t: &TypePath) -> TokenStream2 {
    let t = t.path.segments.last().expect("Type path must exist");
    let ident = &t.ident;
    let nested = &t.arguments;
    match nested {
        PathArguments::None => match t.ident.to_string().as_str() {
            "bool" => quote!(crate::parse::util::LooseNumericBool),
            "f32" => quote!(crate::parse::util::LooseNumber<f32>),
            "f64" => quote!(crate::parse::util::LooseNumber<f64>),
            "i8" => quote!(crate::parse::util::LooseNumber<i8>),
            "i16" => quote!(crate::parse::util::LooseNumber<i16>),
            "i32" => quote!(crate::parse::util::LooseNumber<i32>),
            "i64" => quote!(crate::parse::util::LooseNumber<i64>),
            "u8" => quote!(crate::parse::util::LooseNumber<u8>),
            "u16" => quote!(crate::parse::util::LooseNumber<u16>),
            "u32" => quote!(crate::parse::util::LooseNumber<u32>),
            "u64" => quote!(crate::parse::util::LooseNumber<u64>),
            "isize" => quote!(crate::parse::util::LooseNumber<isize>),
            "usize" => quote!(crate::parse::util::LooseNumber<usize>),
            _ => quote!(#t),
        },
        PathArguments::AngleBracketed(b) => {
            let generic = b.args.first().expect("Must have generic type");
            let inner = if let GenericArgument::Type(t) = generic {
                if let Type::Path(p) = t {
                    let inner = process_explicit_type_proxy_path(p);
                    match ident.to_string().as_str() {
                        "Vec" => quote!(Option<#inner>), // this is only for serde_vector_proxy
                        _ => inner,
                    }
                } else {
                    panic!("Expected type path");
                }
            } else {
                panic!("Expected type path");
            };

            quote!(#ident<#inner>)
        }
        _ => panic!("Unexpected PathArguments"),
    }
}

/// Replace paths like `f32` with their proxy like `LooseFloat<f32>`
fn process_explicit_type_proxy(t: Type) -> TokenStream2 {
    if let Type::Path(path) = t {
        process_explicit_type_proxy_path(&path)
    } else {
        quote!(#t)
    }
}

/// Returns the first generic argument of the provided type, `name_vec` provides already stringified type names
fn get_first_generic_argument(name_vec: &[String], path: &TypePath) -> TokenStream2 {
    let last_name = name_vec.last().map(String::as_str);
    match last_name {
        Some("BVec2") | Some("BVec3") | Some("BVec4") => quote!(crate::parse::util::LooseNumericBool),
        Some("ColorU8RGB") | Some("ColorU8RGBA") => quote!(crate::parse::util::LooseNumber<u8>),
        Some("UVec2") | Some("UVec3") | Some("UVec4") => quote!(crate::parse::util::LooseNumber<u32>),
        Some("IVec2") | Some("IVec3") | Some("IVec4") => quote!(crate::parse::util::LooseNumber<i32>),
        Some("ColorF32R") | Some("ColorF32RG") | Some("ColorF32RGB") | Some("ColorF32RGBA") | Some("Vec2")
        | Some("Vec3") | Some("Vec4") => quote!(crate::parse::util::LooseNumber<f32>),
        name => match &path.path.segments.last().expect("Type path must exist.").arguments {
            PathArguments::AngleBracketed(arg) => match &arg.args[0] {
                GenericArgument::Type(t) => process_explicit_type_proxy((*t).clone()),
                _ => panic!("{:?} generic argument must be a type", name),
            },
            _ => panic!("{:?} must have generic arguments", name),
        },
    }
}

/// Given a type, find the conversion to reverse the proxying. From `LooseFloat` -> `f32` for example
fn process_type_proxy_conversion(inner_type: TokenStream2) -> TokenStream2 {
    let parsed = syn::parse2::<syn::TypePath>(inner_type).expect("Expected a type, found some other crap");
    let t = parsed.path.segments.last().expect("Type must have segments");
    let ident = t.ident.to_string();
    let nested = &t.arguments;

    match nested {
        PathArguments::None => match ident.as_str() {
            "LooseNumericBool" => quote!(.0),
            _ => quote!(),
        },
        PathArguments::AngleBracketed(b) => {
            let generic = b.args.first().expect("Must have generic type");
            let inner = if let GenericArgument::Type(t) = generic {
                if let Type::Path(p) = t {
                    process_type_proxy_conversion(p.into_token_stream())
                } else {
                    panic!("Expected type path");
                }
            } else {
                panic!("Expected type path");
            };

            match ident.as_str() {
                "LooseNumber" => quote!(.0 #inner),
                "Vec" => quote!(.into_iter().filter_map(std::convert::identity).map(|v| v #inner).collect()),
                "Option" => quote!(#inner),
                _ => quote!(),
            }
        }
        _ => panic!("Unexpected PathArguments"),
    }
}

/// Process the default operation. This takes the member name of the proxy struct, the resulting type, and the
/// expression that will give you a default. It then outputs the proxy type, the conversion back to the normal type, and
/// the remaining attributes.
fn process_default(
    new_name: &Ident,
    inner_type: &TokenStream2,
    default_expr: &Option<Literal>,
) -> (TokenStream2, TokenStream2, TokenStream2) {
    let inner_conversion = process_type_proxy_conversion(inner_type.clone());

    match default_expr {
        Some(s) => {
            let l_parsed =
                syn::parse2::<syn::LitStr>(s.to_token_stream()).expect("Argument to default must be string literal.");
            let default_call: ExprPath = l_parsed.parse().expect("Argument to default must be a valid ExprPath.");

            let new_type = quote!(Option<#inner_type>);
            let conversion = quote!(match proxy.#new_name {
                Some(v) => v #inner_conversion,
                None => #default_call().unwrap() #inner_conversion,
            });
            let attribute = quote!(#[serde(default = #s)]);
            (new_type, conversion, attribute)
        }
        None => {
            let new_type = quote!(#inner_type);
            let conversion = quote!(proxy.#new_name #inner_conversion);
            let attribute = TokenStream2::new();
            (new_type, conversion, attribute)
        }
    }
}

fn generate_proxy_object(name: &Ident, fields: &[Field]) -> TokenStream2 {
    let new_data = fields
        .iter()
        .map(|field| match &field.ty.0 {
            vec if match vec.last().map(String::as_str) {
                Some("Vec2") | Some("BVec2") | Some("UVec2") | Some("IVec2") => true,
                _ => false,
            } =>
            {
                let inner_type = get_first_generic_argument(vec, &field.ty.1);
                let original_name = &field.name;
                let x_new = format_ident!("{}_x", original_name);
                let y_new = format_ident!("{}_y", original_name);
                let attributes = combine_attributes(&field.attributes);
                let (x_inner, x_conversion, x_attribute) = process_default(&x_new, &inner_type, &field.default);
                let (y_inner, y_conversion, y_attribute) = process_default(&y_new, &inner_type, &field.default);
                let proxy_fields = quote! {
                    #attributes
                    #x_attribute
                    #x_new: #x_inner,

                    #attributes
                    #y_attribute
                    #y_new: #y_inner,
                };
                let containing_type = &field.ty.1;
                let from_fields = quote! {
                    #original_name: #containing_type::new(#x_conversion, #y_conversion),
                };
                (proxy_fields, from_fields)
            }
            vec if match vec.last().map(String::as_str) {
                Some("ColorU8RGB") | Some("Vec3") | Some("BVec3") | Some("UVec3") | Some("IVec3") => true,
                _ => false,
            } =>
            {
                let original_name = &field.name;
                let inner_type = get_first_generic_argument(vec, &field.ty.1);
                let x_new = format_ident!("{}_x", original_name);
                let y_new = format_ident!("{}_y", original_name);
                let z_new = format_ident!("{}_z", original_name);
                let attributes = combine_attributes(&field.attributes);
                let (x_inner, x_conversion, x_attribute) = process_default(&x_new, &inner_type, &field.default);
                let (y_inner, y_conversion, y_attribute) = process_default(&y_new, &inner_type, &field.default);
                let (z_inner, z_conversion, z_attribute) = process_default(&z_new, &inner_type, &field.default);
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
                let containing_type = &field.ty.1;
                let from_fields = quote! {
                    #original_name: #containing_type::new(#x_conversion, #y_conversion, #z_conversion),
                };
                (proxy_fields, from_fields)
            }
            vec if match vec.last().map(String::as_str) {
                Some("ColorU8RGBA") | Some("Vec4") | Some("BVec4") | Some("UVec4") | Some("IVec4") => true,
                _ => false,
            } =>
            {
                let original_name = &field.name;
                let inner_type = get_first_generic_argument(vec, &field.ty.1);
                let x_new = format_ident!("{}_x", original_name);
                let y_new = format_ident!("{}_y", original_name);
                let z_new = format_ident!("{}_z", original_name);
                let w_new = format_ident!("{}_w", original_name);
                let attributes = combine_attributes(&field.attributes);
                let (x_inner, x_conversion, x_attribute) = process_default(&x_new, &inner_type, &field.default);
                let (y_inner, y_conversion, y_attribute) = process_default(&y_new, &inner_type, &field.default);
                let (z_inner, z_conversion, z_attribute) = process_default(&z_new, &inner_type, &field.default);
                let (w_inner, w_conversion, w_attribute) = process_default(&w_new, &inner_type, &field.default);
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
                let containing_type = &field.ty.1;
                let from_fields = quote! {
                    #original_name: #containing_type::new(#x_conversion, #y_conversion, #z_conversion, #w_conversion),
                };
                (proxy_fields, from_fields)
            }
            _ => {
                let original_name = &field.name;
                let attributes = combine_attributes(&field.attributes);
                let ty = &field.ty.1;
                let ty = process_explicit_type_proxy_path(ty);
                let (inner, conversion, attribute) = process_default(original_name, &ty, &field.default);
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

        #[automatically_derived]
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
    let mut default = None;
    let mut default_buffer = Vec::new();
    for attr in attributes.drain(0..) {
        match attr.path.segments.first().map(|s| s.ident.to_string()).as_deref() {
            Some("default") => {
                default = Some(
                    syn::parse2::<Literal>(
                        syn::parse2::<Group>(attr.tokens)
                            .expect("expected group in default")
                            .stream(),
                    )
                    .expect("expected string in default"),
                )
            }
            _ => default_buffer.push(attr),
        }
    }
    (default, default_buffer)
}

fn find_primary_attribute(mut attributes: Vec<Attribute>) -> (bool, Vec<Attribute>) {
    let mut found = false;
    let mut attr_buffer = Vec::new();
    for attr in attributes.drain(0..) {
        match attr.path.segments.first().map(|s| s.ident.to_string()).as_deref() {
            Some("primary") => found = true,
            _ => attr_buffer.push(attr),
        }
    }
    (found, attr_buffer)
}

fn generate_pretty_print(ident: &Ident, fields: &[impl Into<PrettyPrintField> + Clone]) -> TokenStream2 {
    let members = combine_token_streams(fields.iter().map(|f| {
        let f: PrettyPrintField = f.clone().into();
        let name = f.name;
        let name_str = name.to_string() + ": ";
        let ty = f.ty;
        quote! {
            crate::parse::util::indent(indent, out)?;
            write!(out, #name_str)?;
            <#ty as crate::parse::PrettyPrintResult>::fmt(&self.#name, indent + 1, out)?;
        }
    }));

    let ident_str = ident.to_string();

    quote! {
        impl crate::parse::PrettyPrintResult for #ident {
            fn fmt(&self, indent: usize, out: &mut dyn std::io::Write) -> std::io::Result<()> {
                writeln!(out, #ident_str)?;

                #members

                Ok(())
            }
        }
    }
}

pub fn serde_proxy(item: TokenStream) -> TokenStream {
    let mut parsed = syn::parse_macro_input!(item as syn::ItemStruct);

    let mut fields = Vec::new();

    for field in &mut parsed.fields {
        let attributes = field.attrs.clone();
        let (default, attributes) = find_default_attribute(attributes);
        field.attrs = attributes.clone();
        let visibility = field.vis.clone();
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
            default,
            visibility,
            name,
            ty,
        })
    }

    let proxy = generate_proxy_object(&parsed.ident, &fields);
    let proxy_name = format!("{}SerdeProxy", &parsed.ident);
    let pretty_print = generate_pretty_print(&parsed.ident, &fields);

    let current = quote!(
        #proxy

        #[derive(Debug, Clone, PartialEq, Deserialize)]
        #[serde(from = #proxy_name)]
        #parsed

        #pretty_print
    );

    current.into()
}

#[derive(Debug, Clone)]
struct VectorProxyField {
    name: Ident,
    ty: Type,
    conversion: TokenStream2,
}

#[allow(clippy::fallible_impl_from)]
impl From<VectorProxyField> for PrettyPrintField {
    fn from(f: VectorProxyField) -> Self {
        Self {
            name: f.name,
            ty: match f.ty {
                Type::Path(p) => p,
                _ => panic!("Why is is anything other than a Type::Path?"),
            },
        }
    }
}

pub fn serde_vector_proxy(item: TokenStream) -> TokenStream {
    let mut parsed = syn::parse_macro_input!(item as syn::ItemStruct);

    let name = &parsed.ident;

    let mut primary_type = None;
    let mut parsed_fields = Vec::new();

    for field in &mut parsed.fields {
        let (primary, attributes) = find_primary_attribute(field.attrs.clone());
        let (default, attributes) = find_default_attribute(attributes);
        field.attrs = attributes;

        let name = field.ident.as_ref().expect("Must have ident").clone();

        parsed_fields.push(if primary {
            let proxy_type = process_explicit_type_proxy(field.ty.clone());
            let proxy_type = quote!(#proxy_type);
            primary_type = Some(proxy_type.clone());

            let conversion = process_type_proxy_conversion(proxy_type.clone());

            VectorProxyField {
                name: name.clone(),
                ty: field.ty.clone(),
                conversion: quote!(proxy #conversion),
            }
        } else if let Some(d) = default {
            VectorProxyField {
                name,
                ty: field.ty.clone(),
                conversion: quote!(#d ()),
            }
        } else {
            VectorProxyField {
                name,
                ty: field.ty.clone(),
                conversion: quote!(std::default::Default::default()),
            }
        });
    }

    let from_fields = combine_token_streams(parsed_fields.iter().map(|f| {
        let VectorProxyField { name, conversion, .. } = f;

        quote!(#name: #conversion,)
    }));

    let primary_type = primary_type.expect("Must be a type with the primary attribute");
    let primary_type_str = primary_type.to_string();
    let pretty_print = generate_pretty_print(&parsed.ident, &parsed_fields);

    let result = quote! {
        #[derive(Debug, Clone, PartialEq, Deserialize)]
        #[serde(from = #primary_type_str)]
        #parsed

        #[automatically_derived]
        impl ::std::convert::From<#primary_type> for #name {
            #[allow(clippy::default_trait_access)]
            fn from(proxy: #primary_type) -> Self {
                Self {
                    #from_fields
                }
            }
        }

        #pretty_print
    };

    result.into()
}
