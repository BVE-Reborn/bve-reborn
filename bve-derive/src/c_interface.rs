use crate::helpers::combine_token_streams;
use proc_macro::TokenStream;
use syn::export::TokenStream2;

pub fn c_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = syn::parse_macro_input!(item as syn::ItemFn);

    let mangle = if attr.to_string() != "mangle" {
        quote::quote!(#[no_mangle])
    } else {
        TokenStream2::new()
    };

    let block = &*function.block;
    let attrs = combine_token_streams(function.attrs.iter().map(quote::ToTokens::to_token_stream));
    let sig = &function.sig;
    let vis = &function.vis;

    let result = quote::quote! {
        #mangle
        #attrs
        #vis #sig {
            let result = std::panic::catch_unwind(::std::panic::AssertUnwindSafe(move || #block));
            match result {
                Ok(r) => r,
                Err(_) => {
                    std::process::abort()
                }
            }
        }
    };
    result.into()
}
