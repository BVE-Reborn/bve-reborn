use crate::{AutomatedBuffer, BeltBufferId, IdBuffer};
use std::{future::Future, hash::Hash, sync::Arc};
use wgpu::*;

#[cfg(doc)]
pub struct FutMock<T>(std::marker::PhantomData<T>);

#[cfg(doc)]
impl<T> Future for FutMock<T> {
    type Output = T;

    fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        unimplemented!()
    }
}

pub trait AutomatedBufferSet<'buf> {
    type Key: Hash + Eq + Clone;
    type Value;
    type Return: Future<Output = Self::Value> + 'buf;
    fn get(self) -> Self::Return;
    fn value_to_key(value: &Self::Value) -> Self::Key;
}

impl<'buf> AutomatedBufferSet<'buf> for &'buf AutomatedBuffer {
    type Key = BeltBufferId;
    type Value = Arc<IdBuffer>;
    #[cfg(not(doc))]
    type Return = impl Future<Output = Self::Value> + 'buf;
    #[cfg(doc)]
    type Return = FutMock<Self::Value>;
    fn get(self) -> Self::Return {
        async move { self.get_current_inner().await }
    }

    fn value_to_key(value: &Self::Value) -> Self::Key {
        value.id
    }
}

impl<'buf> AutomatedBufferSet<'buf> for (&'buf AutomatedBuffer,) {
    type Key = BeltBufferId;
    type Value = Arc<IdBuffer>;
    #[cfg(not(doc))]
    type Return = impl Future<Output = Self::Value> + 'buf;
    #[cfg(doc)]
    type Return = FutMock<Self::Value>;
    fn get(self) -> Self::Return {
        async move { self.0.get_current_inner().await }
    }

    fn value_to_key(value: &Self::Value) -> Self::Key {
        value.id
    }
}

impl<'buf> AutomatedBufferSet<'buf> for (&'buf AutomatedBuffer, &'buf AutomatedBuffer) {
    type Key = (BeltBufferId, BeltBufferId);
    type Value = (Arc<IdBuffer>, Arc<IdBuffer>);
    #[cfg(not(doc))]
    type Return = impl Future<Output = Self::Value> + 'buf;
    #[cfg(doc)]
    type Return = FutMock<Self::Value>;
    fn get(self) -> Self::Return {
        async move { (self.0.get_current_inner().await, self.1.get_current_inner().await) }
    }

    fn value_to_key(value: &Self::Value) -> Self::Key {
        (value.0.id, value.1.id)
    }
}

impl<'buf> AutomatedBufferSet<'buf> for (&'buf AutomatedBuffer, &'buf AutomatedBuffer, &'buf AutomatedBuffer) {
    type Key = (BeltBufferId, BeltBufferId, BeltBufferId);
    type Value = (Arc<IdBuffer>, Arc<IdBuffer>, Arc<IdBuffer>);
    #[cfg(not(doc))]
    type Return = impl Future<Output = Self::Value> + 'buf;
    #[cfg(doc)]
    type Return = FutMock<Self::Value>;
    fn get(self) -> Self::Return {
        async move {
            (
                self.0.get_current_inner().await,
                self.1.get_current_inner().await,
                self.2.get_current_inner().await,
            )
        }
    }

    fn value_to_key(value: &Self::Value) -> Self::Key {
        (value.0.id, value.1.id, value.2.id)
    }
}

impl<'buf> AutomatedBufferSet<'buf>
    for (
        &'buf AutomatedBuffer,
        &'buf AutomatedBuffer,
        &'buf AutomatedBuffer,
        &'buf AutomatedBuffer,
    )
{
    type Key = (BeltBufferId, BeltBufferId, BeltBufferId, BeltBufferId);
    type Value = (Arc<IdBuffer>, Arc<IdBuffer>, Arc<IdBuffer>, Arc<IdBuffer>);
    #[cfg(not(doc))]
    type Return = impl Future<Output = Self::Value> + 'buf;
    #[cfg(doc)]
    type Return = FutMock<Self::Value>;
    fn get(self) -> Self::Return {
        async move {
            (
                self.0.get_current_inner().await,
                self.1.get_current_inner().await,
                self.2.get_current_inner().await,
                self.3.get_current_inner().await,
            )
        }
    }

    fn value_to_key(value: &Self::Value) -> Self::Key {
        (value.0.id, value.1.id, value.2.id, value.3.id)
    }
}

pub struct BindGroupCache<Key: Hash + Eq + Clone> {
    cache: lru::LruCache<Key, BindGroup>,
}
impl<Key: Hash + Eq + Clone> BindGroupCache<Key> {
    pub fn new() -> Self {
        Self::with_capacity(4)
    }

    pub fn with_capacity(size: usize) -> Self {
        Self {
            cache: lru::LruCache::new(size),
        }
    }

    pub async fn create_bind_group<'a, Set, BindGroupFn>(&mut self, buffers: Set, bind_group_fn: BindGroupFn) -> Key
    where
        Set: AutomatedBufferSet<'a, Key = Key>,
        BindGroupFn: FnOnce(&Set::Value) -> BindGroup,
    {
        let value = buffers.get().await;
        let key = Set::value_to_key(&value);
        if self.cache.contains(&key) {
            return key;
        }
        // Bumps LRU-ness
        self.cache.put(key.clone(), bind_group_fn(&value));
        key
    }

    pub fn get(&self, key: Key) -> Option<&BindGroup> {
        self.cache.peek(&key)
    }
}
