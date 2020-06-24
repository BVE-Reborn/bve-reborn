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

use std::{future::Future, ops::Deref};
use wgpu::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum UploadStyle {
    Mapping,
    Staging,
}

bitflags::bitflags! {
    pub struct AutomatedBufferUsage: u8 {
        const READ = 0b01;
        const WRITE = 0b10;
        const ALL = Self::READ.bits | Self::WRITE.bits;
    }
}

impl AutomatedBufferUsage {
    pub fn into_buffer_usage(self, style: UploadStyle) -> BufferUsage {
        let mut usage = BufferUsage::empty();
        if self.contains(Self::READ) {
            match style {
                UploadStyle::Mapping => usage.insert(BufferUsage::MAP_READ),
                UploadStyle::Staging => usage.insert(BufferUsage::COPY_SRC),
            }
        }
        if self.contains(Self::WRITE) {
            match style {
                UploadStyle::Mapping => usage.insert(BufferUsage::MAP_WRITE),
                UploadStyle::Staging => usage.insert(BufferUsage::COPY_DST),
            }
        }
        usage
    }
}

type BufferWriteResult = Result<(), BufferAsyncError>;

/// A buffer which automatically uses either staging buffers or direct mapping to read/write to its
/// internal buffer based on the provided [`UploadStyle`]
pub struct AutomatedBuffer {
    inner: Buffer,
    style: UploadStyle,
    usage: AutomatedBufferUsage,
    size: BufferAddress,
}
impl AutomatedBuffer {
    /// Creates a new AutomatedBuffer with given settings. All operations directly
    /// done on the automated buffer according to `usage` will be added to the
    /// internal buffer's usage flags.
    pub fn new(
        device: &Device,
        size: BufferAddress,
        usage: AutomatedBufferUsage,
        other_usages: BufferUsage,
        label: Option<&str>,
        style: UploadStyle,
    ) -> Self {
        let inner = device.create_buffer(&BufferDescriptor {
            size,
            usage: usage.into_buffer_usage(style) | other_usages,
            label,
            mapped_at_creation: false,
        });

        Self {
            inner,
            style,
            usage,
            size,
        }
    }

    /// When the returned future is awaited, writes the data to the buffer if it is a mapped buffer.
    /// No-op for the use of a staging buffer.
    fn map_write<'a>(
        data: &'a [u8],
        mapping: Option<impl Future<Output = BufferWriteResult> + 'a>,
    ) -> impl Future<Output = ()> + 'a {
        async move {
            if let Some(mapping) = mapping {
                mapping.await.unwrap().as_slice().copy_from_slice(data);
            }
        }
    }

    /// Writes to the underlying buffer using the proper write style.
    ///
    /// This function is safe, but has the following constraints so as to not cause a panic in wgpu:
    ///  - Buffer usage must contain [`WRITE`](AutomatedBufferUsage::WRITE)
    ///  - The returned future must be awaited _after_ calling device.poll() to resolve it.
    ///  - The command buffer created by `encoder` must **not** be submitted to a queue before this future is awaited.
    ///
    /// Example:
    ///
    /// ```ignore
    /// let buffer = AutomatedBuffer::new(..);
    ///
    /// let map_write = buffer.write_to_buffer(&device, &mut encoder, &data);
    /// device.poll(...); // must happen before await
    ///
    /// let mapping = map_write.await; // Calling await will write to the mapping
    ///
    /// queue.submit(&[encoder.submit()]); // must happen after await
    /// ```
    pub fn write_to_buffer<'a>(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        data: &'a [u8],
    ) -> impl Future<Output = ()> + 'a {
        assert!(
            self.usage.contains(AutomatedBufferUsage::WRITE),
            "Must have usage WRITE to write to buffer. Current usage {:?}",
            self.usage
        );
        match self.style {
            UploadStyle::Mapping => Self::map_write(data, Some(self.inner.map_write(0, data.len() as BufferAddress))),
            UploadStyle::Staging => {
                let staging = device.create_buffer_with_data(data, BufferUsage::COPY_SRC);
                encoder.copy_buffer_to_buffer(&staging, 0, &self.inner, 0, data.len() as BufferAddress);
                Self::map_write(data, None)
            }
        }
    }
}

impl Deref for AutomatedBuffer {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
