// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::{
    Result,
    SeekFrom,
};

use crate::util::{
    MIN_CODEC_BUFFER_CAPACITY,
    checked_u64_len,
    encode_infallible_unchecked,
    write_all,
};
use qubit_codec_binary::{
    Leb128Codec,
    NonStrict,
};
use qubit_io::{
    Output,
    Seekable,
};

/// Writer wrapper for canonical LEB128 integers.
///
/// # Target-width integers
///
/// `usize` and `isize` methods use the current Rust target's pointer width.
/// Prefer fixed-width integer methods such as `write_u64` or `write_i64` for
/// persistent files and cross-platform protocols.
///
/// # Type Parameters
///
/// - `W`: Underlying byte output.
pub struct Leb128Writer<W> {
    /// Wrapped byte output.
    inner: W,
    /// Scratch storage for the largest LEB128 payload.
    buffer: [u8; MIN_CODEC_BUFFER_CAPACITY],
}

impl<W> Leb128Writer<W> {
    /// Creates a LEB128 writer.
    ///
    /// # Parameters
    ///
    /// - `inner`: Underlying byte output.
    ///
    /// # Returns
    ///
    /// Returns a canonical LEB128 writer.
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
        #[doc = "- `value`: Integer to encode and write."]
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
            type Codec = Leb128Codec<$ty, NonStrict>;

            self.write_leb128::<$ty, { Codec::MAX_ENCODE_UNITS_PER_VALUE }, _>(
                value,
                |bytes, value| unsafe {
                    encode_infallible_unchecked::<Codec>(value, bytes, 0)
                },
            )
        }
    };
}

impl<W> Leb128Writer<W>
where
    W: Output<Item = u8>,
{
    impl_write_value!(write_u8, u8, "Writes an unsigned LEB128 `u8`.");
    impl_write_value!(write_u16, u16, "Writes an unsigned LEB128 `u16`.");
    impl_write_value!(write_u32, u32, "Writes an unsigned LEB128 `u32`.");
    impl_write_value!(write_u64, u64, "Writes an unsigned LEB128 `u64`.");
    impl_write_value!(write_u128, u128, "Writes an unsigned LEB128 `u128`.");
    impl_write_value!(write_usize, usize, "Writes an unsigned LEB128 `usize`.");
    impl_write_value!(write_i8, i8, "Writes a signed LEB128 `i8`.");
    impl_write_value!(write_i16, i16, "Writes a signed LEB128 `i16`.");
    impl_write_value!(write_i32, i32, "Writes a signed LEB128 `i32`.");
    impl_write_value!(write_i64, i64, "Writes a signed LEB128 `i64`.");
    impl_write_value!(write_i128, i128, "Writes a signed LEB128 `i128`.");
    impl_write_value!(write_isize, isize, "Writes a signed LEB128 `isize`.");

    /// Writes a UTF-8 string prefixed by an unsigned LEB128 byte length.
    ///
    /// The length prefix is encoded as `usize`, so this format is target-width
    /// dependent. Prefer a fixed-width length prefix for persistent files and
    /// cross-platform protocols.
    ///
    /// # Parameters
    ///
    /// - `value`: String slice to write.
    ///
    /// # Returns
    ///
    /// Returns after the length and payload have been written.
    ///
    /// # Errors
    ///
    /// Returns an I/O error from the underlying writer.
    #[inline]
    pub fn write_utf8_string_usize(&mut self, value: &str) -> Result<()> {
        self.write_usize(value.len())?;
        let bytes = value.as_bytes();
        write_all(&mut self.inner, bytes)
    }

    /// Writes a UTF-8 string prefixed by an unsigned LEB128 `u64` byte length.
    ///
    /// Prefer this method over [`Self::write_utf8_string_usize`] for persistent files
    /// and cross-platform protocols because the length field is independent of
    /// the current Rust target's pointer width.
    ///
    /// # Parameters
    ///
    /// - `value`: String slice to write.
    ///
    /// # Returns
    ///
    /// Returns after the length and payload have been written.
    ///
    /// # Errors
    ///
    /// Returns [`std::io::ErrorKind::InvalidInput`] when the UTF-8 byte length
    /// cannot be represented as `u64`, or an I/O error from the underlying
    /// writer.
    #[inline]
    pub fn write_utf8_string_u64(&mut self, value: &str) -> Result<()> {
        self.write_u64(checked_u64_len(value.len())?)?;
        let bytes = value.as_bytes();
        write_all(&mut self.inner, bytes)
    }

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
    fn write_leb128<T, const N: usize, F>(
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

impl<W> Output for Leb128Writer<W>
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

impl<W> Seekable for Leb128Writer<W>
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
