// Rust warnings
#![warn(unused)]
#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
// Rustdoc Warnings
#![deny(intra_doc_link_resolution_failure)]
// Clippy warnings
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::restriction)]
// Annoying regular clippy warnings
#![allow(clippy::cast_lossless)] // Annoying
#![allow(clippy::cast_sign_loss)] // Annoying
#![allow(clippy::cast_precision_loss)] // Annoying
#![allow(clippy::cast_possible_truncation)] // Annoying
#![allow(clippy::cognitive_complexity)] // This is dumb
#![allow(clippy::too_many_lines)] // This is also dumb
// Annoying/irrelevant clippy Restrictions
#![allow(clippy::as_conversions)]
#![allow(clippy::decimal_literal_representation)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::expect_used)]
#![allow(clippy::fallible_impl_from)] // This fails horribly when you try to panic in a macro inside a From impl
#![allow(clippy::float_arithmetic)]
#![allow(clippy::float_cmp)]
#![allow(clippy::float_cmp_const)]
#![allow(clippy::future_not_send)]
#![allow(clippy::implicit_return)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::integer_arithmetic)]
#![allow(clippy::integer_division)]
#![allow(clippy::let_underscore_must_use)]
#![allow(clippy::match_bool)] // prettier
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::multiple_crate_versions)] // Cargo deny's job
#![allow(clippy::multiple_inherent_impl)]
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::panic)]
#![allow(clippy::similar_names)]
#![allow(clippy::shadow_reuse)]
#![allow(clippy::shadow_same)]
#![allow(clippy::string_add)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnested_or_patterns)] // CLion no loves me
#![allow(clippy::unreachable)]
#![allow(clippy::wildcard_enum_match_arm)]
#![allow(clippy::wildcard_imports)]

use async_std::sync::Mutex;
use std::{
    future::Future,
    ops::DerefMut,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Weak,
    },
};
use wgpu::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum UploadStyle {
    Mapping,
    Staging,
}
impl UploadStyle {
    pub fn from_device_type(ty: DeviceType) -> Self {
        match ty {
            DeviceType::IntegratedGpu | DeviceType::Cpu => UploadStyle::Mapping,
            DeviceType::DiscreteGpu | DeviceType::VirtualGpu | DeviceType::Other => UploadStyle::Staging,
        }
    }
}

pub struct AutomatedBufferManager {
    belts: Vec<Weak<Mutex<Belt>>>,
    style: UploadStyle,
}
impl AutomatedBufferManager {
    pub fn new(style: UploadStyle) -> Self {
        AutomatedBufferManager {
            belts: Vec::new(),
            style,
        }
    }

    pub fn create_new_buffer(
        &mut self,
        device: &Device,
        size: BufferAddress,
        usage: BufferUsage,
        label: Option<&str>,
    ) -> AutomatedBuffer {
        let buffer = AutomatedBuffer::new(device, size, usage, label, self.style);
        self.belts.push(Arc::downgrade(&buffer.belt));
        buffer
    }

    pub async fn pump(&mut self) {
        let mut valid = Vec::with_capacity(self.belts.len());
        for belt in &self.belts {
            if let Some(belt) = belt.upgrade() {
                async_std::task::spawn(Belt::pump(belt).await);
                valid.push(true);
            } else {
                valid.push(false);
            }
        }
        let mut valid_iter = valid.into_iter();
        self.belts.retain(|_| valid_iter.next().unwrap_or(false));
    }
}

fn check_should_resize(current: BufferAddress, desired: BufferAddress) -> Option<BufferAddress> {
    assert!(current.is_power_of_two());
    if current == 16 && desired <= 16 {
        return None;
    }
    let lower_bound = current / 4;
    if desired <= lower_bound || current < desired {
        Some((desired + 1).next_power_of_two())
    } else {
        None
    }
}

pub struct IdBuffer {
    pub inner: Buffer,
    pub id: usize,
    size: BufferAddress,
    dirty: AtomicBool,
}

struct Belt {
    usable: Option<Arc<IdBuffer>>,
    usage: BufferUsage,
    current_id: usize,
    live_buffers: usize,
}

impl Belt {
    fn new(usage: BufferUsage) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            usable: None,
            usage,
            current_id: 0,
            live_buffers: 0,
        }))
    }

    fn create_buffer(&mut self, device: &Device, size: BufferAddress) {
        let raw_buffer = device.create_buffer(&BufferDescriptor {
            usage: self.usage,
            size,
            mapped_at_creation: true,
            label: None,
        });
        let buffer_id = self.current_id;
        self.current_id += 1;
        self.usable = Some(Arc::new(IdBuffer {
            inner: raw_buffer,
            id: buffer_id,
            size,
            dirty: AtomicBool::new(false),
        }));
    }

    fn ensure_buffer(&mut self, device: &Device, size: BufferAddress) {
        if let Some(ref old) = self.usable {
            if let Some(new_size) = check_should_resize(old.size, size) {
                self.create_buffer(device, new_size);
            }
        } else {
            self.create_buffer(device, size.next_power_of_two().max(16));
            self.live_buffers += 1;
        }
    }

    fn get_buffer(&self) -> &IdBuffer {
        self.get_buffer_arc()
    }

    fn get_buffer_arc(&self) -> &Arc<IdBuffer> {
        self.usable
            .as_ref()
            .expect("Cannot call get_buffer without calling ensure_buffer first")
    }

    async fn pump(lockable: Arc<Mutex<Self>>) -> impl Future<Output = ()> {
        let mut inner = lockable.lock().await;

        let buffer = inner
            .usable
            .take()
            .expect("Cannot call pump without calling ensure_buffer first");
        drop(inner);

        if !buffer.dirty.load(Ordering::Relaxed) {
            buffer.inner.unmap()
        }
        let mapping = buffer.inner.slice(..).map_async(MapMode::Write);
        async move {
            mapping.await.expect("Could not map buffer");
            let mut inner = lockable.lock().await;
            buffer.dirty.store(false, Ordering::Relaxed);
            if let Some(old) = inner.usable.replace(buffer) {
                drop(old);
                inner.live_buffers -= 1;
            };
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AutomatedBufferStats {
    current_id: usize,
    live_buffers: usize,
}

enum UpstreamBuffer {
    Mapping,
    Staging {
        inner: Arc<IdBuffer>,
        usage: BufferUsage,
        label: Option<String>,
    },
}

/// A buffer which automatically uses either staging buffers or direct mapping to read/write to its
/// internal buffer based on the provided [`UploadStyle`]
pub struct AutomatedBuffer {
    belt: Arc<Mutex<Belt>>,
    upstream: UpstreamBuffer,
}
impl AutomatedBuffer {
    /// Creates a new AutomatedBuffer with given settings. All operations directly
    /// done on the automated buffer according to `usage` will be added to the
    /// internal buffer's usage flags.
    fn new(
        device: &Device,
        initial_size: BufferAddress,
        usage: BufferUsage,
        label: Option<&str>,
        style: UploadStyle,
    ) -> Self {
        let initial_size = initial_size.next_power_of_two().max(16);
        let (upstream, belt_usage) = if style == UploadStyle::Staging {
            let upstream_usage = BufferUsage::COPY_DST | usage;
            let belt_usage = BufferUsage::MAP_WRITE | BufferUsage::COPY_SRC;
            let upstream = UpstreamBuffer::Staging {
                inner: Arc::new(IdBuffer {
                    inner: device.create_buffer(&BufferDescriptor {
                        size: initial_size,
                        usage: upstream_usage,
                        label,
                        mapped_at_creation: false,
                    }),
                    id: 0,
                    dirty: AtomicBool::new(false),
                    size: initial_size,
                }),
                usage: upstream_usage,
                label: label.map(String::from),
            };
            (upstream, belt_usage)
        } else {
            let belt_usage = BufferUsage::MAP_WRITE | usage;
            let upstream = UpstreamBuffer::Mapping;
            (upstream, belt_usage)
        };

        Self {
            belt: Belt::new(belt_usage),
            upstream,
        }
    }

    pub async fn stats(&self) -> AutomatedBufferStats {
        let guard = self.belt.lock().await;
        AutomatedBufferStats {
            current_id: guard.current_id,
            live_buffers: guard.live_buffers,
        }
    }

    pub async fn get_current_inner(&self) -> Arc<IdBuffer> {
        match self.upstream {
            UpstreamBuffer::Mapping => Arc::clone(self.belt.lock().await.get_buffer_arc()),
            UpstreamBuffer::Staging { inner: ref arc, .. } => Arc::clone(arc),
        }
    }

    fn ensure_upstream(&mut self, device: &Device, size: BufferAddress) {
        let size = size.max(16);
        if let UpstreamBuffer::Staging {
            ref mut inner,
            usage,
            ref label,
        } = self.upstream
        {
            if let Some(new_size) = check_should_resize(inner.size, size) {
                let new_buffer = device.create_buffer(&BufferDescriptor {
                    size: new_size,
                    label: label.as_deref(),
                    mapped_at_creation: false,
                    usage,
                });
                *inner = Arc::new(IdBuffer {
                    inner: new_buffer,
                    size: new_size,
                    dirty: AtomicBool::new(false),
                    id: inner.id + 1,
                })
            }
        }
    }

    /// Writes to the underlying buffer using the proper write style.
    pub async fn write_to_buffer<DataFn>(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        size: BufferAddress,
        data_fn: DataFn,
    ) where
        DataFn: FnOnce(&mut [u8]),
    {
        self.ensure_upstream(device, size);
        let mut inner = self.belt.lock().await;
        inner.ensure_buffer(device, size);
        let buffer = inner.get_buffer();
        let slice = buffer.inner.slice(0..size);
        let mut mapping = slice.get_mapped_range_mut();
        data_fn(mapping.deref_mut());
        drop(mapping);
        buffer.dirty.store(true, Ordering::Relaxed);
        buffer.inner.unmap();

        if let UpstreamBuffer::Staging { ref inner, .. } = self.upstream {
            encoder.copy_buffer_to_buffer(&buffer.inner, 0, &inner.inner, 0, size as BufferAddress);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::check_should_resize;

    #[test]
    fn automated_buffer_resize() {
        assert_eq!(check_should_resize(64, 128), Some(256));
        assert_eq!(check_should_resize(128, 128), None);
        assert_eq!(check_should_resize(256, 128), None);

        assert_eq!(check_should_resize(64, 64), None);
        assert_eq!(check_should_resize(128, 64), None);
        assert_eq!(check_should_resize(256, 65), None);
        assert_eq!(check_should_resize(256, 64), Some(128));
        assert_eq!(check_should_resize(256, 63), Some(64));

        assert_eq!(check_should_resize(16, 16), None);
        assert_eq!(check_should_resize(16, 8), None);
        assert_eq!(check_should_resize(16, 4), None);
    }
}
