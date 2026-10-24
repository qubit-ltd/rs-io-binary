// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Buffered writer for fixed-width binary values.

use core::marker::PhantomData;
use std::collections::TryReserveError;
use std::io::Result;
use std::io::SeekFrom;

use qubit_codec::BigEndian;
use qubit_codec::ByteOrder;
use qubit_codec::ByteOrderSpec;
use qubit_codec::LittleEndian;
use qubit_codec::NativeEndian;
use qubit_codec::TranscodeEncodeOutput;
use qubit_codec_binary::BinaryCodec;
use qubit_io::BufferedOutputRecovery;
use qubit_io::ByteOutput;
use qubit_io::Output;
use qubit_io::SeekableByteOutput;
use qubit_io::Seekable;

use super::internal::TranscodeEncodeOutputExt;
use crate::util::MIN_CODEC_BUFFER_CAPACITY;
use crate::util::checked_u16_len;
use crate::util::checked_u32_len;
use crate::util::write_all;

/// Buffered writer for fixed-width binary values.
///
/// Scalar writes encode directly into the internal output buffer and flush that
/// buffer to the wrapped writer only when it becomes full or when explicitly
/// flushed.
///
/// # Flush contract
///
/// Dropping this writer delegates to `qubit_io::BufferedOutput`, which makes a
/// best-effort attempt to drain pending bytes, ignores drop-time errors, and
/// does not guarantee that the wrapped writer itself is flushed. Call
/// [`Output::flush`] to guarantee that all bytes reach the wrapped output.
/// [`Self::inner`] can observe the wrapped writer before pending bytes have
/// been flushed.
///
/// # Type Parameters
///
/// - `W`: Underlying byte output.
/// - `O`: Compile-time byte-order specification.
///
/// # Examples
///
/// ```
/// use qubit_codec::BigEndian;
/// use qubit_io_binary::BufferedBinaryWriter;
///
/// let mut writer = BufferedBinaryWriter::<_, BigEndian>::new(Vec::new());
/// writer.write_u16(0x1234)?;
/// writer.flush()?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct BufferedBinaryWriter<W, O = BigEndian>
where
    W: ByteOutput,
{
    /// Buffered codec output and wrapped writer.
    output: TranscodeEncodeOutput<W>,
    /// Associates the selected byte order without storing a value.
    marker: PhantomData<fn() -> O>,
}

impl_output_methods!(BufferedBinaryWriter<W, O> where W: ByteOutput);
impl_seekable_methods!(BufferedBinaryWriter<W, O> where W: SeekableByteOutput);

impl<W, O> BufferedBinaryWriter<W, O>
where
    W: ByteOutput,
    O: ByteOrderSpec,
{
    /// Creates a buffered binary writer with the default buffer capacity.
    ///
    /// # Parameters
    ///
    /// - `inner`: Underlying byte output.
    ///
    /// # Returns
    ///
    /// Returns a buffered writer using byte order `O`.
    #[must_use]
    #[inline]
    pub fn new(inner: W) -> Self {
        Self {
            output: TranscodeEncodeOutput::new(inner),
            marker: PhantomData,
        }
    }

    /// Creates a buffered binary writer with at least `capacity` bytes.
    ///
    /// # Parameters
    ///
    /// - `inner`: Underlying byte output.
    /// - `capacity`: Requested internal buffer capacity in bytes.
    ///
    /// # Returns
    ///
    /// Returns a buffered writer whose capacity also satisfies the largest
    /// supported codec payload.
    #[must_use]
    #[inline]
    pub fn with_capacity(inner: W, capacity: usize) -> Self {
        Self {
            output: TranscodeEncodeOutput::with_capacity(inner, capacity.max(MIN_CODEC_BUFFER_CAPACITY)),
            marker: PhantomData,
        }
    }

    /// Tries to create a buffered binary writer with at least `capacity` bytes.
    ///
    /// # Errors
    ///
    /// Returns an allocation error when the requested buffer cannot be
    /// allocated.
    #[inline]
    pub fn try_with_capacity(inner: W, capacity: usize) -> std::result::Result<Self, TryReserveError> {
        Ok(Self {
            output: TranscodeEncodeOutput::try_with_capacity(inner, capacity.max(MIN_CODEC_BUFFER_CAPACITY))?,
            marker: PhantomData,
        })
    }

    /// Returns the byte order selected by this writer.
    ///
    /// # Returns
    ///
    /// Returns the compile-time byte order as a runtime value.
    #[must_use]
    #[inline(always)]
    pub const fn byte_order(&self) -> ByteOrder {
        O::ORDER
    }

    /// Returns a shared reference to the underlying writer.
    ///
    /// Pending bytes may still be held in this wrapper's internal buffer.
    ///
    /// # Returns
    ///
    /// Returns the wrapped writer.
    #[must_use]
    #[inline(always)]
    pub const fn inner(&self) -> &W {
        self.output.inner()
    }
    /// Recovers the wrapped writer and the status of bytes retained by this writer.
    ///
    /// This method performs no I/O and does not call [`Self::flush`]. `Pending`
    /// bytes were not reported as written and may be retried. `Indeterminate`
    /// means an inner write panicked and may have produced an external prefix;
    /// do not replay it automatically. Flush first for normal completion.
    ///
    /// # Returns
    ///
    /// Returns the wrapped writer and pending or indeterminate bytes in logical
    /// write order.
    #[must_use = "the wrapped writer and recovery status must be handled"]
    #[inline(always)]
    pub fn into_parts(self) -> (W, BufferedOutputRecovery<u8>) {
        self.output.into_parts()
    }
}

macro_rules! impl_value_write {
    ($order:ty, $method:ident, $ty:ty, $doc:literal) => {
        #[doc = $doc]
        #[doc = ""]
        #[doc = "# Parameters"]
        #[doc = ""]
        #[doc = "- `value`: Scalar to encode and buffer."]
        #[doc = ""]
        #[doc = "# Returns"]
        #[doc = ""]
        #[doc = "Returns after the encoded bytes have been accepted."]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns an output error encountered while making buffer \
                 space."]
        #[inline(always)]
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
            W: ByteOutput,
        {
            impl_value_write!($order, write_u8, u8, "Writes an unsigned 8-bit integer.");
            impl_value_write!($order, write_i8, i8, "Writes a signed 8-bit integer.");
            impl_value_write!($order, write_u16, u16, "Writes an unsigned 16-bit integer.");
            impl_value_write!($order, write_u32, u32, "Writes an unsigned 32-bit integer.");
            impl_value_write!($order, write_u64, u64, "Writes an unsigned 64-bit integer.");
            impl_value_write!($order, write_u128, u128, "Writes an unsigned 128-bit integer.");
            impl_value_write!($order, write_i16, i16, "Writes a signed 16-bit integer.");
            impl_value_write!($order, write_i32, i32, "Writes a signed 32-bit integer.");
            impl_value_write!($order, write_i64, i64, "Writes a signed 64-bit integer.");
            impl_value_write!($order, write_i128, i128, "Writes a signed 128-bit integer.");
            impl_value_write!($order, write_f32, f32, "Writes a 32-bit float.");
            impl_value_write!($order, write_f64, f64, "Writes a 64-bit float.");

            /// Writes a UTF-8 string prefixed by a `u16` byte length.
            ///
            /// # Errors
            ///
            /// Returns [`std::io::ErrorKind::InvalidInput`] when the UTF-8
            /// byte length does not fit `u16`, or an output error.
            pub fn write_string_with_u16_len(&mut self, value: &str) -> Result<()> {
                self.write_u16(checked_u16_len(value.len())?)?;
                write_all(self, value.as_bytes())
            }

            /// Writes a UTF-8 string prefixed by a `u32` byte length.
            ///
            /// # Errors
            ///
            /// Returns [`std::io::ErrorKind::InvalidInput`] when the UTF-8
            /// byte length does not fit `u32`, or an output error.
            pub fn write_string_with_u32_len(&mut self, value: &str) -> Result<()> {
                self.write_u32(checked_u32_len(value.len())?)?;
                write_all(self, value.as_bytes())
            }
        }
    };
}

impl_for_order!(BigEndian);
impl_for_order!(LittleEndian);
impl_for_order!(NativeEndian);

impl<W, O> Output for BufferedBinaryWriter<W, O>
where
    W: ByteOutput,
{
    type Item = u8;

    /// Reports that this adapter buffers output.
    ///
    /// # Returns
    ///
    /// Always returns `true`.
    #[inline(always)]
    fn is_buffered(&self) -> bool {
        true
    }

    /// Writes up to `count` bytes from an indexed input range.
    ///
    /// # Parameters
    ///
    /// - `input`: Source slice.
    /// - `index`: First source index.
    /// - `count`: Maximum number of bytes to write.
    ///
    /// # Returns
    ///
    /// Returns the number of bytes accepted.
    ///
    /// # Errors
    ///
    /// Returns an output error encountered while draining pending bytes.
    ///
    /// # Safety
    ///
    /// `index..index + count` must be a valid range within `input`.
    #[inline(always)]
    unsafe fn write_unchecked(&mut self, input: &[u8], index: usize, count: usize) -> Result<usize> {
        // SAFETY: The caller upholds the indexed source range contract.
        unsafe { self.output.write_unchecked(input, index, count) }
    }

    /// Flushes the internal buffer and then the wrapped writer.
    ///
    /// # Returns
    ///
    /// Returns after buffered and wrapped output have flushed.
    ///
    /// # Errors
    ///
    /// Returns an error encountered while draining or flushing.
    #[inline(always)]
    fn flush(&mut self) -> Result<()> {
        self.output.flush()
    }
}

impl<W, O> Seekable for BufferedBinaryWriter<W, O>
where
    W: SeekableByteOutput,
{
    type Unit = u8;

    /// Flushes pending bytes before seeking the wrapped writer.
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
    /// Returns an error encountered while flushing or seeking.
    #[inline(always)]
    fn seek_to(&mut self, position: SeekFrom) -> Result<u64> {
        self.output.seek(position)
    }
}
