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

use crate::parse::mesh::{Mesh, Mesh_Error};
use bve::parse::mesh::Vertex;
use bve::ColorU8RGB;
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::os::raw::*;
use std::ptr::null_mut;

macro_rules! bve_option {
    ($name:ident, $t:ty) => {
        #[repr(C)]
        pub struct $name {
            /// Actual value inside the option. Reading this is undefined behavior if exists is false.
            value: $t,
            /// Flag if the value exists or not.
            exists: bool,
        }

        impl<T> From<Option<T>> for $name
        where
            T: Into<$t>,
        {
            #[inline]
            #[must_use]
            fn from(other: Option<T>) -> Self {
                match other {
                    Some(v) => Self {
                        value: v.into(),
                        exists: true,
                    },
                    None => Self {
                        value: unsafe { std::mem::zeroed() },
                        exists: false,
                    },
                }
            }
        }

        impl Into<Option<$t>> for $name {
            #[inline]
            #[must_use]
            fn into(self) -> Option<$t> {
                match self.exists {
                    true => Some(self.value),
                    false => None,
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

        impl<T> From<Vec<T>> for $name
        where
            T: Into<$t>,
        {
            #[inline]
            #[must_use]
            fn from(other: Vec<T>) -> Self {
                let mut converted: Vec<$t> = other.into_iter().map(|v| v.into()).collect();
                let ret = Self {
                    ptr: converted.as_mut_ptr(),
                    count: converted.len(),
                    capacity: converted.capacity(),
                };
                std::mem::forget(converted);
                ret
            }
        }

        impl<T> Into<Vec<T>> for $name
        where
            $t: Into<T>,
        {
            #[inline]
            #[must_use]
            fn into(self) -> Vec<T> {
                let converted: Vec<$t> = unsafe { Vec::from_raw_parts(self.ptr, self.count, self.capacity) };
                let other: Vec<T> = converted.into_iter().map(|v| v.into()).collect();
                other
            }
        }
    };
}

bve_option!(Option_unsigned_long_long, c_ulonglong);
bve_option!(Option_size_t, libc::size_t);
bve_option!(Option_ColorU8RGB, ColorU8RGB);

bve_vector!(Vector_size_t, libc::size_t);
bve_vector!(Vector_Mesh, Mesh);

bve_vector!(Vector_Mesh_Error, Mesh_Error);
bve_vector!(Vector_Vertex, Vertex);

fn string_to_owned_ptr(input: &str) -> *mut c_char {
    CString::new(input).map(|v| v.into_raw()).unwrap_or(null_mut())
}

/// Consumes the given owning pointer and converts it to a rust string.
unsafe fn owned_ptr_to_string(input: *const c_char) -> String {
    // This cast is valid as the underlying data will never be changed
    // Either way we own it and it's being destroyed.
    CString::from_raw(input as *mut c_char).to_string_lossy().into()
}

unsafe fn unowned_ptr_to_str(input: *const c_char) -> Cow<'static, str> {
    CStr::from_ptr(input).to_string_lossy()
}

#[no_mangle]
pub unsafe extern "C" fn bve_delete_string(ptr: *mut c_char) {
    CString::from_raw(ptr);
}

pub mod parse;
