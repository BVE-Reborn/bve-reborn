use crate::log::{SerializationMethod, Subscriber};
use std::io::Write;
use std::sync::Mutex;
use std::thread::JoinHandle;
use tracing_core::dispatcher::with_default;
use tracing_core::Dispatch;

lazy_static::lazy_static! {
    static ref GLOBAL_LOGGER: Mutex<Option<Subscriber>> = Mutex::new(None);
}

/// Automatically removes the global logger when it goes out of scope, making sure everything gets cleaned up correctly.
pub struct GlobalLoggerGuard();

impl Drop for GlobalLoggerGuard {
    fn drop(&mut self) {
        if let Ok(mut guard) = GLOBAL_LOGGER.lock() {
            *guard = None
        }
    }
}

/// Sets the global logger to a newly made BVE event subscriber.
///
/// # Arguments
///
/// - `dest` the destination of the logger. Can be a file, stdout/err, or anything else that implements `Write` and
///   `Send` and `'static`.
/// - `method` the [format](SerializationMethod) the logger should serialize its messages in.
///
/// # Returns
///
/// [`GlobalLoggerGuard`] which automatically removes the global logger when it goes out of scope, making sure
/// everything gets cleaned up correctly.
pub fn set_global_logger(dest: impl Write + Send + 'static, method: SerializationMethod) -> GlobalLoggerGuard {
    *GLOBAL_LOGGER.lock().expect("Cannot lock to set global logger") = Some(Subscriber::new(dest, method));
    GlobalLoggerGuard()
}

/// Removes the global logger.
///
/// May be called multiple times.
pub fn remove_global_logger() {
    *GLOBAL_LOGGER.lock().expect("Cannot lock to clear global logger") = None;
}

/// Runs the function with the defined global logger.
pub fn run_with_global_logger<T>(f: impl FnOnce() -> T) -> T {
    let global = GLOBAL_LOGGER
        .lock()
        .expect("Cannot lock to get global logger")
        .clone()
        .map_or_else(Dispatch::none, Dispatch::new);
    with_default(&global, f)
}

/// Spawn a thread with the currently set global logger.
///
/// This is aliased in the concurrency module.
pub fn thread_spawn_with_global_logger<T>(f: impl FnOnce() -> T + Send + 'static) -> JoinHandle<T>
where
    T: Send + 'static,
{
    std::thread::spawn(move || run_with_global_logger(f))
}
