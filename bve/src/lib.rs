//! Implementation of the high performance runtime logic for the game BVE-Reborn.

#![feature(async_closure)]
#![feature(test)]
#![feature(type_alias_impl_trait)]

#![allow(clippy::cognitive_complexity)]
#![allow(clippy::float_cmp)]
#![warn(unused)]
#![deny(nonstandard_style)]
#![deny(future_incompatible)]
#![deny(rust_2018_idioms)]
#![forbid(unsafe_code)]

pub use datatypes::*;

mod datatypes;
pub mod parse;
