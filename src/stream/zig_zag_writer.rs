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
    encode_infallible_unchecked,
    write_all,
};
use qubit_codec_binary::{
    NonStrict,
    ZigZagCodec,
};
use qubit_io::{
    Output,
    Seekable,
};

/// Writer wrapper for canonical ZigZag + unsigned LEB128 integers.
///
/// # Target-width integers
///
/// `isize` methods use the current Rust target's pointer width. Prefer
/// fixed-width integer methods such as `write_i64` for persistent files and
/// cross-platform protocols.
pub struct ZigZagWriter<W> {
    inner: W,
    buffer: [u8; 19],
}

impl<W> ZigZagWriter<W> {
    /// Creates a ZigZag writer.
    #[must_use]
    #[inline]
    pub const fn new(inner: W) -> Self {
        Self {
            inner,
            buffer: [0; 19],
        }
    }

    /// Returns a shared reference to the underlying writer.
    #[must_use]
    #[inline]
    pub const fn inner(&self) -> &W {
        &self.inner
    }

    /// Returns an exclusive reference to the underlying writer.
    #[must_use]
    #[inline]
    pub fn inner_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    /// Consumes this wrapper and returns the underlying writer.
    #[must_use]
    #[inline]
    pub fn into_inner(self) -> W {
        self.inner
    }
}

macro_rules! impl_write_value {
    ($method:ident, $ty:ty, $doc:literal) => {
        #[doc = $doc]
        #[inline]
        pub fn $method(&mut self, value: $ty) -> Result<()> {
            type Codec = ZigZagCodec<$ty, NonStrict>;

            self.write_zig_zag::<$ty, { Codec::MAX_UNITS_PER_VALUE }, _>(
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
    #[inline]
    fn write_zig_zag<T, const N: usize, F>(
        &mut self,
        value: T,
        encode: F,
    ) -> Result<()>
    where
        F: FnOnce(&mut [u8; 19], T) -> usize,
    {
        let len = encode(&mut self.buffer, value);
        write_all(&mut self.inner, &self.buffer[..len])
    }

    impl_write_value!(write_i8, i8, "Writes a ZigZag `i8`.");
    impl_write_value!(write_i16, i16, "Writes a ZigZag `i16`.");
    impl_write_value!(write_i32, i32, "Writes a ZigZag `i32`.");
    impl_write_value!(write_i64, i64, "Writes a ZigZag `i64`.");
    impl_write_value!(write_i128, i128, "Writes a ZigZag `i128`.");
    impl_write_value!(write_isize, isize, "Writes a ZigZag `isize`.");
}

impl<W> Output for ZigZagWriter<W>
where
    W: Output<Item = u8>,
{
    type Item = u8;

    #[inline(always)]
    fn is_buffered(&self) -> bool {
        self.inner.is_buffered()
    }

    #[inline]
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
    /// # Errors
    ///
    /// Returns the I/O error reported by the wrapped writer.
    #[inline]
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
    #[inline]
    fn seek_to(&mut self, position: SeekFrom) -> Result<u64> {
        self.inner.seek_to(position)
    }
}
