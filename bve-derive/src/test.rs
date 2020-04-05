use crate::helpers::combine_token_streams;
use proc_macro::TokenStream;

pub fn test(item: TokenStream) -> TokenStream {
    let function = syn::parse_macro_input!(item as syn::ItemFn);

    let block = &*function.block;
    let attrs = combine_token_streams(function.attrs.iter().map(quote::ToTokens::to_token_stream));
    let sig = &function.sig;
    let vis = &function.vis;

    let result = quote::quote! {
        #attrs
        #vis #sig {
            // clion causes the logger issues, disable it when in the test harness. Output isn't seen anyway.
            let in_clion = !std::env::args().any(|v| v == "--nocapture");
            if !in_clion {
                let _ = ::fern::Dispatch::new().format(crate::log::log_formatter).chain(::std::io::stderr()).apply();
            }
            // We don't care if it succeeded or not
            #block
        }
    };
    result.into()
}
