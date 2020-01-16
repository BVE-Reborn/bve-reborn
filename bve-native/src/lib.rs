//! C API for BVE-Reborn high performance libraries.

// Rust warnings
#![warn(unused)]
#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![allow(non_snake_case)] // Naming in FFI is weird
#![allow(non_camel_case_types)] // Naming in FFI is weird
#![allow(unsafe_code)] // We're doing FFI
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
#![allow(clippy::indexing_slicing)]
#![allow(clippy::integer_arithmetic)]
#![allow(clippy::integer_division)]
#![allow(clippy::mem_forget)] // Useful for FFI
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::option_expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::result_expect_used)]
#![allow(clippy::shadow_reuse)]
#![allow(clippy::shadow_same)]
#![allow(clippy::unreachable)]
#![allow(clippy::wildcard_enum_match_arm)]

macro_rules! bve_option {
    ($name:ident, $t:ty) => {
        #[repr(C)]
        pub struct $name {
            /// Actual value inside the option. Reading this is undefined behavior if exists is false.
            value: std::mem::MaybeUninit<$t>,
            /// Flag if the value exists or not.
            exists: bool,
        }
        impl From<Option<$t>> for $name {
            fn from(other: Option<$t>) -> Self {
                match other {
                    Some(v) => Self {
                        value: std::mem::MaybeUninit::new(v),
                        exists: true,
                    },
                    None => Self {
                        value: std::mem::MaybeUninit::uninit(),
                        exists: false,
                    },
                }
            }
        }
    };
}

macro_rules! bve_vector {
    ($name:ident, $t:ty) => {
        #[repr(C)]
        pub struct $name {
            /// Ptr to the array of elements
            ptr: *mut $t,
            /// Count of elements, do not run beyond this amount
            count: libc::size_t,
            /// Capacity of the underlying buffer
            capacity: libc::size_t,
        }
        impl From<Vec<$t>> for $name {
            fn from(mut other: Vec<$t>) -> Self {
                let ret = Self {
                    ptr: other.as_mut_ptr(),
                    count: other.len(),
                    capacity: other.capacity(),
                };
                std::mem::forget(other);
                ret
            }
        }
    };
}

pub mod parse;
