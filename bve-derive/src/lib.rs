// Rust warnings
#![warn(unused)]
#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![forbid(unsafe_code)]
#![allow(unused_extern_crates)] // cargo < 1.42 doesn't add `--extern proc_macro`, so allow an duplicate declaration
// Rustdoc Warnings
#![deny(intra_doc_link_resolution_failure)]
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
#![allow(clippy::too_many_lines)] // This is also dumb
// Annoying/irrelevant clippy Restrictions
#![allow(clippy::as_conversions)]
#![allow(clippy::decimal_literal_representation)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::float_cmp_const)]
#![allow(clippy::implicit_return)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::indexing_slicing)] // Proc macros are error prone
#![allow(clippy::integer_arithmetic)]
#![allow(clippy::integer_division)]
#![allow(clippy::let_underscore_must_use)]
#![allow(clippy::match_bool)] // prettier
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::multiple_crate_versions)] // Cargo deny's job
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::option_expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::result_expect_used)]
#![allow(clippy::shadow_reuse)]
#![allow(clippy::shadow_same)]
#![allow(clippy::string_add)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::unimplemented)] // Specific use case proc macro, lots of holes
#![allow(clippy::unreachable)]
#![allow(clippy::wildcard_enum_match_arm)]
// CLion is having a fit about panic not existing
#![feature(core_panic)]

extern crate proc_macro;

use proc_macro::TokenStream;

// Code coverage is silly on these derives
#[cfg_attr(tarpaulin, skip)]
mod serde_proxy;

#[cfg_attr(tarpaulin, skip)]
mod c_interface;

#[cfg_attr(tarpaulin, skip)]
mod span;

#[cfg_attr(tarpaulin, skip)]
mod helpers;

#[cfg_attr(tarpaulin, skip)]
mod test;

#[cfg_attr(tarpaulin, skip)]
mod kvp;

#[proc_macro_attribute]
#[cfg_attr(tarpaulin, skip)]
pub fn serde_proxy(_attr: TokenStream, item: TokenStream) -> TokenStream {
    serde_proxy::serde_proxy(item)
}

#[proc_macro_attribute]
#[cfg_attr(tarpaulin, skip)]
pub fn serde_vector_proxy(_attr: TokenStream, item: TokenStream) -> TokenStream {
    serde_proxy::serde_vector_proxy(item)
}

#[proc_macro_attribute]
#[cfg_attr(tarpaulin, skip)]
pub fn c_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    c_interface::c_interface(&attr, item)
}

#[proc_macro_attribute]
#[cfg_attr(tarpaulin, skip)]
pub fn bve_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    test::test(item)
}

#[proc_macro_attribute]
#[cfg_attr(tarpaulin, skip)]
pub fn span(attr: TokenStream, item: TokenStream) -> TokenStream {
    span::span(attr, item)
}

#[proc_macro_derive(FromKVPFile, attributes(kvp))]
#[cfg_attr(tarpaulin, skip)]
pub fn from_kvp_file(item: TokenStream) -> TokenStream {
    kvp::kvp_file(item)
}

#[proc_macro_derive(FromKVPSection, attributes(kvp))]
#[cfg_attr(tarpaulin, skip)]
pub fn from_kvp_section(item: TokenStream) -> TokenStream {
    kvp::kvp_section(item)
}

#[proc_macro_derive(FromKVPValue, attributes(kvp))]
#[cfg_attr(tarpaulin, skip)]
pub fn from_kvp_value(item: TokenStream) -> TokenStream {
    kvp::kvp_value(item)
}

#[proc_macro_derive(FromKVPValueEnumNumbers, attributes(kvp))]
#[cfg_attr(tarpaulin, skip)]
pub fn from_kvp_value_enum_numbers(item: TokenStream) -> TokenStream {
    kvp::kvp_enum_numbers(item)
}
