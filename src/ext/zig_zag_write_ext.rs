// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Extension methods for encoding ZigZag-encoded integers into streams.

use std::io::Result;

use qubit_codec_binary::NonStrict;
use qubit_codec_binary::ZigZagCodec;
use qubit_io::ByteOutput;

use crate::util::encode_infallible_unchecked;
use crate::util::write_all;

/// Encodes a signed integer into a stack buffer and writes its ZigZag payload.
macro_rules! write_zig_zag_value {
    ($writer:expr, $value:expr, $ty:ty) => {
        write_zig_zag::<{ ZigZagCodec::<$ty, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE }, _, _, _>(
            $writer,
            $value,
            |bytes, value| {
                type Codec = ZigZagCodec<$ty, NonStrict>;
                // SAFETY: The local buffer is exactly the codec's maximum buffer
                // length.
                unsafe { encode_infallible_unchecked::<Codec>(value, bytes, 0) }
            },
        )
    };
}

/// Extension methods for writing ZigZag + unsigned LEB128 integers.
///
/// # Target-width integers
///
/// `isize` methods use the current Rust target's pointer width. Prefer
/// fixed-width integer methods such as [`Self::write_zig_zag_i64`] for
/// persistent files and cross-platform protocols.
///
/// # Partial writes
///
/// These methods are not transactional. An output error may occur after the
/// output has accepted a byte prefix of the encoded value, and those bytes are
/// not rolled back. Calling a method again encodes the entire value from the
/// output's current position and may duplicate that prefix. Recovery requires
/// outer framing, a trusted seek or rollback strategy, or a transactional
/// transport. Output acceptance does not imply durable persistence.
///
/// # Examples
///
/// ```
/// use qubit_io_binary::ZigZagWriteExt;
///
/// let mut bytes = Vec::new();
/// bytes.write_zig_zag_i32(-300).expect("value should be written");
/// assert_eq!(bytes, [0xD7, 0x04]);
/// ```
pub trait ZigZagWriteExt: ByteOutput {
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

impl<W> ZigZagWriteExt for W where W: ByteOutput + ?Sized {}

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
fn write_zig_zag<const N: usize, T, W, F>(writer: &mut W, value: T, encode: F) -> Result<()>
where
    W: ByteOutput + ?Sized,
    F: FnOnce(&mut [u8], T) -> usize,
{
    let mut bytes = [0u8; N];
    let len = encode(&mut bytes, value);
    write_all(writer, &bytes[..len])
}
