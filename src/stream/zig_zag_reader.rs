// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::marker::PhantomData;
use std::io::{
    Result,
    SeekFrom,
};

use crate::util::read_leb128_from_reader;
use qubit_codec_binary::{
    Leb128DecodePolicy,
    NonStrict,
    Strict,
    ZigZagCodec,
};
use qubit_io::{
    Input,
    Seekable,
};

/// Reader wrapper for ZigZag + unsigned LEB128 integers.
///
/// # Target-width integers
///
/// `isize` methods use the current Rust target's pointer width. Prefer
/// fixed-width integer methods such as `read_i64` for persistent files and
/// cross-platform protocols.
pub struct ZigZagReader<R, P = NonStrict> {
    inner: R,
    buffer: [u8; 19],
    marker: PhantomData<fn() -> P>,
}

impl<R, P> ZigZagReader<R, P>
where
    P: Leb128DecodePolicy,
{
    /// Creates a ZigZag reader.
    #[must_use]
    #[inline]
    pub const fn new(inner: R) -> Self {
        Self {
            inner,
            buffer: [0; 19],
            marker: PhantomData,
        }
    }

    /// Returns whether this reader rejects non-canonical LEB128 encodings.
    #[must_use]
    #[inline]
    pub const fn is_strict(&self) -> bool {
        P::STRICT
    }

    /// Returns a shared reference to the underlying reader.
    #[must_use]
    #[inline]
    pub const fn inner(&self) -> &R {
        &self.inner
    }

    /// Returns an exclusive reference to the underlying reader.
    #[must_use]
    #[inline]
    pub fn inner_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Consumes this wrapper and returns the underlying reader.
    #[must_use]
    #[inline]
    pub fn into_inner(self) -> R {
        self.inner
    }
}

macro_rules! impl_read_value {
    ($policy:ty, $method:ident, $ty:ty, $doc:literal) => {
        #[doc = $doc]
        #[inline]
        pub fn $method(&mut self) -> Result<$ty> {
            type Codec = ZigZagCodec<$ty, $policy>;

            read_leb128_from_reader::<{ Codec::MAX_UNITS_PER_VALUE }, Codec, _>(
                &mut self.inner,
                &mut self.buffer,
            )
        }
    };
}

macro_rules! impl_for_policy {
    ($policy:ty) => {
        impl<R> ZigZagReader<R, $policy>
        where
            R: Input<Item = u8>,
        {
            impl_read_value!($policy, read_i8, i8, "Reads a ZigZag `i8`.");
            impl_read_value!($policy, read_i16, i16, "Reads a ZigZag `i16`.");
            impl_read_value!($policy, read_i32, i32, "Reads a ZigZag `i32`.");
            impl_read_value!($policy, read_i64, i64, "Reads a ZigZag `i64`.");
            impl_read_value!(
                $policy,
                read_i128,
                i128,
                "Reads a ZigZag `i128`."
            );
            impl_read_value!(
                $policy,
                read_isize,
                isize,
                "Reads a ZigZag `isize`."
            );
        }
    };
}

impl_for_policy!(NonStrict);
impl_for_policy!(Strict);

impl<R, P> Input for ZigZagReader<R, P>
where
    R: Input<Item = u8>,
{
    type Item = u8;

    #[inline(always)]
    fn is_buffered(&self) -> bool {
        self.inner.is_buffered()
    }

    #[inline]
    unsafe fn read_unchecked(
        &mut self,
        output: &mut [u8],
        index: usize,
        count: usize,
    ) -> Result<usize> {
        // SAFETY: The caller upholds the wrapped input's range contract.
        unsafe { self.inner.read_unchecked(output, index, count) }
    }
}

impl<R, P> Seekable for ZigZagReader<R, P>
where
    R: Seekable<Unit = u8>,
{
    type Unit = u8;

    /// Seeks the wrapped reader.
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
    /// Returns the seek error reported by the wrapped reader.
    #[inline]
    fn seek_to(&mut self, position: SeekFrom) -> Result<u64> {
        self.inner.seek_to(position)
    }
}
