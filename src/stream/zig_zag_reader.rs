// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Reader for ZigZag-encoded integer values.

use core::marker::PhantomData;
use std::io::{
    Result,
    SeekFrom,
};

use crate::util::{
    MIN_CODEC_BUFFER_CAPACITY,
    read_leb128_from_reader,
};
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
///
/// # Type Parameters
///
/// - `R`: Underlying byte input.
/// - `P`: LEB128 canonicality policy applied after ZigZag decoding.
pub struct ZigZagReader<R, P> {
    /// Wrapped byte input.
    inner: R,
    /// Scratch storage for the largest encoded ZigZag payload.
    buffer: [u8; MIN_CODEC_BUFFER_CAPACITY],
    /// Associates the selected decoding policy without storing a value.
    marker: PhantomData<fn() -> P>,
}

impl<R, P> ZigZagReader<R, P>
where
    P: Leb128DecodePolicy,
{
    /// Creates a ZigZag reader.
    ///
    /// # Parameters
    ///
    /// - `inner`: Underlying byte input.
    ///
    /// # Returns
    ///
    /// Returns a reader using policy `P`.
    #[must_use]
    #[inline]
    pub const fn new(inner: R) -> Self {
        Self {
            inner,
            buffer: [0; MIN_CODEC_BUFFER_CAPACITY],
            marker: PhantomData,
        }
    }

    /// Returns whether this reader rejects non-canonical LEB128 encodings.
    ///
    /// # Returns
    ///
    /// Returns `true` when policy `P` requires canonical encodings.
    #[must_use]
    #[inline(always)]
    pub const fn is_strict(&self) -> bool {
        P::STRICT
    }

    /// Returns a shared reference to the underlying reader.
    ///
    /// # Returns
    ///
    /// Returns the wrapped reader.
    #[must_use]
    #[inline(always)]
    pub const fn inner(&self) -> &R {
        &self.inner
    }

    /// Returns an exclusive reference to the underlying reader.
    ///
    /// # Returns
    ///
    /// Returns mutable access to the wrapped reader.
    #[must_use]
    #[inline(always)]
    pub fn inner_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Consumes this wrapper and returns the underlying reader.
    ///
    /// # Returns
    ///
    /// Returns the wrapped reader.
    #[must_use]
    #[inline(always)]
    pub fn into_inner(self) -> R {
        self.inner
    }
}

macro_rules! impl_read_value {
    ($policy:ty, $method:ident, $ty:ty, $doc:literal) => {
        #[doc = $doc]
        #[doc = ""]
        #[doc = "# Returns"]
        #[doc = ""]
        #[doc = concat!("Returns the decoded `", stringify!($ty), "`.")]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns an input error or an invalid-data error when the \
                 payload is malformed or violates the selected policy."]
        #[inline(always)]
        pub fn $method(&mut self) -> Result<$ty> {
            type Codec = ZigZagCodec<$ty, $policy>;

            read_leb128_from_reader::<
                { Codec::MAX_DECODE_UNITS_PER_VALUE },
                Codec,
                _,
            >(&mut self.inner, &mut self.buffer)
        }
    };
}

macro_rules! impl_for_policy {
    (
        $policy:ty,
        {
            $(($method:ident, $ty:ty, $doc:literal)),* $(,)?
        }
        $(,)?
    ) => {
        impl<R> ZigZagReader<R, $policy>
        where
            R: Input<Item = u8>,
        {
            $(
                impl_read_value!($policy, $method, $ty, $doc);
            )*
        }
    };
}

impl_for_policy!(
    NonStrict,
    {
        (read_i8_non_strict, i8, "Reads a ZigZag i8."),
        (read_i16_non_strict, i16, "Reads a ZigZag i16."),
        (read_i32_non_strict, i32, "Reads a ZigZag i32."),
        (read_i64_non_strict, i64, "Reads a ZigZag i64."),
        (read_i128_non_strict, i128, "Reads a ZigZag i128."),
        (read_isize_non_strict, isize, "Reads a ZigZag isize."),
    },
);
impl_for_policy!(
    Strict,
    {
        (read_i8, i8, "Reads a ZigZag i8."),
        (read_i16, i16, "Reads a ZigZag i16."),
        (read_i32, i32, "Reads a ZigZag i32."),
        (read_i64, i64, "Reads a ZigZag i64."),
        (read_i128, i128, "Reads a ZigZag i128."),
        (read_isize, isize, "Reads a ZigZag isize."),
    },
);
impl<R, P> Input for ZigZagReader<R, P>
where
    R: Input<Item = u8>,
{
    type Item = u8;

    /// Reports whether the wrapped input is buffered.
    ///
    /// # Returns
    ///
    /// Returns the wrapped input's buffering state.
    #[inline(always)]
    fn is_buffered(&self) -> bool {
        self.inner.is_buffered()
    }

    /// Reads up to `count` bytes into an indexed output range.
    ///
    /// # Parameters
    ///
    /// - `output`: Destination slice.
    /// - `index`: First destination index.
    /// - `count`: Maximum number of bytes to read.
    ///
    /// # Returns
    ///
    /// Returns the number of bytes read.
    ///
    /// # Errors
    ///
    /// Returns an error reported by the wrapped input.
    ///
    /// # Safety
    ///
    /// `index..index + count` must be a valid range within `output`.
    #[inline(always)]
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
    #[inline(always)]
    fn seek_to(&mut self, position: SeekFrom) -> Result<u64> {
        self.inner.seek_to(position)
    }
}
