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
    use std::time::Duration;

    #[test]
    fn kill_thread() {
        super::kill_thread(std::thread::spawn(|| {
            std::thread::sleep(Duration::from_secs(1));
            assert!(false, "Thread not killed in time.")
        }))
        .unwrap();
    }
}
