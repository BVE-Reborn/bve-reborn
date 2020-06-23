//! C/C++ API for BVE-Reborn high performance libraries.
//!
//! This library isn't made to be used directly from rust and can actually create UB fairly easily if used from Rust.
//! C/C++ Headers will be generated in `bve-native/include` by `bve-build`.
//!
//! Documentation is automatically copied to the C headers, but is missing module documentation. Therefore it is
//! recommended to use the Rust documentation for this library while writing C/C++. Attempts have been made to make
//! it as easy as possible to figure out what C/C++ code to write from the Rust documentation.
//!
//! While code is separated into modules in the rust documentation, all functions and types are exported to c++ in
//! global scope with no name mangling.
//!
//! The modules correspond 1:1 with the modules in the `bve` crate.  
//!
//! # Use Warning
//!
//! ***DO NOT CALL ANY OTHER FUNCTION BEFORE YOU CALL INIT***.
//!
//! Libraries ***must*** be initialized by calling [`bve_init`] before calling anything else. The library
//! does not need to be de-initialized. If this is not done, panics may propagate beyond the C -> Rust boundary,
//! leading to undefined behavior.
//!
//! # API Basics
//!
//! The API tries to be as consistent and predictable as possible.
//!
//! Structs are all prefixed with `BVE_` in C mode to help eliminate conflicts. C++ code is all in the `bve` namespace.
//! Type names are kept as short as is reasonable, while still having clear connection with the underlying rust code.
//!
//! Free functions have their rust path loosely encoded in the name. For example
//! [`bve::load::mesh::load_mesh_from_file`] is [`bve_load_mesh_from_file`](load::mesh::bve_load_mesh_from_file).
//! Duplicate names are removed and idioms are changed to be comprehensible from the interface language.
//!
//! Free functions that are acting like member functions come in the form `BVE_Struct_Name_member_function_name`. They
//! take their first argument by pointer.
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
// Rustdoc Warnings
#![deny(intra_doc_link_resolution_failure)]
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
#![allow(clippy::too_many_lines)] // This is also dumb
// Annoying/irrelevant clippy Restrictions
#![allow(clippy::as_conversions)]
#![allow(clippy::crosspointer_transmute)] // Useful for nasty ffi crap
#![allow(clippy::decimal_literal_representation)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::expect_used)]
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
#![allow(clippy::multiple_crate_versions)] // Cargo deny's job
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::panic)]
#![allow(clippy::shadow_reuse)]
#![allow(clippy::shadow_same)]
#![allow(clippy::unreachable)]
#![allow(clippy::wildcard_enum_match_arm)]
#![allow(clippy::wildcard_imports)]

pub mod filesystem;
pub mod interfaces;
pub mod load;
pub mod panic;
pub mod parse;

use std::{
    borrow::Cow,
    ffi::{CStr, CString},
    os::raw::*,
    ptr::null_mut,
};

/// C safe wrapper for an [`Option`].
///
/// # Safety
///
/// Reading from the `value` member is undefined behavior if `exists` is false. In practice it is zeroed.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
pub extern "C" fn bve_init() {
    panic::init_panic_handler();
}

/// Converts a given owning string to a owning C pointer
fn string_to_owned_ptr(input: String) -> *mut c_char {
    CString::new(input).map(CString::into_raw).unwrap_or(null_mut())
}

/// Converts a given non-owning slice to a owning C pointer
fn str_to_owned_ptr(input: &str) -> *mut c_char {
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

/// Copies a non-owning pointer to a new owning pointer. Does not need to be utf-8, but needs to be
/// null terminated.
///
/// # Safety
///
/// - `input` must be a valid null-terminated string. May be nullptr.
/// - Returned string must be deallocated by [`bve_delete_string`] or equivalent
unsafe fn copy_string(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return null_mut();
    }
    CStr::from_ptr(input).to_owned().into_raw()
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
