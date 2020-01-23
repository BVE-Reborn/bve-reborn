use std::thread::JoinHandle;

#[cfg(target_os = "windows")]
pub fn kill_thread(thread: JoinHandle<()>) {
    use std::os::windows::prelude::*;

    let handle: RawHandle = thread.into_raw_handle();
    unsafe { winapi::um::processthreadsapi::TerminateThread(handle, 1) };
}

#[cfg(target_family = "unix")]
pub fn kill_thread(thread: JoinHandle<()>) {
    use std::os::unix::thread::{JoinHandleExt, RawPthread};

    let handle: RawPthread = thread.into_pthread_t();
    unsafe { libc::pthread_cacnel(handle) };
}
