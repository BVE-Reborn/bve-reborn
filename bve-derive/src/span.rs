use crate::helpers::combine_token_streams;
use syn::export::{TokenStream, TokenStream2};

pub fn span(attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = syn::parse_macro_input!(item as syn::ItemFn);
    let attr = TokenStream2::from(attr);

    let block = &*function.block;
    let attrs = combine_token_streams(function.attrs.iter().map(quote::ToTokens::to_token_stream));
    let sig = &function.sig;
    let vis = &function.vis;

    let result = quote::quote! {
        #attrs
        #vis #sig {
            let span = ::tracing::span!(::tracing::Level:: #attr);
            let _guard = span.enter();
            #block
        }
    };
    result.into()
}
