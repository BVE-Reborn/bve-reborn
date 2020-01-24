use std::cell::RefCell;
use std::io::Write;
use std::panic::PanicInfo;

thread_local! {
    pub static PANIC: RefCell<Option<String>> = RefCell::new(Some(String::new()));
}

pub fn panic_dispatch(info: &PanicInfo<'_>) {
    let bt = backtrace::Backtrace::new();
    let msg = format!(
        "Panic: {:?} {:}\n\n{:?}",
        info.payload()
            .downcast_ref::<&str>()
            .unwrap_or(&"Panic payload must be a &str"),
        info.location().expect("Panic info must have location"),
        bt
    );

    PANIC.with(|v| *v.borrow_mut() = Some(msg));
}
