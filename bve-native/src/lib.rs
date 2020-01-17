//! C API for BVE-Reborn high performance libraries.
//!
//! ***DO NOT CALL ANY OTHER FUNCTION BEFORE YOU CALL INIT***.
//!
//! Libraries ***must*** be initialized by calling [`init`] before calling anything else. The library
//! does not need to be de-initialized. If this is not done, panics may propagate beyond the C -> Rust boundary,
//! leading to undefined behavior.
//!
//! All pointers are assumed to not take ownership, unless otherwise specified. Non-obvious lifespans will
//! be noted in documentation.

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
#![allow(clippy::as_conversions)]
#![allow(clippy::crosspointer_transmute)] // Useful for nasty ffi crap
#![allow(clippy::decimal_literal_representation)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::float_cmp_const)]
#![allow(clippy::implicit_return)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::integer_arithmetic)]
#![allow(clippy::integer_division)]
#![allow(clippy::let_underscore_must_use)]
#![allow(clippy::match_bool)] // prettier
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

pub mod panic;

use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::os::raw::*;
use std::ptr::null_mut;

/// C safe wrapper for an [`Option`].
///
/// # Safety
///
/// Reading from the `value` member is undefined behavior if `exists` is false. In practice it is zeroed.
#[repr(C)]
pub struct COption<T> {
    /// Actual value inside the option. Reading this is undefined behavior if exists is false.
    pub value: T,
    /// Flag if the value exists or not.
    pub exists: bool,
}

impl<T, U> From<Option<U>> for COption<T>
where
    U: Into<T>,
{
    #[inline]
    #[must_use]
    fn from(other: Option<U>) -> Self {
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

impl<T> Into<Option<T>> for COption<T> {
    #[inline]
    #[must_use]
    fn into(self) -> Option<T> {
        match self.exists {
            true => Some(self.value),
            false => None,
        }
    }
}

/// C safe wrapper for a [`Vec`].
///
/// # Safety
///
/// - Modifying the contents in the array is valid.
/// - Increasing `count` such that `count <= capacity` is valid.
/// - Do not manually delete/realloc the pointer. Must use the deleter for the container where this vector was found.
#[repr(C)]
pub struct CVector<T> {
    /// Ptr to the array of elements
    pub ptr: *mut T,
    /// Count of elements, do not run beyond this amount
    pub count: libc::size_t,
    /// Capacity of the underlying buffer
    pub capacity: libc::size_t,
}

impl<T, U> From<Vec<U>> for CVector<T>
where
    U: Into<T>,
{
    #[inline]
    #[must_use]
    fn from(other: Vec<U>) -> Self {
        let mut converted: Vec<T> = other.into_iter().map(std::convert::Into::into).collect();
        let ret = Self {
            ptr: converted.as_mut_ptr(),
            count: converted.len(),
            capacity: converted.capacity(),
        };
        std::mem::forget(converted);
        ret
    }
}
impl<T, U> Into<Vec<U>> for CVector<T>
where
    T: Into<U>,
{
    #[inline]
    #[must_use]
    fn into(self) -> Vec<U> {
        let converted: Vec<T> = unsafe { Vec::from_raw_parts(self.ptr, self.count, self.capacity) };
        let other: Vec<U> = converted.into_iter().map(std::convert::Into::into).collect();
        other
    }
}

/// Initialize the runtime functionality of BVE. Initializes minimal global state to make the rest
/// of the API safe to call. ***DO NOT CALL ANY OTHER FUNCTION BEFORE YOU CALL INIT***.
///
/// This function is not protected against panics as it must not panic due to the handler not being set up.
///
/// May be called multiple times, but all global state will be reset.
#[no_mangle]
pub extern "C" fn init() {
    panic::init_panic_handler();
}

/// Converts a given non-owning slice to a owning C pointer
fn string_to_owned_ptr(input: &str) -> *mut c_char {
    CString::new(input).map(CString::into_raw).unwrap_or(null_mut())
}

/// Consumes the given owning pointer and converts it to a rust string.
///
/// # Safety
///
/// - `input` **ASSUMES OWNERSHIP** must be a valid pointer. It must be zero terminated.
unsafe fn owned_ptr_to_string(input: *const c_char) -> String {
    // This cast is valid as the underlying data will never be changed
    // Either way we own it and it's being destroyed.
    CString::from_raw(input as *mut c_char).to_string_lossy().into()
}

/// Translates a non-owning pointer to a rust str. If it is not valid utf8, it is converted to
/// being owning through [`Cow`].
///
/// # Safety
///
/// - `input` must be a valid pointer. It must be zero terminated.
unsafe fn unowned_ptr_to_str(input: &*const c_char) -> Cow<'_, str> {
    CStr::from_ptr(*input).to_string_lossy()
}

/// Takes an owning pointer to a rust-generated string and deletes it.
///
/// # Safety
///
/// - `ptr` **ASSUMES OWNERSHIP** must be a valid pointer and the string must have been allocated in Rust. It must be
///   zero terminated.
#[bve_derive::c_interface]
pub unsafe extern "C" fn bve_delete_string(ptr: *mut c_char) {
    CString::from_raw(ptr);
}

pub mod parse;
