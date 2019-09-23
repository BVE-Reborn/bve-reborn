//! Implementation of the high performance runtime logic for the game BVE-Reborn.

// Features
#![feature(const_vec_new)]
// Rust warnings
#![warn(unused)]
#![deny(nonstandard_style)]
#![deny(future_incompatible)]
#![deny(rust_2018_idioms)]
#![forbid(unsafe_code)]
// Clippy warnings
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::restriction)]
// Annoying regular clippy warnings
#![allow(clippy::cast_sign_loss)] // Annoying
#![allow(clippy::cast_precision_loss)] // Annoying
#![allow(clippy::cast_possible_truncation)] // Annoying
#![allow(clippy::cognitive_complexity)] // This is dumb
// Annoying/irrelevant clippy Restrictions
#![allow(clippy::decimal_literal_representation)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::float_cmp_const)]
#![allow(clippy::implicit_return)]
#![allow(clippy::integer_arithmetic)]
#![allow(clippy::integer_division)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::shadow_reuse)]
#![allow(clippy::shadow_same)]
#![allow(clippy::wildcard_enum_match_arm)]

pub use datatypes::*;

mod datatypes;
pub mod parse;
