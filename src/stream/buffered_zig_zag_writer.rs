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

use crate::stream::TranscodeEncodeOutputExt;
use crate::util::MIN_CODEC_BUFFER_CAPACITY;
use qubit_codec::TranscodeEncodeOutput;
use qubit_codec_binary::{
    NonStrict,
    ZigZagCodec,
};
use qubit_io::{
    Output,
    Seekable,
};

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
pub struct BufferedZigZagWriter<W>
where
    W: Output<Item = u8>,
{
    output: TranscodeEncodeOutput<W>,
}

impl<W> BufferedZigZagWriter<W>
where
    W: Output<Item = u8>,
{
    /// Creates a buffered ZigZag writer with the default buffer capacity.
    #[must_use]
    #[inline]
    pub fn new(inner: W) -> Self {
        Self {
            output: TranscodeEncodeOutput::new(inner),
        }
    }

    /// Creates a buffered ZigZag writer with at least `capacity` bytes.
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
    W: Output<Item = u8>,
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
    W: Output<Item = u8>,
{
    type Item = u8;

    #[inline(always)]
    fn is_buffered(&self) -> bool {
        true
    }

    /// Writes bytes through the internal buffer.
    #[inline]
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
    #[inline]
    fn flush(&mut self) -> Result<()> {
        self.output.flush()
    }
}

impl<W> Seekable for BufferedZigZagWriter<W>
where
    W: Output<Item = u8> + Seekable<Unit = u8>,
{
    type Unit = u8;

    /// Flushes pending bytes before seeking the wrapped writer.
    #[inline]
    fn seek_to(&mut self, position: SeekFrom) -> Result<u64> {
        self.output.seek(position)
    }
}
