// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Extension methods for encoding LEB128 values into byte streams.

use std::io::Result;

use crate::util::{
    encode_infallible_unchecked,
    write_all,
};
use qubit_codec_binary::{
    Leb128Codec,
    NonStrict,
};
use qubit_io::Output;

macro_rules! write_leb128_value {
    ($writer:expr, $value:expr, $ty:ty) => {
        write_leb128::<
            { Leb128Codec::<$ty, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE },
            _,
            _,
            _,
        >($writer, $value, |bytes, value| {
            type Codec = Leb128Codec<$ty, NonStrict>;
            // SAFETY: The local buffer is exactly the codec's maximum buffer
            // length.
            unsafe { encode_infallible_unchecked::<Codec>(value, bytes, 0) }
        })
    };
}

/// Extension methods for writing canonical LEB128 integers to byte streams.
///
/// # Target-width integers
///
/// `usize` and `isize` methods use the current Rust target's pointer width.
/// Prefer fixed-width integer methods such as [`Self::write_uleb_u64`] or
/// [`Self::write_sleb_i64`] for persistent files and cross-platform protocols.
pub trait Leb128WriteExt: Output<Item = u8> {
    /// Writes an unsigned LEB128 `u8`.
    ///
    /// # Parameters
    ///
    /// - `value`: Integer to encode and write.
    ///
    /// # Returns
    ///
    /// Returns after the canonical payload has been written.
    ///
    /// # Errors
    ///
    /// Returns an output error, including a write-zero error when the output
    /// stops making progress.
    #[inline(always)]
    fn write_uleb_u8(&mut self, value: u8) -> Result<()> {
        write_leb128_value!(self, value, u8)
    }

    /// Writes an unsigned LEB128 `u16`.
    ///
    /// # Parameters
    ///
    /// - `value`: Integer to encode and write.
    ///
    /// # Returns
    ///
    /// Returns after the canonical payload has been written.
    ///
    /// # Errors
    ///
    /// Returns an output error, including a write-zero error when the output
    /// stops making progress.
    #[inline(always)]
    fn write_uleb_u16(&mut self, value: u16) -> Result<()> {
        write_leb128_value!(self, value, u16)
    }

    /// Writes an unsigned LEB128 `u32`.
    ///
    /// # Parameters
    ///
    /// - `value`: Integer to encode and write.
    ///
    /// # Returns
    ///
    /// Returns after the canonical payload has been written.
    ///
    /// # Errors
    ///
    /// Returns an output error, including a write-zero error when the output
    /// stops making progress.
    #[inline(always)]
    fn write_uleb_u32(&mut self, value: u32) -> Result<()> {
        write_leb128_value!(self, value, u32)
    }

    /// Writes an unsigned LEB128 `u64`.
    ///
    /// # Parameters
    ///
    /// - `value`: Integer to encode and write.
    ///
    /// # Returns
    ///
    /// Returns after the canonical payload has been written.
    ///
    /// # Errors
    ///
    /// Returns an output error, including a write-zero error when the output
    /// stops making progress.
    #[inline(always)]
    fn write_uleb_u64(&mut self, value: u64) -> Result<()> {
        write_leb128_value!(self, value, u64)
    }

    /// Writes an unsigned LEB128 `u128`.
    ///
    /// # Parameters
    ///
    /// - `value`: Integer to encode and write.
    ///
    /// # Returns
    ///
    /// Returns after the canonical payload has been written.
    ///
    /// # Errors
    ///
    /// Returns an output error, including a write-zero error when the output
    /// stops making progress.
    #[inline(always)]
    fn write_uleb_u128(&mut self, value: u128) -> Result<()> {
        write_leb128_value!(self, value, u128)
    }

    /// Writes an unsigned LEB128 `usize`.
    ///
    /// # Parameters
    ///
    /// - `value`: Integer to encode and write.
    ///
    /// # Returns
    ///
    /// Returns after the canonical payload has been written.
    ///
    /// # Errors
    ///
    /// Returns an output error, including a write-zero error when the output
    /// stops making progress.
    #[inline(always)]
    fn write_uleb_usize(&mut self, value: usize) -> Result<()> {
        write_leb128_value!(self, value, usize)
    }

    /// Writes a signed LEB128 `i8`.
    ///
    /// # Parameters
    ///
    /// - `value`: Integer to encode and write.
    ///
    /// # Returns
    ///
    /// Returns after the canonical payload has been written.
    ///
    /// # Errors
    ///
    /// Returns an output error, including a write-zero error when the output
    /// stops making progress.
    #[inline(always)]
    fn write_sleb_i8(&mut self, value: i8) -> Result<()> {
        write_leb128_value!(self, value, i8)
    }

    /// Writes a signed LEB128 `i16`.
    ///
    /// # Parameters
    ///
    /// - `value`: Integer to encode and write.
    ///
    /// # Returns
    ///
    /// Returns after the canonical payload has been written.
    ///
    /// # Errors
    ///
    /// Returns an output error, including a write-zero error when the output
    /// stops making progress.
    #[inline(always)]
    fn write_sleb_i16(&mut self, value: i16) -> Result<()> {
        write_leb128_value!(self, value, i16)
    }

    /// Writes a signed LEB128 `i32`.
    ///
    /// # Parameters
    ///
    /// - `value`: Integer to encode and write.
    ///
    /// # Returns
    ///
    /// Returns after the canonical payload has been written.
    ///
    /// # Errors
    ///
    /// Returns an output error, including a write-zero error when the output
    /// stops making progress.
    #[inline(always)]
    fn write_sleb_i32(&mut self, value: i32) -> Result<()> {
        write_leb128_value!(self, value, i32)
    }

    /// Writes a signed LEB128 `i64`.
    ///
    /// # Parameters
    ///
    /// - `value`: Integer to encode and write.
    ///
    /// # Returns
    ///
    /// Returns after the canonical payload has been written.
    ///
    /// # Errors
    ///
    /// Returns an output error, including a write-zero error when the output
    /// stops making progress.
    #[inline(always)]
    fn write_sleb_i64(&mut self, value: i64) -> Result<()> {
        write_leb128_value!(self, value, i64)
    }

    /// Writes a signed LEB128 `i128`.
    ///
    /// # Parameters
    ///
    /// - `value`: Integer to encode and write.
    ///
    /// # Returns
    ///
    /// Returns after the canonical payload has been written.
    ///
    /// # Errors
    ///
    /// Returns an output error, including a write-zero error when the output
    /// stops making progress.
    #[inline(always)]
    fn write_sleb_i128(&mut self, value: i128) -> Result<()> {
        write_leb128_value!(self, value, i128)
    }

    /// Writes a signed LEB128 `isize`.
    ///
    /// # Parameters
    ///
    /// - `value`: Integer to encode and write.
    ///
    /// # Returns
    ///
    /// Returns after the canonical payload has been written.
    ///
    /// # Errors
    ///
    /// Returns an output error, including a write-zero error when the output
    /// stops making progress.
    #[inline(always)]
    fn write_sleb_isize(&mut self, value: isize) -> Result<()> {
        write_leb128_value!(self, value, isize)
    }
}

impl<W> Leb128WriteExt for W where W: Output<Item = u8> + ?Sized {}

/// Encodes and writes one canonical LEB128 value.
///
/// # Type Parameters
///
/// - `N`: Maximum encoded payload length in bytes.
/// - `T`: Value type accepted by the encoder.
/// - `W`: Destination byte output.
/// - `F`: Infallible encoding callback.
///
/// # Parameters
///
/// - `writer`: Destination for the encoded payload.
/// - `value`: Value passed to `encode`.
/// - `encode`: Callback that fills the local buffer and returns its used
///   length.
///
/// # Returns
///
/// Returns after the complete payload has been written.
///
/// # Errors
///
/// Returns an output error, including a write-zero error when the output stops
/// making progress.
#[inline]
fn write_leb128<const N: usize, T, W, F>(
    writer: &mut W,
    value: T,
    encode: F,
) -> Result<()>
where
    W: Output<Item = u8> + ?Sized,
    F: FnOnce(&mut [u8], T) -> usize,
{
    let mut bytes = [0u8; N];
    let len = encode(&mut bytes, value);
    write_all(writer, &bytes[..len])
}
