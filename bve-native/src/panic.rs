//! Controlling the behavior of BVE when it panics.
//!
//! Panicking is by definition a bug in BVE, but it does happen, and it needs to be handled
//! before the application is shutdown. The global panic handler consists of two parts.
//! A function pointer and a void* data. They are both stored globally and can be controlled with
//! the functions in this module. The function is called with the void* and a string which
//! contains printable information about the panic.
//!
//! There is a [`bve_default_panic_handler`] which is called if you don't manually set your own.
//! This takes the provided string and prints it to stderr and returns.
//!
//! # Safety
//!
//! There is a race condition between [`bve_set_panic_handler`] and [`bve_set_panic_data`].
//!
//! If:
//! - you set the panic handler
//! - haven't called panic data to the proper pointer
//! - there is a panic in rust code running concurrently,
//!
//! the new panic handler will be called with the wrong pointer. This can cause all kinds of bad things if the panic
//! handler expects that pointer to be of a specific type.
//!
//! This is, in practice, little cause for concern with this race. As long as no other rust code is executing, there is
//! no problem. If you call these at the beginning of the program, like would be expected, there's no chace of race.

use std::ffi::{c_void, CStr, CString};
use std::io::Write;
use std::os::raw::c_char;
use std::panic::PanicInfo;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, Ordering};

/// Function pointer type for the Panic Handler.
///
/// # Arguments
///
/// - `void*`: The data pointer provided using [`bve_set_panic_data`]. Must be able to gracefully deal with null.
/// - `const char*`: String containing human readable information about the panic, including a backtrace. Will never be
///   null.
///
/// # Safety
///
/// Always allow `void*` to be null. You may assume the string is never null, utf8, and null terminated.
pub type PanicHandler = unsafe extern "C" fn(*mut c_void, *const c_char) -> ();

// Due to the lack of Atomic Function Pointers, we must do some bullshit that is very questionable.
type PanicHandlerProxy = *mut PanicHandler;

// Internal statics for the Panic Handler and its data
static PANIC_HANDLER: AtomicPtr<PanicHandler> = AtomicPtr::new(bve_default_panic_handler as PanicHandlerProxy);
static PANIC_HANDLER_DATA: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

/// The default panic handler that is automatically installed. Does not touch the data pointer.
/// Prints the string to stderr, panicking (and aborting) if it fails.
///
/// # Safety
///
/// The string must be non-null per the contract of [`PanicHandler`].
#[no_mangle]
pub unsafe extern "C" fn bve_default_panic_handler(_: *mut c_void, string: *const c_char) {
    std::io::stderr()
        .lock()
        .write_all(CStr::from_ptr(string).to_bytes())
        .expect("Writing to stderr in panic handler failed.");
}

/// Sets the panic handler to the provided function pointer.
///
/// # Safety
///
/// - `handler` must not be null and must point to a valid function of the proper signature.
/// - The function `handler` points to must uphold the invariants of the contract of [`PanicHandler`]
/// - There is a minor race between this function and [`bve_set_panic_data`]. See module documentation.
#[no_mangle]
pub unsafe extern "C" fn bve_set_panic_handler(handler: PanicHandler) {
    let handler_transmute: PanicHandlerProxy = handler as PanicHandlerProxy;
    PANIC_HANDLER.store(handler_transmute, Ordering::SeqCst);
}

/// Sets the data passed to the panic handler.
///
/// # Safety
///
/// - If the installed panic handler touches this data, it must be non-null and point to the data it expects
/// - There is a minor race between this function and [`bve_set_panic_handler`]. See module documentation.
#[no_mangle]
pub unsafe extern "C" fn bve_set_panic_data(data: *mut c_void) {
    PANIC_HANDLER_DATA.store(data, Ordering::SeqCst);
}

/// Returns the currently set panic handler. Non-null.
#[no_mangle]
pub extern "C" fn bve_get_panic_handler() -> PanicHandler {
    let handler_transmute: PanicHandlerProxy = PANIC_HANDLER.load(Ordering::SeqCst);
    let handler: PanicHandler = unsafe { std::mem::transmute(handler_transmute) };
    handler
}

/// Returns the currently set data to be passed to the panic handler. May be null.
#[no_mangle]
pub extern "C" fn bve_get_panic_data() -> *mut c_void {
    PANIC_HANDLER_DATA.load(Ordering::SeqCst)
}

/// Hooks up our panic hook to the standard library hook.
pub(crate) fn init_panic_handler() {
    std::panic::set_hook(Box::new(panic_dispatch))
}

/// Direct callback from rust that provides the information for the panic handler and calls it.
fn panic_dispatch(info: &PanicInfo<'_>) {
    let bt = backtrace::Backtrace::new();
    let msg = format!(
        "Panic: {:?} {:}\n\n{:?}",
        info.payload()
            .downcast_ref::<&str>()
            .expect("Panic payload must be a &str"),
        info.location().expect("Panic info must have location"),
        bt
    );
    let c_msg = CString::new(msg).expect("Formatted message must not have interior null byte");
    unsafe {
        let handler_transmute: PanicHandlerProxy = PANIC_HANDLER.load(Ordering::SeqCst);
        let handler: PanicHandler = std::mem::transmute(handler_transmute);
        handler(PANIC_HANDLER_DATA.load(Ordering::SeqCst), c_msg.as_ptr())
    }
}
