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
    write_all,
};
use qubit_codec::TranscodeEncodeOutput;
use qubit_codec_binary::{
    Leb128Codec,
    NonStrict,
};
use qubit_io::{
    IntoInnerError,
    Output,
    Seekable,
};

use super::internal::TranscodeEncodeOutputExt;

/// Buffered writer for canonical LEB128 integers.
///
/// Values are encoded directly into the internal output buffer and flushed to
/// the wrapped writer in larger chunks.
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
/// # Target-width integers
///
/// `usize` and `isize` methods use the current Rust target's pointer width.
/// Prefer fixed-width integer methods such as `write_u64` or `write_i64` for
/// persistent files and cross-platform protocols.
///
/// # Type Parameters
///
/// - `W`: Underlying byte output.
pub struct BufferedLeb128Writer<W>
where
    W: Output<Item = u8>,
{
    /// Buffered codec output and wrapped writer.
    output: TranscodeEncodeOutput<W>,
}

impl<W> BufferedLeb128Writer<W>
where
    W: Output<Item = u8>,
{
    /// Creates a buffered LEB128 writer with the default buffer capacity.
    ///
    /// # Parameters
    ///
    /// - `inner`: Underlying byte output.
    ///
    /// # Returns
    ///
    /// Returns a buffered canonical LEB128 writer.
    #[must_use]
    #[inline]
    pub fn new(inner: W) -> Self {
        Self {
            output: TranscodeEncodeOutput::new(inner),
        }
    }

    /// Creates a buffered LEB128 writer with at least `capacity` bytes.
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
            output: TranscodeEncodeOutput::with_capacity(
                inner,
                capacity.max(MIN_CODEC_BUFFER_CAPACITY),
            ),
        }
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

    /// Returns mutable access to the underlying writer.
    ///
    /// Direct writes through the returned writer bypass pending bytes in this
    /// wrapper and can reorder the physical byte stream. Flush this wrapper
    /// before using the returned writer directly.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the wrapped writer.
    #[must_use]
    #[inline(always)]
    pub fn inner_mut(&mut self) -> &mut W {
        self.output.inner_mut()
    }

    /// Flushes pending bytes and returns the underlying writer.
    ///
    /// If flushing fails, the returned [`IntoInnerError`] retains this entire
    /// wrapper, including every byte that remains buffered, so callers can
    /// inspect the error and retry.
    ///
    /// # Returns
    ///
    /// Returns the wrapped writer after a successful flush.
    ///
    /// # Errors
    ///
    /// Returns the I/O error reported while draining or flushing the wrapped
    /// writer together with the retained wrapper.
    #[inline]
    pub fn into_inner(
        mut self,
    ) -> std::result::Result<W, IntoInnerError<Self>> {
        if let Err(error) = self.output.flush() {
            return Err(IntoInnerError::new(error, self));
        }
        let (inner, buffer) = self.output.into_parts();
        debug_assert!(buffer.is_empty(), "flushed writer retained bytes");
        Ok(inner)
    }

    /// Writes a UTF-8 string prefixed by an unsigned LEB128 byte length.
    ///
    /// The length prefix is encoded as `usize`, so this format is target-width
    /// dependent. Prefer a fixed-width length prefix for persistent files and
    /// cross-platform protocols.
    ///
    /// # Parameters
    ///
    /// - `value`: String to length-prefix and buffer.
    ///
    /// # Returns
    ///
    /// Returns after the length and payload have been accepted.
    ///
    /// # Errors
    ///
    /// Returns an output error encountered while making buffer space.
    #[inline]
    pub fn write_utf8_string(&mut self, value: &str) -> Result<()> {
        self.write_usize(value.len())?;
        write_all(&mut self.output, value.as_bytes())
    }

    /// Writes a UTF-8 string prefixed by an unsigned LEB128 `u64` byte length.
    ///
    /// Prefer this method over [`Self::write_utf8_string`] for persistent files
    /// and cross-platform protocols because the length field is independent of
    /// the current Rust target's pointer width.
    ///
    /// # Parameters
    ///
    /// - `value`: String to length-prefix and buffer.
    ///
    /// # Returns
    ///
    /// Returns after the length and payload have been accepted.
    ///
    /// # Errors
    ///
    /// Returns an invalid-input error when the length cannot fit in `u64`, or
    /// an output error encountered while making buffer space.
    #[inline]
    pub fn write_utf8_string_u64(&mut self, value: &str) -> Result<()> {
        self.write_u64(checked_u64_len(value.len())?)?;
        write_all(&mut self.output, value.as_bytes())
    }
}

macro_rules! impl_write_value {
    ($method:ident, $ty:ty, $doc:literal) => {
        #[doc = $doc]
        #[doc = ""]
        #[doc = "# Parameters"]
        #[doc = ""]
        #[doc = "- `value`: Integer to encode and buffer."]
        #[doc = ""]
        #[doc = "# Returns"]
        #[doc = ""]
        #[doc = "Returns after the canonical payload has been accepted."]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns an output error encountered while making buffer \
                 space."]
        #[inline(always)]
        pub fn $method(&mut self, value: $ty) -> Result<()> {
            type Codec = Leb128Codec<$ty, NonStrict>;

            self.output.write_encoded::<Codec>(value)
        }
    };
}

impl<W> BufferedLeb128Writer<W>
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
}

impl<W> Output for BufferedLeb128Writer<W>
where
    W: Output<Item = u8>,
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
    unsafe fn write_unchecked(
        &mut self,
        input: &[u8],
        index: usize,
        count: usize,
    ) -> Result<usize> {
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

impl<W> Seekable for BufferedLeb128Writer<W>
where
    W: Output<Item = u8> + Seekable<Unit = u8>,
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
