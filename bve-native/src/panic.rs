use std::ffi::{c_void, CStr, CString};
use std::io::Write;
use std::os::raw::c_char;
use std::panic::PanicInfo;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Mutex;

pub type PanicHandler = unsafe extern "C" fn(*mut c_void, *const c_char) -> ();
type PanicHandlerProxy = *mut PanicHandler;
static PANIC_HANDLER: AtomicPtr<PanicHandler> = AtomicPtr::new(default_panic_handler as PanicHandlerProxy);
static PANIC_HANDLER_DATA: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

#[no_mangle]
pub unsafe extern "C" fn default_panic_handler(_: *mut c_void, string: *const c_char) {
    std::io::stderr().lock().write(CStr::from_ptr(string).to_bytes());
}

#[no_mangle]
pub unsafe extern "C" fn set_panic_handler(handler: PanicHandler) {
    let handler_transmute: PanicHandlerProxy = handler as PanicHandlerProxy;
    PANIC_HANDLER.store(handler_transmute, Ordering::SeqCst);
}

#[no_mangle]
pub unsafe extern "C" fn set_panic_data(data: *mut c_void) {
    PANIC_HANDLER_DATA.store(data, Ordering::SeqCst);
}

#[no_mangle]
pub unsafe extern "C" fn get_panic_handler() -> PanicHandler {
    let handler_transmute: PanicHandlerProxy = PANIC_HANDLER.load(Ordering::SeqCst);
    let handler: PanicHandler = std::mem::transmute(handler_transmute);
    handler
}

#[no_mangle]
pub unsafe extern "C" fn get_panic_data() -> *mut c_void {
    PANIC_HANDLER_DATA.load(Ordering::SeqCst)
}

pub fn init_panic_handler() {
    std::panic::set_hook(Box::new(panic_dispatch))
}

pub fn panic_dispatch(info: &PanicInfo<'_>) {
    let bt = backtrace::Backtrace::new();
    let msg = format!(
        "Panic: {:?} {:}\n\n{:?}",
        info.payload().downcast_ref::<&str>().unwrap(),
        info.location().unwrap(),
        bt
    );
    let c_msg = CString::new(msg).unwrap();
    unsafe {
        let handler_transmute: PanicHandlerProxy = PANIC_HANDLER.load(Ordering::SeqCst);
        let handler: PanicHandler = std::mem::transmute(handler_transmute);
        handler(PANIC_HANDLER_DATA.load(Ordering::SeqCst), c_msg.as_ptr())
    }
}
