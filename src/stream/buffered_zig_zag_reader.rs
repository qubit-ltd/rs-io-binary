// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Buffered reader for ZigZag-encoded integer values.

use core::marker::PhantomData;
use std::collections::TryReserveError;
use std::io::Result;
use std::io::SeekFrom;

use qubit_codec::TranscodeDecodeInput;
use qubit_codec_binary::Leb128DecodePolicy;
use qubit_codec_binary::NonStrict;
use qubit_codec_binary::Strict;
use qubit_codec_binary::ZigZagCodec;
use qubit_io::Buffer;
use qubit_io::Input;
use qubit_io::Seekable;

use crate::util::MIN_CODEC_BUFFER_CAPACITY;
use super::internal::TranscodeDecodeInputExt;

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
///
/// # Type Parameters
///
/// - `R`: Underlying byte input.
/// - `P`: LEB128 canonicality policy applied after ZigZag decoding.
pub struct BufferedZigZagReader<R, P>
where
    R: Input<Item = u8>,
{
    /// Buffered codec input and wrapped reader.
    input: TranscodeDecodeInput<R>,
    /// Associates the selected decoding policy without storing a value.
    marker: PhantomData<fn() -> P>,
}

impl<R, P> BufferedZigZagReader<R, P>
where
    R: Input<Item = u8>,
    P: Leb128DecodePolicy,
{
    /// Creates a buffered ZigZag reader with the default buffer capacity.
    ///
    /// # Parameters
    ///
    /// - `inner`: Underlying byte input.
    ///
    /// # Returns
    ///
    /// Returns a buffered reader using policy `P`.
    #[must_use]
    #[inline]
    pub fn new(inner: R) -> Self {
        Self {
            input: TranscodeDecodeInput::new(inner),
            marker: PhantomData,
        }
    }

    /// Creates a buffered ZigZag reader with at least `capacity` bytes.
    ///
    /// # Parameters
    ///
    /// - `inner`: Underlying byte input.
    /// - `capacity`: Requested internal buffer capacity in bytes.
    ///
    /// # Returns
    ///
    /// Returns a buffered reader whose capacity also satisfies the largest
    /// supported codec payload.
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

    /// Tries to create a buffered ZigZag reader with at least `capacity` bytes.
    ///
    /// # Errors
    ///
    /// Returns an allocation error when the requested buffer cannot be
    /// allocated.
    #[inline]
    pub fn try_with_capacity(
        inner: R,
        capacity: usize,
    ) -> std::result::Result<Self, TryReserveError> {
        Ok(Self {
            input: TranscodeDecodeInput::try_with_capacity(
                inner,
                capacity.max(MIN_CODEC_BUFFER_CAPACITY),
            )?,
            marker: PhantomData,
        })
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
    /// The underlying reader may already be positioned past unread bytes held
    /// in this wrapper's internal buffer.
    ///
    /// # Returns
    ///
    /// Returns the wrapped reader.
    #[must_use]
    #[inline(always)]
    pub const fn inner(&self) -> &R {
        self.input.inner()
    }

    /// Consumes this wrapper and preserves its unread buffered bytes.
    ///
    /// # Returns
    ///
    /// Returns the wrapped reader and the buffer whose [`Buffer::readable`]
    /// slice contains every prefetched byte not yet consumed logically. To
    /// continue the same logical stream, consume that slice before reading
    /// from the returned reader.
    #[inline(always)]
    #[must_use = "the returned inner reader and unread buffer must be handled"]
    pub fn into_parts(self) -> (R, Buffer<u8>) {
        self.input.into_parts()
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

            self.input.read_decoded::<Codec>()
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
        impl<R> BufferedZigZagReader<R, $policy>
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
impl<R, P> Input for BufferedZigZagReader<R, P>
where
    R: Input<Item = u8>,
{
    type Item = u8;

    /// Reports that this adapter buffers input.
    ///
    /// # Returns
    ///
    /// Always returns `true`.
    #[inline(always)]
    fn is_buffered(&self) -> bool {
        true
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
    /// Returns an error reported while reading the buffered input.
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
    /// Returns the seek error reported by the wrapped reader and preserves
    /// buffered bytes when seeking fails.
    #[inline(always)]
    fn seek_to(&mut self, position: SeekFrom) -> Result<u64> {
        self.input.seek(position)
    }
}
