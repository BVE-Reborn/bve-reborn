use itertools::Itertools;
use syn::export::TokenStream2;

pub fn combine_token_streams<I: IntoIterator<Item = TokenStream2>>(streams: I) -> TokenStream2 {
    streams
        .into_iter()
        .fold1(|mut l, r| {
            l.extend(r);
            l
        })
        .unwrap_or_else(TokenStream2::new)
}
