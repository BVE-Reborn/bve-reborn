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
            ::tracing::dispatcher::with_default(
                &::tracing::dispatcher::Dispatch::new(
                    ::tracing_subscriber::FmtSubscriber::builder()
                        .with_writer(|| std::io::stdout())
                        .with_max_level(tracing_subscriber::filter::LevelFilter::TRACE)
                        .finish(),
                ),
                || {
                    #block;
                },
            );
        }
    };
    result.into()
}
