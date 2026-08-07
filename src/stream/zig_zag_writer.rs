// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Writer for ZigZag-encoded integer values.

use std::io::Result;
use std::io::SeekFrom;

use qubit_codec_binary::NonStrict;
use qubit_codec_binary::ZigZagCodec;
use qubit_io::Output;
use qubit_io::Seekable;

use crate::util::MIN_CODEC_BUFFER_CAPACITY;
use crate::util::encode_infallible_unchecked;
use crate::util::write_all;

/// Writer wrapper for canonical ZigZag + unsigned LEB128 integers.
///
/// # Target-width integers
///
/// `isize` methods use the current Rust target's pointer width. Prefer
/// fixed-width integer methods such as `write_i64` for persistent files and
/// cross-platform protocols.
///
/// # Type Parameters
///
/// - `W`: Underlying byte output.
pub struct ZigZagWriter<W> {
    /// Wrapped byte output.
    inner: W,
    /// Scratch storage for the largest encoded ZigZag payload.
    buffer: [u8; MIN_CODEC_BUFFER_CAPACITY],
}

impl<W> ZigZagWriter<W> {
    /// Creates a ZigZag writer.
    ///
    /// # Parameters
    ///
    /// - `inner`: Underlying byte output.
    ///
    /// # Returns
    ///
    /// Returns a canonical ZigZag writer.
    #[must_use]
    #[inline]
    pub const fn new(inner: W) -> Self {
        Self {
            inner,
            buffer: [0; MIN_CODEC_BUFFER_CAPACITY],
        }
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

macro_rules! impl_write_value {
    ($method:ident, $ty:ty, $doc:literal) => {
        #[doc = $doc]
        #[doc = ""]
        #[doc = "# Parameters"]
        #[doc = ""]
        #[doc = "- `value`: Signed integer to encode and write."]
        #[doc = ""]
        #[doc = "# Returns"]
        #[doc = ""]
        #[doc = "Returns after the canonical payload has been written."]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns an output error, including a write-zero error when \
                 the output stops making progress."]
        #[inline(always)]
        pub fn $method(&mut self, value: $ty) -> Result<()> {
            type Codec = ZigZagCodec<$ty, NonStrict>;

            self.write_zig_zag::<$ty, { Codec::MAX_ENCODE_UNITS_PER_VALUE }, _>(
                value,
                |bytes, value| unsafe {
                    encode_infallible_unchecked::<Codec>(value, bytes, 0)
                },
            )
        }
    };
}

impl<W> ZigZagWriter<W>
where
    W: Output<Item = u8>,
{
    impl_write_value!(write_i8, i8, "Writes a ZigZag `i8`.");
    impl_write_value!(write_i16, i16, "Writes a ZigZag `i16`.");
    impl_write_value!(write_i32, i32, "Writes a ZigZag `i32`.");
    impl_write_value!(write_i64, i64, "Writes a ZigZag `i64`.");
    impl_write_value!(write_i128, i128, "Writes a ZigZag `i128`.");
    impl_write_value!(write_isize, isize, "Writes a ZigZag `isize`.");

    /// Encodes one value into the scratch buffer and writes its payload.
    ///
    /// # Type Parameters
    ///
    /// - `T`: Value type accepted by the encoder.
    /// - `N`: Maximum payload length declared by the codec.
    /// - `F`: Infallible encoding callback.
    ///
    /// # Parameters
    ///
    /// - `value`: Value passed to the encoder.
    /// - `encode`: Callback that fills the scratch buffer and returns the
    ///   encoded length.
    ///
    /// # Returns
    ///
    /// Returns after the encoded payload has been written.
    ///
    /// # Errors
    ///
    /// Returns an output error, including a write-zero error when the output
    /// stops making progress.
    #[inline]
    fn write_zig_zag<T, const N: usize, F>(
        &mut self,
        value: T,
        encode: F,
    ) -> Result<()>
    where
        F: FnOnce(&mut [u8; MIN_CODEC_BUFFER_CAPACITY], T) -> usize,
    {
        let len = encode(&mut self.buffer, value);
        write_all(&mut self.inner, &self.buffer[..len])
    }
}

impl<W> Output for ZigZagWriter<W>
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
        // SAFETY: The caller upholds the wrapped output's range contract.
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

impl<W> Seekable for ZigZagWriter<W>
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
