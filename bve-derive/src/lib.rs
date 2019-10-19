// Rust warnings
#![warn(unused)]
#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![forbid(unsafe_code)]
// Clippy warnings
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::restriction)]
// Annoying regular clippy warnings
#![allow(clippy::cast_possible_truncation)] // Annoying
#![allow(clippy::cast_precision_loss)] // Annoying
#![allow(clippy::cast_sign_loss)] // Annoying
#![allow(clippy::cognitive_complexity)] // This is dumb
#![allow(clippy::single_match_else)] // Future expansion
// Annoying/irrelevant clippy Restrictions
#![allow(clippy::decimal_literal_representation)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::float_cmp_const)]
#![allow(clippy::implicit_return)]
#![allow(clippy::indexing_slicing)] // Proc macros are error prone
#![allow(clippy::integer_arithmetic)]
#![allow(clippy::integer_division)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::shadow_reuse)]
#![allow(clippy::shadow_same)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::wildcard_enum_match_arm)]

extern crate proc_macro;

use proc_macro::TokenStream;

mod serde_proxy;

#[proc_macro_attribute]
pub fn serde_proxy(_attr: TokenStream, item: TokenStream) -> TokenStream {
    serde_proxy::serde_proxy(item)
}

#[proc_macro_attribute]
pub fn serde_vector_proxy(_attr: TokenStream, item: TokenStream) -> TokenStream {
    serde_proxy::serde_vector_proxy(item)
}
