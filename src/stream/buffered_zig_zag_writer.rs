// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::{Result, Seek, SeekFrom, Write};

use crate::stream::BufferedEncodeOutputExt;
use crate::util::MIN_CODEC_BUFFER_CAPACITY;
use qubit_codec::BufferedEncodeOutput;
use qubit_codec_binary::{NonStrict, ZigZagCodec};

/// Buffered writer for canonical ZigZag + unsigned LEB128 integers.
///
/// Values are encoded directly into the internal output buffer and flushed to
/// the wrapped writer in larger chunks.
///
/// # Flush contract
///
/// Pending buffered bytes are not flushed from [`Drop`]. Call [`Write::flush`]
/// to guarantee that all bytes reach the wrapped writer. [`Self::inner`] can
/// observe the wrapped writer before pending bytes have been flushed.
///
/// # Target-width integers
///
/// `isize` methods use the current Rust target's pointer width. Prefer
/// fixed-width integer methods such as `write_i64` for persistent files and
/// cross-platform protocols.
pub struct BufferedZigZagWriter<W>
where
    W: Write,
{
    output: BufferedEncodeOutput<W>,
}

impl<W> BufferedZigZagWriter<W>
where
    W: Write,
{
    /// Creates a buffered ZigZag writer with the default buffer capacity.
    #[must_use]
    #[inline]
    pub fn new(inner: W) -> Self {
        Self {
            output: BufferedEncodeOutput::new(inner),
        }
    }

    /// Creates a buffered ZigZag writer with at least `capacity` bytes.
    #[must_use]
    #[inline]
    pub fn with_capacity(inner: W, capacity: usize) -> Self {
        Self {
            output: BufferedEncodeOutput::with_capacity(
                inner,
                capacity.max(MIN_CODEC_BUFFER_CAPACITY),
            ),
        }
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

macro_rules! impl_write_value {
    ($method:ident, $ty:ty, $doc:literal) => {
        #[doc = $doc]
        #[inline]
        pub fn $method(&mut self, value: $ty) -> Result<()> {
            type Codec = ZigZagCodec<$ty, NonStrict>;

            self.output.write_encoded::<Codec>(value)
        }
    };
}

impl<W> BufferedZigZagWriter<W>
where
    W: Write,
{
    impl_write_value!(write_i8, i8, "Writes a ZigZag `i8`.");
    impl_write_value!(write_i16, i16, "Writes a ZigZag `i16`.");
    impl_write_value!(write_i32, i32, "Writes a ZigZag `i32`.");
    impl_write_value!(write_i64, i64, "Writes a ZigZag `i64`.");
    impl_write_value!(write_i128, i128, "Writes a ZigZag `i128`.");
    impl_write_value!(write_isize, isize, "Writes a ZigZag `isize`.");
}

impl<W> Write for BufferedZigZagWriter<W>
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

impl<W> Seek for BufferedZigZagWriter<W>
where
    W: Write + Seek,
{
    /// Flushes pending bytes before seeking the wrapped writer.
    #[inline]
    fn seek(&mut self, position: SeekFrom) -> Result<u64> {
        self.output.seek(position)
    }
}
