// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Reader for LEB128-encoded values.

use core::marker::PhantomData;
use std::io::{
    Result,
    SeekFrom,
};

#[cfg(not(target_pointer_width = "64"))]
use crate::util::usize_from_u64_len;
use crate::util::{
    MIN_CODEC_BUFFER_CAPACITY,
    read_leb128_from_reader,
    read_utf8_payload,
};
use qubit_codec_binary::{
    Leb128Codec,
    Leb128DecodePolicy,
    NonStrict,
    Strict,
};
use qubit_io::{
    Input,
    Seekable,
};

/// Reader wrapper for LEB128 integers.
///
/// The decoding policy is selected by the `P` type parameter. Use
/// `Leb128Reader<R, NonStrict>` for permissive decoding and
/// `Leb128Reader<R, Strict>` for canonical-only decoding.
///
/// # Target-width integers
///
/// `usize` and `isize` methods use the current Rust target's pointer width.
/// Prefer fixed-width integer methods such as `read_u64` or `read_i64` for
/// persistent files and cross-platform protocols.
///
/// # Type Parameters
///
/// - `R`: Underlying byte input.
/// - `P`: LEB128 canonicality policy.
pub struct Leb128Reader<R, P> {
    /// Wrapped byte input.
    inner: R,
    /// Scratch storage for the largest LEB128 payload.
    buffer: [u8; MIN_CODEC_BUFFER_CAPACITY],
    /// Associates the selected decoding policy without storing a value.
    marker: PhantomData<fn() -> P>,
}

impl<R, P> Leb128Reader<R, P>
where
    P: Leb128DecodePolicy,
{
    /// Creates a LEB128 reader.
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

    /// Returns whether this reader rejects non-canonical encodings.
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
            type Codec = Leb128Codec<$ty, $policy>;

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
        $read_usize:ident,
        $read_u64:ident,
        $read_utf8_string_usize:ident,
        $read_utf8_string_u64:ident,
        {
            $(($method:ident, $ty:ty, $doc:literal)),* $(,)?
        }
        $(,)?
    ) => {
        impl<R> Leb128Reader<R, $policy>
        where
            R: Input<Item = u8>,
        {
            $(
                impl_read_value!($policy, $method, $ty, $doc);
            )*

            /// Reads a UTF-8 string prefixed by an unsigned LEB128 byte length.
            ///
            /// The length prefix is decoded as usize, so this format is
            /// target-width dependent. Prefer a fixed-width length prefix for
            /// persistent files and cross-platform protocols.
            ///
            /// # Parameters
            ///
            /// - max_len: Maximum accepted UTF-8 payload length in bytes.
            ///
            /// # Returns
            ///
            /// Returns the decoded UTF-8 string.
            ///
            /// # Errors
            ///
            /// Returns an I/O error for length or payload reads,
            /// InvalidData when the encoded length exceeds max_len, or
            /// InvalidData when the payload is not valid UTF-8.
            #[inline]
            pub fn $read_utf8_string_usize(
                &mut self,
                max_len: usize,
            ) -> Result<String> {
                let len = self.$read_usize()?;
                read_utf8_payload(&mut self.inner, len, max_len)
            }

            /// Reads a UTF-8 string prefixed by an unsigned LEB128 u64 byte
            /// length.
            ///
            /// Prefer this method over the usize variant for persistent files
            /// and cross-platform protocols because the length field is
            /// independent of the current Rust target's pointer width.
            ///
            /// # Parameters
            ///
            /// - max_len: Maximum accepted UTF-8 payload length in bytes.
            ///
            /// # Returns
            ///
            /// Returns the decoded UTF-8 string.
            ///
            /// # Errors
            ///
            /// Returns an input, conversion, or allocation error, or an
            /// invalid-data error for a malformed or excessive length or
            /// invalid UTF-8.
            #[inline]
            pub fn $read_utf8_string_u64(
                &mut self,
                max_len: usize,
            ) -> Result<String> {
                let len = self.$read_u64()?;
                #[cfg(target_pointer_width = "64")]
                let len = len as usize;
                #[cfg(not(target_pointer_width = "64"))]
                let len = usize_from_u64_len(len)?;
                read_utf8_payload(&mut self.inner, len, max_len)
            }
        }
    };
}

impl_for_policy!(
    NonStrict,
    read_usize_non_strict,
    read_u64_non_strict,
    read_utf8_string_usize_non_strict,
    read_utf8_string_u64_non_strict,
    {
        (read_u8_non_strict, u8, "Reads an unsigned LEB128 u8."),
        (read_u16_non_strict, u16, "Reads an unsigned LEB128 u16."),
        (read_u32_non_strict, u32, "Reads an unsigned LEB128 u32."),
        (read_u64_non_strict, u64, "Reads an unsigned LEB128 u64."),
        (read_u128_non_strict, u128, "Reads an unsigned LEB128 u128."),
        (read_usize_non_strict, usize, "Reads an unsigned LEB128 usize."),
        (read_i8_non_strict, i8, "Reads a signed LEB128 i8."),
        (read_i16_non_strict, i16, "Reads a signed LEB128 i16."),
        (read_i32_non_strict, i32, "Reads a signed LEB128 i32."),
        (read_i64_non_strict, i64, "Reads a signed LEB128 i64."),
        (read_i128_non_strict, i128, "Reads a signed LEB128 i128."),
        (read_isize_non_strict, isize, "Reads a signed LEB128 isize."),
    },
);
impl_for_policy!(
    Strict,
    read_usize,
    read_u64,
    read_utf8_string_usize,
    read_utf8_string_u64,
    {
        (read_u8, u8, "Reads an unsigned LEB128 u8."),
        (read_u16, u16, "Reads an unsigned LEB128 u16."),
        (read_u32, u32, "Reads an unsigned LEB128 u32."),
        (read_u64, u64, "Reads an unsigned LEB128 u64."),
        (read_u128, u128, "Reads an unsigned LEB128 u128."),
        (read_usize, usize, "Reads an unsigned LEB128 usize."),
        (read_i8, i8, "Reads a signed LEB128 i8."),
        (read_i16, i16, "Reads a signed LEB128 i16."),
        (read_i32, i32, "Reads a signed LEB128 i32."),
        (read_i64, i64, "Reads a signed LEB128 i64."),
        (read_i128, i128, "Reads a signed LEB128 i128."),
        (read_isize, isize, "Reads a signed LEB128 isize."),
    },
);
impl<R, P> Input for Leb128Reader<R, P>
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

impl<R, P> Seekable for Leb128Reader<R, P>
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
