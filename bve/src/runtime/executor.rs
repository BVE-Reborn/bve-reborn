use once_cell::sync::Lazy;
use std::{future::Future, sync::Arc};
use switchyard::{
    threads::{double_pool_one_to_one, thread_info},
    JoinHandle, Priority, Switchyard,
};

pub struct ThreadLocalData {}

pub static BVE_EXECUTOR: Lazy<Switchyard<ThreadLocalData>> = Lazy::new(|| {
    Switchyard::new(2, double_pool_one_to_one(thread_info(), Some("bve-executor")), || {
        ThreadLocalData {}
    })
    .expect("Could not launch executor")
});

pub enum Pool {
    Compute = 0,
    IO = 1,
}

pub fn spawn<Fut, T>(pool: Pool, priority: Priority, fut: Fut) -> JoinHandle<T>
where
    Fut: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    BVE_EXECUTOR.spawn(pool as switchyard::Pool, priority, fut)
}

pub fn spawn_local<Func, Fut, T>(pool: Pool, priority: Priority, async_fn: Func) -> JoinHandle<T>
where
    Func: FnOnce(Arc<ThreadLocalData>) -> Fut + Send + 'static,
    Fut: Future<Output = T>,
    T: Send + 'static,
{
    BVE_EXECUTOR.spawn_local(pool as switchyard::Pool, priority, async_fn)
}
