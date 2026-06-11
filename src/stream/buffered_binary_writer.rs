// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::marker::PhantomData;
use std::io::{Result, Seek, SeekFrom, Write};

use crate::stream::TranscodeEncodeOutputExt;
use crate::util::MIN_CODEC_BUFFER_CAPACITY;
use qubit_codec::TranscodeEncodeOutput;
use qubit_codec_binary::{BigEndian, BinaryCodec, ByteOrder, ByteOrderSpec, LittleEndian};

/// Buffered writer for fixed-width binary values.
///
/// Scalar writes encode directly into the internal output buffer and flush that
/// buffer to the wrapped writer only when it becomes full or when explicitly
/// flushed.
///
/// # Flush contract
///
/// Pending buffered bytes are not flushed from [`Drop`]. Call [`Write::flush`]
/// to guarantee that all bytes reach the wrapped writer. [`Self::inner`] can
/// observe the wrapped writer before pending bytes have been flushed.
pub struct BufferedBinaryWriter<W, O = BigEndian>
where
    W: Write,
{
    output: TranscodeEncodeOutput<W>,
    marker: PhantomData<fn() -> O>,
}

impl<W, O> BufferedBinaryWriter<W, O>
where
    W: Write,
    O: ByteOrderSpec,
{
    /// Creates a buffered binary writer with the default buffer capacity.
    #[must_use]
    #[inline]
    pub fn new(inner: W) -> Self {
        Self {
            output: TranscodeEncodeOutput::new(inner),
            marker: PhantomData,
        }
    }

    /// Creates a buffered binary writer with at least `capacity` bytes.
    #[must_use]
    #[inline]
    pub fn with_capacity(inner: W, capacity: usize) -> Self {
        Self {
            output: TranscodeEncodeOutput::with_capacity(
                inner,
                capacity.max(MIN_CODEC_BUFFER_CAPACITY),
            ),
            marker: PhantomData,
        }
    }

    /// Returns the byte order selected by this writer.
    #[must_use]
    #[inline]
    pub const fn byte_order(&self) -> ByteOrder {
        O::ORDER
    }

    /// Returns a shared reference to the underlying writer.
    ///
    /// Pending bytes may still be held in this wrapper's internal buffer.
    #[must_use]
    #[inline]
    pub const fn inner(&self) -> &W {
        self.output.inner()
    }
}

macro_rules! impl_value_write {
    ($order:ty, $method:ident, $ty:ty, $doc:literal) => {
        #[doc = $doc]
        #[inline]
        pub fn $method(&mut self, value: $ty) -> Result<()> {
            type Codec = BinaryCodec<$ty, $order>;

            self.output.write_encoded::<Codec>(value)
        }
    };
}

macro_rules! impl_for_order {
    ($order:ty) => {
        impl<W> BufferedBinaryWriter<W, $order>
        where
            W: Write,
        {
            impl_value_write!($order, write_u8, u8, "Writes an unsigned 8-bit integer.");
            impl_value_write!($order, write_i8, i8, "Writes a signed 8-bit integer.");
            impl_value_write!($order, write_u16, u16, "Writes an unsigned 16-bit integer.");
            impl_value_write!($order, write_u32, u32, "Writes an unsigned 32-bit integer.");
            impl_value_write!($order, write_u64, u64, "Writes an unsigned 64-bit integer.");
            impl_value_write!(
                $order,
                write_u128,
                u128,
                "Writes an unsigned 128-bit integer."
            );
            impl_value_write!($order, write_i16, i16, "Writes a signed 16-bit integer.");
            impl_value_write!($order, write_i32, i32, "Writes a signed 32-bit integer.");
            impl_value_write!($order, write_i64, i64, "Writes a signed 64-bit integer.");
            impl_value_write!($order, write_i128, i128, "Writes a signed 128-bit integer.");
            impl_value_write!($order, write_f32, f32, "Writes a 32-bit float.");
            impl_value_write!($order, write_f64, f64, "Writes a 64-bit float.");
        }
    };
}

impl_for_order!(BigEndian);
impl_for_order!(LittleEndian);

impl<W, O> Write for BufferedBinaryWriter<W, O>
where
    W: Write,
{
    /// Writes bytes through the internal buffer.
    #[inline]
    fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        Write::write(&mut self.output, buffer)
    }

    /// Writes all bytes through the internal buffer.
    #[inline]
    fn write_all(&mut self, buffer: &[u8]) -> Result<()> {
        Write::write_all(&mut self.output, buffer)
    }

    /// Flushes the internal buffer and then the wrapped writer.
    #[inline]
    fn flush(&mut self) -> Result<()> {
        Write::flush(&mut self.output)
    }
}

impl<W, O> Seek for BufferedBinaryWriter<W, O>
where
    W: Write + Seek,
{
    /// Flushes pending bytes before seeking the wrapped writer.
    #[inline]
    fn seek(&mut self, position: SeekFrom) -> Result<u64> {
        self.output.seek(position)
    }
}
