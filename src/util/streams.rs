/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
use std::io::{
    Error,
    ErrorKind,
    Read,
    Result,
    Write,
};
use std::string::FromUtf8Error;

use qubit_codec_binary::Leb128DecodeError;

use super::allocation::try_reserve_vec;

const U32_LENGTH_OVERFLOW: &str = "string length exceeds maximum encodable u32 length";

/// Reads one LEB128 payload and decodes it with the provided callback.
///
/// # Parameters
///
/// - `reader`: Source reader.
/// - `decode`: Callback that decodes bytes into a value and consumed length.
///
/// # Returns
///
/// Returns the decoded value.
///
/// # Errors
///
/// Returns an I/O error reported by `reader`, or [`ErrorKind::InvalidData`] when
/// `decode` rejects the payload.
#[inline]
pub(crate) fn read_leb128_payload<const N: usize, T, R, F>(reader: &mut R, decode: F) -> Result<T>
where
    R: Read + ?Sized,
    F: FnOnce(&[u8]) -> std::result::Result<(T, core::num::NonZeroUsize), Leb128DecodeError>,
{
    let mut bytes = [0u8; N];
    for index in 0..N {
        let target = one_byte_slice(&mut bytes, index);
        reader.read_exact(target)?;
        if bytes[index] & 0x80 == 0 {
            return decode(&bytes)
                .map(|(value, _)| value)
                .map_err(|error| Error::new(ErrorKind::InvalidData, error));
        }
    }
    decode(&bytes)
        .map(|(value, _)| value)
        .map_err(|error| Error::new(ErrorKind::InvalidData, error))
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
