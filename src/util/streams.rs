/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
use core::convert::Infallible;
use core::num::NonZeroUsize;
use std::io::{
    Error,
    ErrorKind,
    Read,
    Result,
    Write,
};
use std::string::FromUtf8Error;

use crate::ReadExt;
use qubit_codec_binary::{
    Codec,
    Leb128DecodeError,
};

use super::allocation::try_reserve_vec;

const U32_LENGTH_OVERFLOW: &str = "string length exceeds maximum encodable u32 length";
const U64_LENGTH_OVERFLOW: &str = "string length exceeds maximum encodable u64 length";

/// Result returned by a buffered LEB128 decode attempt.
type AvailableLeb128DecodeResult<T> = std::result::Result<Option<(T, usize)>, (Leb128DecodeError, usize)>;

/// Decodes a value with an infallible byte codec without extra bounds checks.
///
/// # Safety
///
/// The caller must guarantee that `index` is a valid start position in `input`
/// and that at least `C::min_units_per_value()` bytes are readable from it.
#[inline(always)]
pub(crate) unsafe fn decode_infallible_unchecked<C>(input: &[u8], index: usize) -> C::Value
where
    C: Codec<Unit = u8, DecodeError = Infallible> + Default,
{
    let codec = C::default();
    // SAFETY: The caller upholds the unchecked decode contract for `C`.
    match unsafe { Codec::decode_unchecked(&codec, input, index) } {
        Ok((value, _)) => value,
        Err(error) => match error {},
    }
}

/// Encodes a value with an infallible byte codec without extra bounds checks.
///
/// # Safety
///
/// The caller must guarantee that `index` is a valid start position in
/// `output` and that `C::max_units_per_value()` bytes can be written from it.
#[inline(always)]
pub(crate) unsafe fn encode_infallible_unchecked<C>(
    value: C::Value,
    output: &mut [u8],
    index: usize,
) -> usize
where
    C: Codec<Unit = u8, EncodeError = Infallible> + Default,
{
    let codec = C::default();
    // SAFETY: The caller upholds the unchecked encode contract for `C`.
    match unsafe { Codec::encode_unchecked(&codec, &value, output, index) } {
        Ok(written) => written,
        Err(error) => match error {},
    }
}

/// Decodes a LEB128-family value without extra bounds checks.
///
/// # Safety
///
/// The caller must guarantee that `index` is a valid start position in `input`
/// and that at least one byte is readable from it.
#[inline(always)]
pub(crate) unsafe fn decode_leb128_unchecked<C>(
    input: &[u8],
    index: usize,
) -> std::result::Result<(C::Value, NonZeroUsize), Leb128DecodeError>
where
    C: Codec<Unit = u8, DecodeError = Leb128DecodeError> + Default,
{
    let codec = C::default();
    // SAFETY: The caller upholds the unchecked decode contract for `C`.
    unsafe { Codec::decode_unchecked(&codec, input, index) }
}

/// Reads one LEB128-family payload and decodes it.
///
/// # Parameters
///
/// - `reader`: Source reader.
/// # Returns
///
/// Returns the decoded value.
///
/// # Errors
///
/// Returns an I/O error reported by `reader`, or [`ErrorKind::InvalidData`] when
/// the codec rejects the payload.
#[inline]
pub(crate) fn read_leb128_payload<const N: usize, C, R>(reader: &mut R) -> Result<C::Value>
where
    R: Read + ?Sized,
    C: Codec<Unit = u8, DecodeError = Leb128DecodeError> + Default,
{
    let mut bytes = [0u8; N];
    for index in 0..N {
        let target = one_byte_slice(&mut bytes, index);
        reader.read_exact(target)?;
        if bytes[index] & 0x80 == 0 {
            // SAFETY: At least one byte has been read, and decoding starts at 0.
            return unsafe { decode_leb128_unchecked::<C>(&bytes, 0) }
                .map(|(value, _)| value)
                .map_err(map_leb128_decode_error);
        }
    }
    // SAFETY: The fixed payload buffer contains the codec-declared maximum
    // number of readable bytes.
    unsafe { decode_leb128_unchecked::<C>(&bytes, 0) }
        .map(|(value, _)| value)
        .map_err(map_leb128_decode_error)
}

/// Reads one LEB128-family value into a caller-owned fixed buffer.
///
/// # Parameters
///
/// - `reader`: Source reader.
/// - `buffer`: Scratch buffer reused by the stream adapter.
///
/// # Returns
///
/// Returns the decoded value.
///
/// # Errors
///
/// Returns an I/O error reported by `reader`, or [`ErrorKind::InvalidData`] when
/// the codec rejects the payload.
#[inline]
pub(crate) fn read_leb128_from_reader<const N: usize, C, R>(
    reader: &mut R,
    buffer: &mut [u8; 19],
) -> Result<C::Value>
where
    R: Read + ?Sized,
    C: Codec<Unit = u8, DecodeError = Leb128DecodeError> + Default,
{
    debug_assert!(N <= buffer.len(), "LEB128 read length exceeds internal buffer");
    for index in 0..N {
        // SAFETY: `index` is produced by `0..N`, where `N` is a codec-declared
        // length that fits the fixed internal buffer.
        unsafe {
            reader.read_exact_unchecked(buffer, index, 1)?;
        }
        if read_byte(buffer, index) & 0x80 == 0 {
            // SAFETY: At least one byte has been read into `buffer`.
            return unsafe { decode_leb128_unchecked::<C>(buffer, 0) }
                .map(|(value, _)| value)
                .map_err(map_leb128_decode_error);
        }
    }
    // SAFETY: `buffer` contains the codec-declared maximum number of readable
    // bytes for this payload.
    unsafe { decode_leb128_unchecked::<C>(buffer, 0) }
        .map(|(value, _)| value)
        .map_err(map_leb128_decode_error)
}

/// Decodes from bytes currently available in a buffered input window.
///
/// # Parameters
///
/// - `bytes`: Internal buffered input storage.
/// - `index`: Start index of the unread payload.
/// - `available`: Number of readable bytes for this decode attempt.
///
/// # Returns
///
/// Returns `Ok(Some((value, consumed)))` when a complete value is decoded,
/// `Ok(None)` when more input is required, or `Err((error, consumed))` when an
/// invalid payload should be consumed before reporting the error.
#[inline(always)]
pub(crate) fn decode_available_leb128<C>(
    bytes: &[u8],
    index: usize,
    available: usize,
) -> AvailableLeb128DecodeResult<C::Value>
where
    C: Codec<Unit = u8, DecodeError = Leb128DecodeError> + Default,
{
    debug_assert!(available > 0, "decode requires at least one available byte");
    debug_assert!(
        index + available <= bytes.len(),
        "decode range exceeds buffered input"
    );
    // SAFETY: `BufferedInput::read_variable_decoded` guarantees that
    // `index + available` is inside the internal buffer. The shortened slice
    // prevents a variable-width codec from observing stale bytes beyond the
    // currently available payload window.
    let available_bytes = unsafe { core::slice::from_raw_parts(bytes.as_ptr(), index + available) };
    // SAFETY: `available > 0`, and `index` is a valid boundary in
    // `available_bytes`.
    match unsafe { decode_leb128_unchecked::<C>(available_bytes, index) } {
        Ok((value, consumed)) => Ok(Some((value, consumed.get()))),
        Err(error) => match error.consumed() {
            Some(consumed) => Err((error, consumed.get())),
            None => Ok(None),
        },
    }
}

/// Converts a LEB128 decode error into an invalid-data I/O error.
pub(crate) fn map_leb128_decode_error(error: Leb128DecodeError) -> Error {
    Error::new(ErrorKind::InvalidData, error)
}

/// Creates a mutable one-byte slice at `index`.
///
/// # Parameters
///
/// - `bytes`: Fixed-size temporary buffer.
/// - `index`: Byte index inside `bytes`.
///
/// # Returns
///
/// Returns a mutable slice containing exactly `bytes[index]`.
#[inline]
fn one_byte_slice(bytes: &mut [u8], index: usize) -> &mut [u8] {
    // SAFETY: Callers pass an index inside the fixed-size local buffer.
    unsafe { core::slice::from_raw_parts_mut(bytes.as_mut_ptr().add(index), 1) }
}

/// Reads one byte from a fixed LEB128 scratch buffer without an extra bounds check.
#[inline(always)]
fn read_byte(buffer: &[u8; 19], index: usize) -> u8 {
    debug_assert!(index < buffer.len(), "LEB128 read index exceeds internal buffer");
    // SAFETY: Callers only pass indexes already checked against the fixed
    // LEB128 scratch buffer.
    unsafe { *buffer.as_ptr().add(index) }
}

/// Reads a UTF-8 payload after its length has already been decoded.
///
/// # Parameters
///
/// - `reader`: Reader that provides the UTF-8 payload bytes.
/// - `len`: Payload length in bytes.
/// - `max_len`: Maximum accepted payload length in bytes.
///
/// # Returns
///
/// Returns the decoded UTF-8 string.
///
/// # Errors
///
/// Returns [`ErrorKind::InvalidData`] when `len` exceeds `max_len`, an
/// allocation error when reserving the output buffer fails, an I/O error from
/// `reader`, or [`ErrorKind::InvalidData`] when the payload is not valid UTF-8.
pub(crate) fn read_utf8_payload<R>(reader: &mut R, len: usize, max_len: usize) -> Result<String>
where
    R: Read + ?Sized,
{
    if len > max_len {
        return Err(length_exceeded_error(len, max_len));
    }
    let mut bytes = Vec::new();
    try_reserve_vec(&mut bytes, len)?;
    bytes.resize(len, 0);
    reader.read_exact(&mut bytes)?;
    String::from_utf8(bytes).map_err(invalid_utf8_error)
}

/// Writes a UTF-8 payload without a length prefix.
///
/// # Parameters
///
/// - `writer`: Destination writer.
/// - `value`: String slice to write.
///
/// # Errors
///
/// Returns the I/O error reported by `writer`.
pub(crate) fn write_utf8_payload<W>(writer: &mut W, value: &str) -> Result<()>
where
    W: Write + ?Sized,
{
    writer.write_all(value.as_bytes())
}

/// Writes a UTF-8 string after a `u16` byte-length prefix.
pub(crate) fn write_utf8_string_with_u16_len<W, F>(writer: &mut W, value: &str, write_len: F) -> Result<()>
where
    W: Write + ?Sized,
    F: FnOnce(&mut W, u16) -> Result<()>,
{
    let bytes = value.as_bytes();
    write_len(writer, checked_u16_len(bytes.len())?)?;
    writer.write_all(bytes)
}

/// Writes a UTF-8 string after a `u32` byte-length prefix.
pub(crate) fn write_utf8_string_with_u32_len<W, F>(writer: &mut W, value: &str, write_len: F) -> Result<()>
where
    W: Write + ?Sized,
    F: FnOnce(&mut W, u32) -> Result<()>,
{
    let bytes = value.as_bytes();
    write_len(writer, checked_u32_len(bytes.len())?)?;
    writer.write_all(bytes)
}

/// Converts a UTF-8 payload length to a `u16` length prefix value.
pub(crate) fn checked_u16_len(len: usize) -> Result<u16> {
    u16::try_from(len).map_err(|_| {
        Error::new(
            ErrorKind::InvalidInput,
            format!("string length {len} exceeds maximum encodable u16 length"),
        )
    })
}

/// Converts a UTF-8 payload length to a `u32` length prefix value.
pub(crate) fn checked_u32_len(len: usize) -> Result<u32> {
    if len <= u32::MAX as usize {
        Ok(len as u32)
    } else {
        Err(Error::new(ErrorKind::InvalidInput, U32_LENGTH_OVERFLOW))
    }
}

/// Converts a UTF-8 payload length to a `u64` length prefix value.
pub(crate) fn checked_u64_len(len: usize) -> Result<u64> {
    u64::try_from(len).map_err(|_| Error::new(ErrorKind::InvalidInput, U64_LENGTH_OVERFLOW))
}

/// Converts a `u64` length prefix to a local `usize` payload length.
pub(crate) fn usize_from_u64_len(len: u64) -> Result<usize> {
    usize::try_from(len).map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            format!("string length {len} exceeds maximum supported usize length"),
        )
    })
}

/// Builds an invalid-data error for UTF-8 payloads that exceed their limit.
fn length_exceeded_error(len: usize, max_len: usize) -> Error {
    Error::new(
        ErrorKind::InvalidData,
        format!("string length {len} exceeds maximum length of {max_len} bytes"),
    )
}

/// Converts an invalid UTF-8 payload error into an I/O error.
fn invalid_utf8_error(error: FromUtf8Error) -> Error {
    Error::new(
        ErrorKind::InvalidData,
        format!("length-prefixed string is not valid UTF-8: {error}"),
    )
}
