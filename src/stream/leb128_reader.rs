/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use core::marker::PhantomData;
use std::io::{
    Read,
    Result,
    Seek,
    SeekFrom,
};

use crate::util::{
    read_leb128_from_reader,
    read_utf8_payload,
    usize_from_u64_len,
};
use qubit_codec_binary::{
    Leb128Codec,
    Leb128DecodePolicy,
    NonStrict,
    Strict,
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
pub struct Leb128Reader<R, P = NonStrict> {
    inner: R,
    buffer: [u8; 19],
    marker: PhantomData<fn() -> P>,
}

impl<R, P> Leb128Reader<R, P>
where
    P: Leb128DecodePolicy,
{
    /// Creates a LEB128 reader.
    #[must_use]
    #[inline]
    pub const fn new(inner: R) -> Self {
        Self {
            inner,
            buffer: [0; 19],
            marker: PhantomData,
        }
    }

    /// Returns whether this reader rejects non-canonical encodings.
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
            type Codec = Leb128Codec<$ty, $policy>;

            read_leb128_from_reader::<{ Codec::MAX_UNITS_PER_VALUE }, Codec, _>(&mut self.inner, &mut self.buffer)
        }
    };
}

macro_rules! impl_for_policy {
    ($policy:ty) => {
        impl<R> Leb128Reader<R, $policy>
        where
            R: Read,
        {
            impl_read_value!($policy, read_u8, u8, "Reads an unsigned LEB128 `u8`.");
            impl_read_value!($policy, read_u16, u16, "Reads an unsigned LEB128 `u16`.");
            impl_read_value!($policy, read_u32, u32, "Reads an unsigned LEB128 `u32`.");
            impl_read_value!($policy, read_u64, u64, "Reads an unsigned LEB128 `u64`.");
            impl_read_value!($policy, read_u128, u128, "Reads an unsigned LEB128 `u128`.");
            impl_read_value!($policy, read_usize, usize, "Reads an unsigned LEB128 `usize`.");
            impl_read_value!($policy, read_i8, i8, "Reads a signed LEB128 `i8`.");
            impl_read_value!($policy, read_i16, i16, "Reads a signed LEB128 `i16`.");
            impl_read_value!($policy, read_i32, i32, "Reads a signed LEB128 `i32`.");
            impl_read_value!($policy, read_i64, i64, "Reads a signed LEB128 `i64`.");
            impl_read_value!($policy, read_i128, i128, "Reads a signed LEB128 `i128`.");
            impl_read_value!($policy, read_isize, isize, "Reads a signed LEB128 `isize`.");

            /// Reads a UTF-8 string prefixed by an unsigned LEB128 byte length.
            ///
            /// The length prefix is decoded as `usize`, so this format is
            /// target-width dependent. Prefer a fixed-width length prefix for
            /// persistent files and cross-platform protocols.
            ///
            /// # Parameters
            ///
            /// - `max_len`: Maximum accepted UTF-8 payload length in bytes.
            ///
            /// # Returns
            ///
            /// Returns the decoded UTF-8 string.
            ///
            /// # Errors
            ///
            /// Returns an I/O error for length or payload reads, [`std::io::ErrorKind::InvalidData`]
            /// when the encoded length exceeds `max_len`, or [`std::io::ErrorKind::InvalidData`]
            /// when the payload is not valid UTF-8.
            #[inline]
            pub fn read_utf8_string(&mut self, max_len: usize) -> Result<String> {
                let len = self.read_usize()?;
                read_utf8_payload(&mut self.inner, len, max_len)
            }

            /// Reads a UTF-8 string prefixed by an unsigned LEB128 `u64` byte length.
            ///
            /// Prefer this method over [`Self::read_utf8_string`] for persistent
            /// files and cross-platform protocols because the length field is
            /// independent of the current Rust target's pointer width.
            #[inline]
            pub fn read_utf8_string_u64(&mut self, max_len: usize) -> Result<String> {
                let len = usize_from_u64_len(self.read_u64()?)?;
                read_utf8_payload(&mut self.inner, len, max_len)
            }
        }
    };
}

impl_for_policy!(NonStrict);
impl_for_policy!(Strict);

impl<R, P> Read for Leb128Reader<R, P>
where
    R: Read,
{
    /// Reads bytes from the wrapped reader.
    ///
    /// # Parameters
    ///
    /// - `buffer`: Destination byte buffer.
    ///
    /// # Returns
    ///
    /// Returns the number of bytes read.
    ///
    /// # Errors
    ///
    /// Returns the I/O error reported by the wrapped reader.
    #[inline]
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        self.inner.read(buffer)
    }
}

impl<R, P> Seek for Leb128Reader<R, P>
where
    R: Seek,
{
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
    fn seek(&mut self, position: SeekFrom) -> Result<u64> {
        self.inner.seek(position)
    }
}
