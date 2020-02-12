use crate::unowned_ptr_to_str;
use bve_derive::c_interface;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr::NonNull;

/// Reads the file at `filename` with [`bve::filesystem::read_convert_utf8`].
///
/// # Safety
///
/// - Pointer returned points to an **owned** string containing the contents of the file in utf8.
/// - Returned pointer must be deleted by [`bve_delete_string`].
/// - If file loading fails, output is null.
#[must_use]
#[c_interface]
pub unsafe extern "C" fn bve_filesystem_read_convert_utf8(filename: *const c_char) -> Option<NonNull<c_char>> {
    let cow = unowned_ptr_to_str(&filename);

    let result = bve::filesystem::read_convert_utf8(cow.as_ref());

    result
        .ok()
        .and_then(|v| CString::new(v).ok())
        .map(CString::into_raw)
        .and_then(NonNull::new)
}
