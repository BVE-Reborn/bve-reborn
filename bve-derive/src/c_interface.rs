use crate::helpers::combine_token_streams;
use proc_macro::TokenStream;

pub fn c_interface(item: TokenStream) -> TokenStream {
    let function = syn::parse_macro_input!(item as syn::ItemFn);

    let block = &*function.block;
    let attrs = combine_token_streams(function.attrs.iter().map(quote::ToTokens::to_token_stream));
    let sig = &function.sig;
    let vis = &function.vis;

    let result = quote::quote! {
        #[no_mangle]
        #attrs
        #vis #sig {
            let result = std::panic::catch_unwind(|| #block);
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
