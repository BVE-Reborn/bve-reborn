use crate::helpers::combine_token_streams;
use proc_macro::TokenStream;

pub fn test(item: TokenStream) -> TokenStream {
    let function = syn::parse_macro_input!(item as syn::ItemFn);

    let block = &*function.block;
    let attrs = combine_token_streams(function.attrs.iter().map(quote::ToTokens::to_token_stream));
    let sig = &function.sig;
    let vis = &function.vis;

    let result = quote::quote! {
        #[test]
        #attrs
        #vis #sig {
            let task = || {
                #block;
            };
            // TODO: Actually fix these clion issues
            let in_clion = !std::env::args().any(|v| v == "--nocapture"); // clion causes the logger issues, disable it when in the test harness. Output isn't seen anyway.
            if in_clion {
                task();
            } else {
                let subscriber = crate::log::Subscriber::new(::std::io::stderr(), crate::log::SerializationMethod::JsonPretty);
                ::tracing::dispatcher::with_default(&::tracing::dispatcher::Dispatch::new(subscriber.clone()),
                    task
                );
                ::std::mem::drop(subscriber);
            }
        }
    };
    result.into()
}
