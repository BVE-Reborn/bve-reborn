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
            let subscriber = crate::log::Subscriber::new(std::io::stdout());
            ::tracing::dispatcher::with_default(&::tracing::dispatcher::Dispatch::new(subscriber),
                || {
                    #block;
                }
            );
            subscriber.terminate();
        }
    };
    result.into()
}
