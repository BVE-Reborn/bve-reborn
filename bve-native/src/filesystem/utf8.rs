use crate::unowned_ptr_to_str;
use async_std::task::block_on;
use bve_derive::c_interface;
use std::{ffi::CString, os::raw::c_char, ptr::NonNull};

/// Reads the file at `filename` with [`bve::filesystem::read_convert_utf8`].
///
/// # Safety
///
/// - Pointer returned points to an **owned** string containing the contents of the file in utf8.
/// - Returned pointer must be deleted by [`crate::bve_delete_string`].
/// - If file loading fails, output is null.
#[must_use]
#[c_interface]
pub unsafe extern "C" fn bve_filesystem_read_convert_utf8(filename_ptr: *const c_char) -> Option<NonNull<c_char>> {
    let filename = unowned_ptr_to_str(&filename_ptr);

    let result = block_on(bve::filesystem::read_convert_utf8(filename.as_ref()));

    result
        .ok()
        .and_then(|v| CString::new(v).ok())
        .map(CString::into_raw)
        .and_then(NonNull::new)
}
