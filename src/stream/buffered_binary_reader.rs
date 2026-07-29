// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::marker::PhantomData;
use std::io::{Result, SeekFrom};

use crate::util::MIN_CODEC_BUFFER_CAPACITY;
use qubit_codec::{BigEndian, ByteOrder, ByteOrderSpec, LittleEndian, TranscodeDecodeInput};
use qubit_codec_binary::BinaryCodec;
use qubit_io::{Buffer, Input, Seekable};

use super::internal::TranscodeDecodeInputExt;

/// Buffered reader for fixed-width binary values.
///
/// Scalar reads decode directly from the internal input buffer whenever enough
/// bytes are available, avoiding a per-value temporary byte buffer.
///
/// # Buffered state
///
/// This reader may prefetch bytes from the wrapped reader. As a result,
/// [`Self::inner`] can observe an underlying stream position ahead of the
/// logical position exposed by this wrapper.
///
/// # Type Parameters
///
/// - `R`: Underlying byte input.
/// - `O`: Compile-time byte-order specification.
pub struct BufferedBinaryReader<R, O = BigEndian>
where
    R: Input<Item = u8>,
{
    /// Buffered codec input and wrapped reader.
    input: TranscodeDecodeInput<R>,
    /// Associates the selected byte order without storing a value.
    marker: PhantomData<fn() -> O>,
}

impl<R, O> BufferedBinaryReader<R, O>
where
    R: Input<Item = u8>,
    O: ByteOrderSpec,
{
    /// Creates a buffered binary reader with the default buffer capacity.
    ///
    /// # Parameters
    ///
    /// - `inner`: Underlying byte input.
    ///
    /// # Returns
    ///
    /// Returns a buffered reader using byte order `O`.
    #[must_use]
    #[inline]
    pub fn new(inner: R) -> Self {
        Self {
            input: TranscodeDecodeInput::new(inner),
            marker: PhantomData,
        }
    }

    /// Creates a buffered binary reader with at least `capacity` bytes.
    ///
    /// # Parameters
    ///
    /// - `inner`: Underlying byte input.
    /// - `capacity`: Requested internal buffer capacity in bytes.
    ///
    /// # Returns
    ///
    /// Returns a buffered reader whose capacity also satisfies the largest
    /// supported codec payload.
    #[must_use]
    #[inline]
    pub fn with_capacity(inner: R, capacity: usize) -> Self {
        Self {
            input: TranscodeDecodeInput::with_capacity(
                inner,
                capacity.max(MIN_CODEC_BUFFER_CAPACITY),
            ),
            marker: PhantomData,
        }
    }

    /// Returns the byte order selected by this reader.
    ///
    /// # Returns
    ///
    /// Returns the compile-time byte order as a runtime value.
    #[must_use]
    #[inline(always)]
    pub const fn byte_order(&self) -> ByteOrder {
        O::ORDER
    }

    /// Returns a shared reference to the underlying reader.
    ///
    /// The underlying reader may already be positioned past unread bytes held
    /// in this wrapper's internal buffer.
    ///
    /// # Returns
    ///
    /// Returns the wrapped reader.
    #[must_use]
    #[inline(always)]
    pub const fn inner(&self) -> &R {
        self.input.inner()
    }

    /// Returns mutable access to the underlying reader.
    ///
    /// Direct reads from the returned reader bypass unread bytes already held
    /// by this wrapper and can desynchronize subsequent buffered reads. Use
    /// [`Self::into_parts`] when ownership and unread bytes must be recovered
    /// together.
    ///
    /// # Returns
    ///
    /// Consumes this wrapper and preserves its unread buffered bytes.
    ///
    /// # Returns
    ///
    /// Returns the wrapped reader and the buffer whose [`Buffer::readable`]
    /// slice contains every prefetched byte not yet consumed logically. To
    /// continue the same logical stream, consume that slice before reading
    /// from the returned reader.
    #[inline(always)]
    #[must_use = "the returned inner reader and unread buffer must be handled"]
    pub fn into_parts(self) -> (R, Buffer<u8>) {
        self.input.into_parts()
    }
}

macro_rules! impl_value_read {
    ($order:ty, $method:ident, $ty:ty, $doc:literal) => {
        #[doc = $doc]
        #[doc = ""]
        #[doc = "# Returns"]
        #[doc = ""]
        #[doc = concat!("Returns the decoded `", stringify!($ty), "`.")]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns an input error, including an unexpected-end-of-input \
                 error when the scalar is truncated."]
        #[inline(always)]
        pub fn $method(&mut self) -> Result<$ty> {
            type Codec = BinaryCodec<$ty, $order>;
            self.input.read_decoded::<Codec>()
        }
    };
}

macro_rules! impl_for_order {
    ($order:ty) => {
        impl<R> BufferedBinaryReader<R, $order>
        where
            R: Input<Item = u8>,
        {
            impl_value_read!($order, read_u8, u8, "Reads an unsigned 8-bit integer.");
            impl_value_read!($order, read_i8, i8, "Reads a signed 8-bit integer.");
            impl_value_read!($order, read_u16, u16, "Reads an unsigned 16-bit integer.");
            impl_value_read!($order, read_u32, u32, "Reads an unsigned 32-bit integer.");
            impl_value_read!($order, read_u64, u64, "Reads an unsigned 64-bit integer.");
            impl_value_read!(
                $order,
                read_u128,
                u128,
                "Reads an unsigned 128-bit integer."
            );
            impl_value_read!($order, read_i16, i16, "Reads a signed 16-bit integer.");
            impl_value_read!($order, read_i32, i32, "Reads a signed 32-bit integer.");
            impl_value_read!($order, read_i64, i64, "Reads a signed 64-bit integer.");
            impl_value_read!($order, read_i128, i128, "Reads a signed 128-bit integer.");
            impl_value_read!($order, read_f32, f32, "Reads a 32-bit float.");
            impl_value_read!($order, read_f64, f64, "Reads a 64-bit float.");
        }
    };
}

impl_for_order!(BigEndian);
impl_for_order!(LittleEndian);

impl<R, O> Input for BufferedBinaryReader<R, O>
where
    R: Input<Item = u8>,
{
    type Item = u8;

    /// Reports that this adapter buffers input.
    ///
    /// # Returns
    ///
    /// Always returns `true`.
    #[inline(always)]
    fn is_buffered(&self) -> bool {
        true
    }

    /// Reads up to `count` bytes into an indexed output range.
    ///
    /// # Parameters
    ///
    /// - `output`: Destination slice.
    /// - `index`: First destination index.
    /// - `count`: Maximum number of bytes to read.
    ///
    /// # Returns
    ///
    /// Returns the number of bytes read.
    ///
    /// # Errors
    ///
    /// Returns an error reported while reading the buffered input.
    ///
    /// # Safety
    ///
    /// `index..index + count` must be a valid range within `output`.
    #[inline(always)]
    unsafe fn read_unchecked(
        &mut self,
        output: &mut [u8],
        index: usize,
        count: usize,
    ) -> Result<usize> {
        // SAFETY: The caller upholds the indexed destination range contract.
        unsafe { self.input.read_unchecked(output, index, count) }
    }
}

impl<R, O> Seekable for BufferedBinaryReader<R, O>
where
    R: Input<Item = u8> + Seekable<Unit = u8>,
{
    type Unit = u8;

    /// Seeks the wrapped reader and discards buffered bytes after success.
    ///
    /// # Parameters
    ///
    /// - `position`: Target seek position.
    ///
    /// # Returns
    ///
    /// Returns the new physical stream position.
    ///
    /// # Errors
    ///
    /// Returns the seek error reported by the wrapped reader and preserves
    /// buffered bytes when seeking fails.
    #[inline(always)]
    fn seek_to(&mut self, position: SeekFrom) -> Result<u64> {
        self.input.seek(position)
    }
}
