// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::marker::PhantomData;
use std::io::{Result, SeekFrom};

use crate::util::{checked_u16_len, checked_u32_len, encode_infallible_unchecked, write_all};
use qubit_codec::{BigEndian, ByteOrder, ByteOrderSpec, LittleEndian};
use qubit_codec_binary::BinaryCodec;
use qubit_io::{Output, Seekable};

/// Writer wrapper for fixed-width binary values.
///
/// The byte order is selected by the `O` type parameter. Use
/// `BinaryWriter<W, BigEndian>` for big-endian data and
/// `BinaryWriter<W, LittleEndian>` for little-endian data.
///
/// # Type Parameters
///
/// - `W`: Underlying byte output.
/// - `O`: Compile-time byte-order specification.
pub struct BinaryWriter<W, O = BigEndian> {
    /// Wrapped byte output.
    inner: W,
    /// Scratch storage for the largest fixed-width scalar.
    buffer: [u8; 16],
    /// Associates the selected byte order without storing a value.
    marker: PhantomData<fn() -> O>,
}

impl<W, O> BinaryWriter<W, O>
where
    O: ByteOrderSpec,
{
    /// Creates a binary writer.
    ///
    /// # Parameters
    ///
    /// - `inner`: Underlying byte writer.
    ///
    /// # Returns
    ///
    /// Returns a writer using the byte order selected by `O`.
    #[must_use]
    #[inline]
    pub const fn new(inner: W) -> Self {
        Self {
            inner,
            buffer: [0; 16],
            marker: PhantomData,
        }
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
    /// # Returns
    ///
    /// Returns the wrapped writer.
    #[must_use]
    #[inline(always)]
    pub const fn inner(&self) -> &W {
        &self.inner
    }

    /// Returns an exclusive reference to the underlying writer.
    ///
    /// # Returns
    ///
    /// Returns mutable access to the wrapped writer.
    #[must_use]
    #[inline(always)]
    pub fn inner_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    /// Consumes this wrapper and returns the underlying writer.
    ///
    /// # Returns
    ///
    /// Returns the wrapped writer.
    #[must_use]
    #[inline(always)]
    pub fn into_inner(self) -> W {
        self.inner
    }
}

macro_rules! impl_value_write {
    ($order:ty, $method:ident, $ty:ty, $doc:literal) => {
        #[doc = $doc]
        #[doc = ""]
        #[doc = "# Parameters"]
        #[doc = ""]
        #[doc = "- `value`: Scalar to encode and write."]
        #[doc = ""]
        #[doc = "# Returns"]
        #[doc = ""]
        #[doc = "Returns after all encoded bytes have been written."]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns an output error, including a write-zero error when \
                 the output stops making progress."]
        #[inline]
        pub fn $method(&mut self, value: $ty) -> Result<()> {
            type Codec = BinaryCodec<$ty, $order>;

            const LEN: usize = Codec::MAX_UNITS_PER_VALUE;
            // SAFETY: `LEN` is declared by the codec and fits the fixed
            // internal buffer.
            unsafe {
                let _ = encode_infallible_unchecked::<Codec>(value, &mut self.buffer, 0);
            }
            write_all(&mut self.inner, &self.buffer[..LEN])
        }
    };
}

macro_rules! impl_for_order {
    ($order:ty) => {
        impl<W> BinaryWriter<W, $order>
        where
            W: Output<Item = u8>,
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

            /// Writes a UTF-8 string prefixed by a 16-bit byte length.
            ///
            /// # Parameters
            ///
            /// - `value`: String to length-prefix and write.
            ///
            /// # Returns
            ///
            /// Returns after the length and payload have been written.
            ///
            /// # Errors
            ///
            /// Returns an invalid-input error when the length exceeds
            /// `u16::MAX`, or an output error while writing.
            #[inline]
            pub fn write_string_with_u16_len(&mut self, value: &str) -> Result<()> {
                self.write_u16(checked_u16_len(value.len())?)?;
                let bytes = value.as_bytes();
                write_all(&mut self.inner, bytes)
            }

            /// Writes a UTF-8 string prefixed by a 32-bit byte length.
            ///
            /// # Parameters
            ///
            /// - `value`: String to length-prefix and write.
            ///
            /// # Returns
            ///
            /// Returns after the length and payload have been written.
            ///
            /// # Errors
            ///
            /// Returns an invalid-input error when the length exceeds
            /// `u32::MAX`, or an output error while writing.
            #[inline]
            pub fn write_string_with_u32_len(&mut self, value: &str) -> Result<()> {
                self.write_u32(checked_u32_len(value.len())?)?;
                let bytes = value.as_bytes();
                write_all(&mut self.inner, bytes)
            }
        }
    };
}

impl_for_order!(BigEndian);
impl_for_order!(LittleEndian);

impl<W, O> Output for BinaryWriter<W, O>
where
    W: Output<Item = u8>,
{
    type Item = u8;

    /// Reports whether the wrapped output is buffered.
    ///
    /// # Returns
    ///
    /// Returns the wrapped output's buffering state.
    #[inline(always)]
    fn is_buffered(&self) -> bool {
        self.inner.is_buffered()
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
    /// Returns the number of bytes written.
    ///
    /// # Errors
    ///
    /// Returns an error reported by the wrapped output.
    ///
    /// # Safety
    ///
    /// `index..index + count` must be a valid range within `input`.
    #[inline(always)]
    unsafe fn write_unchecked(
        &mut self,
        input: &[u8],
        index: usize,
        count: usize,
    ) -> Result<usize> {
        // SAFETY: The caller upholds the same indexed range contract required
        // by the wrapped output.
        unsafe { self.inner.write_unchecked(input, index, count) }
    }

    /// Flushes the wrapped writer.
    ///
    /// # Returns
    ///
    /// Returns after the wrapped output has flushed.
    ///
    /// # Errors
    ///
    /// Returns the I/O error reported by the wrapped writer.
    #[inline(always)]
    fn flush(&mut self) -> Result<()> {
        Output::flush(&mut self.inner)
    }
}

impl<W, O> Seekable for BinaryWriter<W, O>
where
    W: Seekable<Unit = u8>,
{
    type Unit = u8;

    /// Seeks the wrapped writer.
    ///
    /// # Parameters
    ///
    /// - `position`: Target seek position.
    ///
    /// # Returns
    ///
    /// Returns the new stream position.
    ///
    /// # Errors
    ///
    /// Returns the seek error reported by the wrapped writer.
    #[inline(always)]
    fn seek_to(&mut self, position: SeekFrom) -> Result<u64> {
        self.inner.seek_to(position)
    }
}
