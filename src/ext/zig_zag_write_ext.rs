// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Extension methods for encoding ZigZag-encoded integers into streams.

use std::io::Result;

use crate::util::{
    encode_infallible_unchecked,
    write_all,
};
use qubit_codec_binary::{
    NonStrict,
    ZigZagCodec,
};
use qubit_io::Output;

macro_rules! write_zig_zag_value {
    ($writer:expr, $value:expr, $ty:ty) => {
        write_zig_zag::<
            { ZigZagCodec::<$ty, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE },
            _,
            _,
            _,
        >($writer, $value, |bytes, value| {
            type Codec = ZigZagCodec<$ty, NonStrict>;
            // SAFETY: The local buffer is exactly the codec's maximum buffer
            // length.
            unsafe { encode_infallible_unchecked::<Codec>(value, bytes, 0) }
        })
    };
}

/// Extension methods for writing ZigZag + unsigned LEB128 integers.
///
/// # Target-width integers
///
/// `isize` methods use the current Rust target's pointer width. Prefer
/// fixed-width integer methods such as [`Self::write_zig_zag_i64`] for
/// persistent files and cross-platform protocols.
pub trait ZigZagWriteExt: Output<Item = u8> {
    /// Writes a ZigZag `i8`.
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
    fn write_zig_zag_i8(&mut self, value: i8) -> Result<()> {
        write_zig_zag_value!(self, value, i8)
    }

    /// Writes a ZigZag `i16`.
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
    fn write_zig_zag_i16(&mut self, value: i16) -> Result<()> {
        write_zig_zag_value!(self, value, i16)
    }

    /// Writes a ZigZag `i32`.
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
    fn write_zig_zag_i32(&mut self, value: i32) -> Result<()> {
        write_zig_zag_value!(self, value, i32)
    }

    /// Writes a ZigZag `i64`.
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
    fn write_zig_zag_i64(&mut self, value: i64) -> Result<()> {
        write_zig_zag_value!(self, value, i64)
    }

    /// Writes a ZigZag `i128`.
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
    fn write_zig_zag_i128(&mut self, value: i128) -> Result<()> {
        write_zig_zag_value!(self, value, i128)
    }

    /// Writes a ZigZag `isize`.
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
    fn write_zig_zag_isize(&mut self, value: isize) -> Result<()> {
        write_zig_zag_value!(self, value, isize)
    }
}

impl<W> ZigZagWriteExt for W where W: Output<Item = u8> + ?Sized {}

/// Encodes and writes one canonical ZigZag value.
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
fn write_zig_zag<const N: usize, T, W, F>(
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
