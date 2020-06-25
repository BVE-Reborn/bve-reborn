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
use smallvec::SmallVec;
use std::{
    future::Future,
    ops::DerefMut,
    sync::{Arc, Weak},
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

enum UpstreamBuffer {
    Mapping,
    Staging(Arc<IdBuffer>),
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

pub struct IdBuffer {
    pub inner: Buffer,
    pub id: usize,
}

struct Belt {
    usable: SmallVec<[Arc<IdBuffer>; 4]>,
    usage: BufferUsage,
    size: BufferAddress,
    current_id: usize,
}

impl Belt {
    fn new(usage: BufferUsage, size: BufferAddress) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            usable: SmallVec::new(),
            usage,
            size,
            current_id: 0,
        }))
    }

    fn ensure_buffer(&mut self, device: &Device) {
        if self.usable.is_empty() {
            let raw_buffer = device.create_buffer(&BufferDescriptor {
                usage: self.usage,
                size: self.size,
                mapped_at_creation: true,
                label: None,
            });
            let buffer_id = self.current_id;
            self.current_id += 1;
            self.usable.push(Arc::new(IdBuffer {
                inner: raw_buffer,
                id: buffer_id,
            }));
        }
    }

    fn get_buffer(&self) -> &IdBuffer {
        self.get_buffer_arc()
    }

    fn get_buffer_arc(&self) -> &Arc<IdBuffer> {
        self.usable
            .first()
            .expect("Cannot call get_buffer without calling ensure_buffer first")
    }

    async fn pump(lockable: Arc<Mutex<Self>>) -> impl Future<Output = ()> {
        let mut inner = lockable.lock().await;
        assert!(
            !inner.usable.is_empty(),
            "Cannot call pump without calling ensure_buffer first"
        );

        let buffer = inner.usable.remove(0);
        drop(inner);

        let mapping = buffer.inner.slice(..).map_async(MapMode::Write);
        async move {
            mapping.await.expect("Could not map buffer");
            let mut inner = lockable.lock().await;
            inner.usable.push(buffer);
        }
    }
}

/// A buffer which automatically uses either staging buffers or direct mapping to read/write to its
/// internal buffer based on the provided [`UploadStyle`]
pub struct AutomatedBuffer {
    belt: Arc<Mutex<Belt>>,
    upstream: UpstreamBuffer,
    size: BufferAddress,
}
impl AutomatedBuffer {
    /// Creates a new AutomatedBuffer with given settings. All operations directly
    /// done on the automated buffer according to `usage` will be added to the
    /// internal buffer's usage flags.
    fn new(device: &Device, size: BufferAddress, usage: BufferUsage, label: Option<&str>, style: UploadStyle) -> Self {
        let (upstream, belt_usage) = if style == UploadStyle::Staging {
            (
                UpstreamBuffer::Staging(Arc::new(IdBuffer {
                    inner: device.create_buffer(&BufferDescriptor {
                        size,
                        usage: BufferUsage::COPY_DST | usage,
                        label,
                        mapped_at_creation: false,
                    }),
                    id: 0,
                })),
                BufferUsage::MAP_WRITE | BufferUsage::COPY_SRC,
            )
        } else {
            (UpstreamBuffer::Mapping, BufferUsage::MAP_WRITE | usage)
        };

        Self {
            belt: Belt::new(belt_usage, size),
            upstream,
            size,
        }
    }

    pub async fn count(&self) -> usize {
        self.belt.lock().await.current_id
    }

    pub async fn get_current_inner(&self) -> Arc<IdBuffer> {
        match self.upstream {
            UpstreamBuffer::Mapping => Arc::clone(self.belt.lock().await.get_buffer_arc()),
            UpstreamBuffer::Staging(ref arc) => Arc::clone(arc),
        }
    }

    /// Writes to the underlying buffer using the proper write style.
    pub async fn write_to_buffer<'a, DataFn>(
        &'a mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        data_fn: DataFn,
    ) where
        DataFn: FnOnce(&mut [u8]) + 'a,
    {
        let mut inner = self.belt.lock().await;
        inner.ensure_buffer(device);
        let buffer = inner.get_buffer();
        let slice = buffer.inner.slice(..);
        let mut mapping = slice.get_mapped_range_mut();
        data_fn(mapping.deref_mut());
        drop(mapping);
        buffer.inner.unmap();

        if let UpstreamBuffer::Staging(ref upstream) = self.upstream {
            encoder.copy_buffer_to_buffer(&buffer.inner, 0, &upstream.inner, 0, self.size);
        }
    }
}
