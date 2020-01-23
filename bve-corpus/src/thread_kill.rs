use std::thread::JoinHandle;

#[cfg(target_os = "windows")]
pub fn kill_thread(thread: JoinHandle<()>) -> Option<()> {
    use std::os::windows::prelude::*;

    let handle: RawHandle = thread.into_raw_handle();
    let res = unsafe { winapi::um::processthreadsapi::TerminateThread(handle, 1) };
    if res == winapi::shared::minwindef::TRUE {
        Some(())
    } else {
        None
    }
}

#[cfg(target_family = "unix")]
pub fn kill_thread(thread: JoinHandle<()>) -> Option<()> {
    use std::os::unix::thread::{JoinHandleExt, RawPthread};

    let handle: RawPthread = thread.into_pthread_t();
    let res = unsafe { libc::pthread_cancel(handle) };
    if res == 0 { Some(()) } else { None }
}

#[cfg(test)]
mod test {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn kill_thread() {
        let b = Arc::new(AtomicBool::new(false));
        let b2 = Arc::clone(&b);
        super::kill_thread(std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(500));
            b2.store(true, Ordering::SeqCst);
            assert!(false, "Thread not killed in time.")
        }))
        .unwrap();
        std::thread::sleep(Duration::from_millis(750));
        assert_eq!(b.load(Ordering::SeqCst), false);
    }
}
