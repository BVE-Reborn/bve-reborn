use std::{cell::RefCell, io::Write, panic::PanicInfo};

thread_local! {
    pub static PANIC: RefCell<Option<String>> = RefCell::new(Some(String::new()));
    pub static USE_DEFAULT_PANIC_HANLDER: RefCell<bool> = RefCell::new(true);
}

pub fn setup_panic_hook() {
    let old_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |pi| panic_dispatch(pi, &old_hook)));
}

fn panic_dispatch(info: &PanicInfo<'_>, default_hook: &(dyn Fn(&PanicInfo<'_>) + 'static + Sync + Send)) {
    if USE_DEFAULT_PANIC_HANLDER.with(|v| *v.borrow()) {
        default_hook(info);
    } else {
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
}
