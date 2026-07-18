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

use crate::stream::TranscodeDecodeInputExt;
use crate::util::MIN_CODEC_BUFFER_CAPACITY;
use qubit_codec::TranscodeDecodeInput;
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

/// Buffered reader for ZigZag + unsigned LEB128 integers.
///
/// Values are decoded directly from the internal input buffer while the codec
/// scans for the underlying LEB128 terminating byte.
///
/// # Buffered state
///
/// This reader may prefetch bytes from the wrapped reader. As a result,
/// [`Self::inner`] can observe an underlying stream position ahead of the
/// logical position exposed by this wrapper.
///
/// # Target-width integers
///
/// `isize` methods use the current Rust target's pointer width. Prefer
/// fixed-width integer methods such as `read_i64` for persistent files and
/// cross-platform protocols.
pub struct BufferedZigZagReader<R, P = NonStrict>
where
    R: Input<Item = u8>,
{
    input: TranscodeDecodeInput<R>,
    marker: PhantomData<fn() -> P>,
}

impl<R, P> BufferedZigZagReader<R, P>
where
    R: Input<Item = u8>,
    P: Leb128DecodePolicy,
{
    /// Creates a buffered ZigZag reader with the default buffer capacity.
    #[must_use]
    #[inline]
    pub fn new(inner: R) -> Self {
        Self {
            input: TranscodeDecodeInput::new(inner),
            marker: PhantomData,
        }
    }

    /// Creates a buffered ZigZag reader with at least `capacity` bytes.
    #[must_use]
    #[inline]
    pub fn with_capacity(inner: R, capacity: usize) -> Self {
        Self {
            input: TranscodeDecodeInput::with_capacity(
                inner,
                capacity.max(MIN_CODEC_BUFFER_CAPACITY),
            ),
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
    ///
    /// The underlying reader may already be positioned past unread bytes held
    /// in this wrapper's internal buffer.
    #[must_use]
    #[inline]
    pub const fn inner(&self) -> &R {
        self.input.inner()
    }
}

macro_rules! impl_read_value {
    ($policy:ty, $method:ident, $ty:ty, $doc:literal) => {
        #[doc = $doc]
        #[inline]
        pub fn $method(&mut self) -> Result<$ty> {
            type Codec = ZigZagCodec<$ty, $policy>;

            self.input.read_decoded::<Codec>()
        }
    };
}

macro_rules! impl_for_policy {
    ($policy:ty) => {
        impl<R> BufferedZigZagReader<R, $policy>
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

impl<R, P> Input for BufferedZigZagReader<R, P>
where
    R: Input<Item = u8>,
{
    type Item = u8;

    #[inline(always)]
    fn is_buffered(&self) -> bool {
        true
    }

    /// Reads bytes from the buffered input.
    #[inline]
    unsafe fn read_unchecked(
        &mut self,
        output: &mut [u8],
        index: usize,
        count: usize,
    ) -> Result<usize> {
        // SAFETY: The caller upholds the indexed destination range contract.
        unsafe { self.input.read_unchecked(output, index, count) }
    }
}

impl<R, P> Seekable for BufferedZigZagReader<R, P>
where
    R: Input<Item = u8> + Seekable<Unit = u8>,
{
    type Unit = u8;

    /// Seeks the wrapped reader and discards buffered bytes after success.
    #[inline]
    fn seek_to(&mut self, position: SeekFrom) -> Result<u64> {
        self.input.seek(position)
    }
}
