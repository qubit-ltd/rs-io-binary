// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Buffered writer for ZigZag-encoded integer values.

use std::collections::TryReserveError;
use std::io::Result;
use std::io::SeekFrom;

use qubit_codec::TranscodeEncodeOutput;
use qubit_codec_binary::NonStrict;
use qubit_codec_binary::ZigZagCodec;
use qubit_io::BufferedOutputRecovery;
use qubit_io::ByteOutput;
use qubit_io::Output;
use qubit_io::Seekable;
use qubit_io::SeekableByteOutput;

use super::internal::TranscodeEncodeOutputExt;
use crate::util::MIN_CODEC_BUFFER_CAPACITY;

/// Buffered writer for canonical ZigZag + unsigned LEB128 integers.
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
/// `isize` methods use the current Rust target's pointer width. Prefer
/// fixed-width integer methods such as `write_i64` for persistent files and
/// cross-platform protocols.
///
/// # Type Parameters
///
/// - `W`: Underlying byte output.
///
/// # Examples
///
/// ```
/// use qubit_io::ByteOutput;
/// use qubit_io_binary::BufferedZigZagWriter;
///
/// let mut writer = BufferedZigZagWriter::new(Vec::new());
/// writer.write_i32(-123)?;
/// writer.flush()?;
/// assert_eq!(writer.inner(), &[0xF5, 0x01]);
/// # Ok::<(), std::io::Error>(())
/// ```
pub struct BufferedZigZagWriter<W>
where
    W: ByteOutput,
{
    /// Buffered codec output and wrapped writer.
    output: TranscodeEncodeOutput<W>,
}

impl_output_methods!(BufferedZigZagWriter<W> where W: ByteOutput);
impl_seekable_methods!(BufferedZigZagWriter<W> where W: SeekableByteOutput);

impl<W> BufferedZigZagWriter<W>
where
    W: ByteOutput,
{
    /// Creates a buffered ZigZag writer with the default buffer capacity.
    ///
    /// # Parameters
    ///
    /// - `inner`: Underlying byte output.
    ///
    /// # Returns
    ///
    /// Returns a buffered canonical ZigZag writer.
    #[must_use]
    #[inline]
    pub fn new(inner: W) -> Self {
        Self {
            output: TranscodeEncodeOutput::new(inner),
        }
    }

    /// Creates a buffered ZigZag writer with at least `capacity` bytes.
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
        }
    }

    /// Tries to create a buffered ZigZag writer with at least `capacity` bytes.
    ///
    /// # Errors
    ///
    /// Returns an allocation error when the requested buffer cannot be
    /// allocated.
    #[inline]
    pub fn try_with_capacity(inner: W, capacity: usize) -> std::result::Result<Self, TryReserveError> {
        Ok(Self {
            output: TranscodeEncodeOutput::try_with_capacity(inner, capacity.max(MIN_CODEC_BUFFER_CAPACITY))?,
        })
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
    /// Recovers the wrapped writer and the status of bytes retained by this
    /// writer.
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

macro_rules! impl_write_value {
    ($method:ident, $ty:ty, $doc:literal) => {
        #[doc = $doc]
        #[doc = ""]
        #[doc = "# Parameters"]
        #[doc = ""]
        #[doc = "- `value`: Signed integer to encode and buffer."]
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
            type Codec = ZigZagCodec<$ty, NonStrict>;

            self.output.write_encoded::<Codec>(value)
        }
    };
}

impl<W> BufferedZigZagWriter<W>
where
    W: ByteOutput,
{
    impl_write_value!(write_i8, i8, "Writes a ZigZag `i8`.");
    impl_write_value!(write_i16, i16, "Writes a ZigZag `i16`.");
    impl_write_value!(write_i32, i32, "Writes a ZigZag `i32`.");
    impl_write_value!(write_i64, i64, "Writes a ZigZag `i64`.");
    impl_write_value!(write_i128, i128, "Writes a ZigZag `i128`.");
    impl_write_value!(write_isize, isize, "Writes a ZigZag `isize`.");
}

impl<W> Output for BufferedZigZagWriter<W>
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

impl<W> Seekable for BufferedZigZagWriter<W>
where
    W: SeekableByteOutput,
{
    type Unit = u8;

    /// Seeks relative to this writer's logical position.
    /// For `SeekFrom::Current(0)`, returns the logical position including
    /// pending bytes without draining them. Other requests drain pending bytes
    /// before repositioning the wrapped output, but do not flush that output.
    /// Call [`Self::flush`] explicitly when buffered data and the wrapped
    /// output must be flushed.
    ///
    /// # Parameters
    ///
    /// - `position`: Target seek position.
    ///
    /// # Returns
    ///
    /// Returns the new logical stream position in bytes.
    ///
    /// # Errors
    ///
    /// Returns an error encountered while draining pending bytes or seeking.
    #[inline(always)]
    fn seek_to(&mut self, position: SeekFrom) -> Result<u64> {
        self.output.seek(position)
    }
}
