// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Low-level bounded reads and writes used by the public stream adapters.

use core::convert::Infallible;
use core::num::NonZeroUsize;
use std::io::{
    Error,
    ErrorKind,
    Result,
};
use std::string::FromUtf8Error;

use qubit_codec::Codec;
use qubit_codec_binary::{
    Leb128Codec,
    Leb128DecodeError,
    NonStrict,
};
use qubit_io::{
    Input,
    Output,
};

use super::try_reserve_vec;

/// Error message used when a UTF-8 byte length does not fit in `u32`.
const U32_LENGTH_OVERFLOW: &str =
    "string length exceeds maximum encodable u32 length";
#[cfg(not(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64",
)))]
/// Error message used when a UTF-8 byte length does not fit in `u64`.
const U64_LENGTH_OVERFLOW: &str =
    "string length exceeds maximum encodable u64 length";
/// Minimum capacity required by the largest scalar codec payload.
pub(crate) const MIN_CODEC_BUFFER_CAPACITY: usize = {
    const ENCODE: usize =
        Leb128Codec::<u128, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE;
    const DECODE: usize =
        Leb128Codec::<u128, NonStrict>::MAX_DECODE_UNITS_PER_VALUE;
    if ENCODE > DECODE { ENCODE } else { DECODE }
};

/// Writes every byte in `input` to a Qubit output.
///
/// # Type Parameters
///
/// - `O`: Destination Qubit output.
///
/// # Parameters
///
/// - `output`: Destination to which bytes are written.
/// - `input`: Source bytes that must be written.
///
/// # Returns
///
/// Returns after every source byte has been written.
///
/// # Errors
///
/// Returns the output error, including [`ErrorKind::WriteZero`] when the
/// output stops making progress.
#[inline(always)]
pub(crate) fn write_all<O>(output: &mut O, input: &[u8]) -> Result<()>
where
    O: Output<Item = u8> + ?Sized,
{
    Output::write_fully(output, input)
}

/// Decodes a value with an infallible byte codec without extra bounds checks.
///
/// # Type Parameters
///
/// - `C`: Infallible byte codec used for decoding.
///
/// # Parameters
///
/// - `input`: Encoded units visible to the codec.
/// - `index`: Decode start position.
///
/// # Returns
///
/// Returns the decoded codec value.
///
/// # Safety
///
/// The caller must guarantee that `index` is a valid start position in `input`
/// and that the available input makes `C` return a complete decoded value.
/// Current callers satisfy this with a fixed-width codec and at least
/// `C::MAX_DECODE_UNITS_PER_VALUE` readable bytes.
///
/// # Panics
///
/// Panics if `C` reports incomplete input despite the caller's completeness
/// guarantee.
#[inline(always)]
pub(crate) unsafe fn decode_infallible_unchecked<C>(
    input: &[u8],
    index: usize,
) -> C::Value
where
    C: Codec<Unit = u8, DecodeError = Infallible> + Default,
{
    let mut codec = C::default();
    // SAFETY: The caller upholds the unchecked decode contract for `C`.
    match unsafe { Codec::decode(&mut codec, input, index) } {
        Ok((value, _)) => value,
        Err(qubit_codec::DecodeFailure::Invalid { source, .. }) => {
            match source {}
        }
        Err(qubit_codec::DecodeFailure::Incomplete { .. }) => {
            unreachable!("infallible codec reported incomplete input")
        }
    }
}

/// Encodes a value with an infallible byte codec without extra bounds checks.
///
/// # Type Parameters
///
/// - `C`: Infallible byte codec used for encoding.
///
/// # Parameters
///
/// - `value`: Codec value to encode.
/// - `output`: Destination slice.
/// - `index`: Encode start position.
///
/// # Returns
///
/// Returns the number of encoded units.
///
/// # Safety
///
/// The caller must guarantee that `index` is a valid start position in
/// `output` and that `C::MAX_ENCODE_UNITS_PER_VALUE` bytes can be written from
/// it.
#[inline(always)]
pub(crate) unsafe fn encode_infallible_unchecked<C>(
    value: C::Value,
    output: &mut [u8],
    index: usize,
) -> usize
where
    C: Codec<Unit = u8, EncodeError = Infallible> + Default,
{
    let mut codec = C::default();
    // SAFETY: The caller upholds the unchecked encode contract for `C`.
    match unsafe { Codec::encode(&mut codec, &value, output, index) } {
        Ok(written) => written,
        Err(error) => match error {},
    }
}

/// Decodes a LEB128-family value without extra bounds checks.
///
/// # Type Parameters
///
/// - `C`: LEB128-family codec used for decoding.
///
/// # Parameters
///
/// - `input`: Encoded units visible to the codec.
/// - `index`: Decode start position.
///
/// # Returns
///
/// Returns the decoded value and consumed unit count.
///
/// # Errors
///
/// Returns the codec's invalid or incomplete LEB128 error.
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
    let mut codec = C::default();
    // SAFETY: The caller upholds the unchecked decode contract for `C`.
    unsafe { Codec::decode(&mut codec, input, index) }.map_err(|failure| {
        match failure {
            qubit_codec::DecodeFailure::Invalid { source, .. } => source,
            qubit_codec::DecodeFailure::Incomplete {
                source: Some(source),
                ..
            } => source,
            qubit_codec::DecodeFailure::Incomplete {
                required_total, ..
            } => Leb128DecodeError::incomplete(
                index,
                required_total,
                input.len().saturating_sub(index),
            ),
        }
    })
}

/// Reads one LEB128-family payload and decodes it.
///
/// # Type Parameters
///
/// - `N`: Maximum encoded payload length in bytes.
/// - `C`: LEB128-family codec used for decoding.
/// - `R`: Source Qubit input.
///
/// # Parameters
///
/// - `reader`: Source reader.
///
/// # Returns
///
/// Returns the decoded value.
///
/// # Errors
///
/// Returns an I/O error reported by `reader`, or [`ErrorKind::InvalidData`]
/// when the codec rejects the payload.
pub(crate) fn read_leb128_payload<const N: usize, C, R>(
    reader: &mut R,
) -> Result<C::Value>
where
    R: Input<Item = u8> + ?Sized,
    C: Codec<Unit = u8, DecodeError = Leb128DecodeError> + Default,
{
    let mut bytes = [0u8; N];
    for index in 0..N {
        Input::read_exactly(reader, &mut bytes[index..=index])?;
        if bytes[index] & 0x80 == 0 {
            // SAFETY: At least one byte has been read, and decoding starts at
            // 0.
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
/// # Type Parameters
///
/// - `N`: Maximum encoded payload length in bytes.
/// - `C`: LEB128-family codec used for decoding.
/// - `R`: Source Qubit input.
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
/// Returns an I/O error reported by `reader`, [`ErrorKind::InvalidInput`] when
/// `N` exceeds the scratch buffer width, or [`ErrorKind::InvalidData`] when the
/// codec rejects the payload.
pub(crate) fn read_leb128_from_reader<const N: usize, C, R>(
    reader: &mut R,
    buffer: &mut [u8; MIN_CODEC_BUFFER_CAPACITY],
) -> Result<C::Value>
where
    R: Input<Item = u8> + ?Sized,
    C: Codec<Unit = u8, DecodeError = Leb128DecodeError> + Default,
{
    let payload = buffer.get_mut(..N).ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidInput,
            "LEB128 codec width exceeds the internal buffer",
        )
    })?;
    for index in 0..N {
        Input::read_exactly(reader, &mut payload[index..=index])?;
        if payload[index] & 0x80 == 0 {
            // SAFETY: At least one byte has been read into `payload`.
            return unsafe { decode_leb128_unchecked::<C>(payload, 0) }
                .map(|(value, _)| value)
                .map_err(map_leb128_decode_error);
        }
    }
    // SAFETY: `payload` contains the codec-declared maximum number of readable
    // bytes for this payload.
    unsafe { decode_leb128_unchecked::<C>(payload, 0) }
        .map(|(value, _)| value)
        .map_err(map_leb128_decode_error)
}

/// Converts a LEB128 decode error into an invalid-data I/O error.
///
/// # Parameters
///
/// - `error`: Codec decode error to wrap.
///
/// # Returns
///
/// Returns an [`ErrorKind::InvalidData`] I/O error retaining `error` as its
/// source.
#[must_use]
#[inline(always)]
pub(crate) fn map_leb128_decode_error(error: Leb128DecodeError) -> Error {
    Error::new(ErrorKind::InvalidData, error)
}

/// Reads a UTF-8 payload after its length has already been decoded.
///
/// # Type Parameters
///
/// - `R`: Source Qubit input.
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
pub(crate) fn read_utf8_payload<R>(
    reader: &mut R,
    len: usize,
    max_len: usize,
) -> Result<String>
where
    R: Input<Item = u8> + ?Sized,
{
    let mut bytes = Vec::new();
    read_utf8_payload_bytes(reader, &mut bytes, len, max_len)?;
    String::from_utf8(bytes).map_err(invalid_utf8_error)
}

/// Reads and validates a UTF-8 payload into reusable caller-owned storage.
///
/// # Type Parameters
///
/// - `R`: Source Qubit input.
///
/// # Parameters
///
/// - `reader`: Reader that provides the UTF-8 payload bytes.
/// - `bytes`: Reusable destination buffer. It is cleared before a permitted
///   read and contains the received bytes when the read succeeds or when UTF-8
///   validation fails.
/// - `len`: Payload length in bytes.
/// - `max_len`: Maximum accepted payload length in bytes.
///
/// # Returns
///
/// Returns after `bytes` contains one valid UTF-8 payload.
///
/// # Errors
///
/// Returns [`ErrorKind::InvalidData`] when `len` exceeds `max_len` or the
/// payload is not valid UTF-8, an allocation error when the buffer cannot be
/// resized, or an I/O error from `reader`.
pub(crate) fn read_utf8_payload_into<R>(
    reader: &mut R,
    bytes: &mut Vec<u8>,
    len: usize,
    max_len: usize,
) -> Result<()>
where
    R: Input<Item = u8> + ?Sized,
{
    read_utf8_payload_bytes(reader, bytes, len, max_len)?;
    std::str::from_utf8(bytes)
        .map(|_| ())
        .map_err(|error| Error::new(ErrorKind::InvalidData, error))
}

/// Reads a bounded payload into a caller-owned byte buffer.
///
/// The helper performs length validation, fallible capacity reservation, and
/// exact input transfer but leaves UTF-8 validation to its caller.
fn read_utf8_payload_bytes<R>(
    reader: &mut R,
    bytes: &mut Vec<u8>,
    len: usize,
    max_len: usize,
) -> Result<()>
where
    R: Input<Item = u8> + ?Sized,
{
    if len > max_len {
        return Err(length_exceeded_error(len, max_len));
    }
    bytes.clear();
    try_reserve_vec(bytes, len)?;
    bytes.resize(len, 0);
    Input::read_exactly(reader, bytes)
}

/// Writes a UTF-8 payload without a length prefix.
///
/// # Type Parameters
///
/// - `W`: Destination Qubit output.
///
/// # Parameters
///
/// - `writer`: Destination writer.
/// - `value`: String slice to write.
///
/// # Returns
///
/// Returns after the entire UTF-8 payload has been written.
///
/// # Errors
///
/// Returns the I/O error reported by `writer`.
#[inline(always)]
pub(crate) fn write_utf8_payload<W>(writer: &mut W, value: &str) -> Result<()>
where
    W: Output<Item = u8> + ?Sized,
{
    write_all(writer, value.as_bytes())
}

/// Writes a UTF-8 string after a `u16` byte-length prefix.
///
/// # Type Parameters
///
/// - `W`: Destination Qubit output.
/// - `F`: Callback that writes the `u16` length prefix.
///
/// # Parameters
///
/// - `writer`: Destination for the prefix and payload.
/// - `value`: String to length-prefix and write.
/// - `write_len`: Callback used to encode and write the byte length.
///
/// # Returns
///
/// Returns after the prefix and payload have been written.
///
/// # Errors
///
/// Returns an invalid-input error when the byte length exceeds `u16::MAX`, or
/// an output error while writing.
#[inline]
pub(crate) fn write_utf8_string_with_u16_len<W, F>(
    writer: &mut W,
    value: &str,
    write_len: F,
) -> Result<()>
where
    W: Output<Item = u8> + ?Sized,
    F: FnOnce(&mut W, u16) -> Result<()>,
{
    let bytes = value.as_bytes();
    write_len(writer, checked_u16_len(bytes.len())?)?;
    write_all(writer, bytes)
}

/// Writes a UTF-8 string after a `u32` byte-length prefix.
///
/// # Type Parameters
///
/// - `W`: Destination Qubit output.
/// - `F`: Callback that writes the `u32` length prefix.
///
/// # Parameters
///
/// - `writer`: Destination for the prefix and payload.
/// - `value`: String to length-prefix and write.
/// - `write_len`: Callback used to encode and write the byte length.
///
/// # Returns
///
/// Returns after the prefix and payload have been written.
///
/// # Errors
///
/// Returns an invalid-input error when the byte length exceeds `u32::MAX`, or
/// an output error while writing.
#[inline]
pub(crate) fn write_utf8_string_with_u32_len<W, F>(
    writer: &mut W,
    value: &str,
    write_len: F,
) -> Result<()>
where
    W: Output<Item = u8> + ?Sized,
    F: FnOnce(&mut W, u32) -> Result<()>,
{
    let bytes = value.as_bytes();
    write_len(writer, checked_u32_len(bytes.len())?)?;
    write_all(writer, bytes)
}

/// Converts a UTF-8 payload length to a `u16` length prefix value.
///
/// # Parameters
///
/// - `len`: Payload length in bytes.
///
/// # Returns
///
/// Returns the representable `u16` length.
///
/// # Errors
///
/// Returns [`ErrorKind::InvalidInput`] when `len` exceeds `u16::MAX`.
#[inline]
pub(crate) fn checked_u16_len(len: usize) -> Result<u16> {
    u16::try_from(len).map_err(|_| {
        Error::new(
            ErrorKind::InvalidInput,
            format!("string length {len} exceeds maximum encodable u16 length"),
        )
    })
}

/// Converts a UTF-8 payload length to a `u32` length prefix value.
///
/// # Parameters
///
/// - `len`: Payload length in bytes.
///
/// # Returns
///
/// Returns the representable `u32` length.
///
/// # Errors
///
/// Returns [`ErrorKind::InvalidInput`] when `len` exceeds `u32::MAX`.
#[inline]
pub(crate) fn checked_u32_len(len: usize) -> Result<u32> {
    if len <= u32::MAX as usize {
        Ok(len as u32)
    } else {
        Err(Error::new(ErrorKind::InvalidInput, U32_LENGTH_OVERFLOW))
    }
}

/// Converts a UTF-8 payload length to a `u64` length prefix value.
///
/// # Parameters
///
/// - `len`: Payload length in bytes.
///
/// # Returns
///
/// Returns the representable `u64` length.
///
/// # Errors
///
/// Returns [`ErrorKind::InvalidInput`] on targets whose `usize` can represent
/// lengths greater than `u64::MAX`.
#[inline]
pub(crate) fn checked_u64_len(len: usize) -> Result<u64> {
    #[cfg(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64",
    ))]
    {
        Ok(len as u64)
    }
    #[cfg(not(any(
        target_pointer_width = "16",
        target_pointer_width = "32",
        target_pointer_width = "64",
    )))]
    {
        u64::try_from(len).map_err(|_| {
            Error::new(ErrorKind::InvalidInput, U64_LENGTH_OVERFLOW)
        })
    }
}

/// Converts a `u32` length prefix to a local `usize` payload length.
///
/// # Parameters
///
/// - `len`: Encoded `u32` payload length.
///
/// # Returns
///
/// Returns the representable local length.
///
/// # Errors
///
/// Returns [`ErrorKind::InvalidData`] when `len` does not fit in `usize`.
#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
#[inline]
pub(crate) fn usize_from_u32_len(len: u32) -> Result<usize> {
    usize::try_from(len).map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            format!(
                "string length {len} exceeds maximum supported usize length"
            ),
        )
    })
}

/// Converts a `u64` length prefix to a local `usize` payload length.
///
/// # Parameters
///
/// - `len`: Encoded `u64` payload length.
///
/// # Returns
///
/// Returns the representable local length.
///
/// # Errors
///
/// Returns [`ErrorKind::InvalidData`] when `len` does not fit in `usize`.
#[cfg(not(target_pointer_width = "64"))]
#[inline]
pub(crate) fn usize_from_u64_len(len: u64) -> Result<usize> {
    usize::try_from(len).map_err(|_| {
        Error::new(
            ErrorKind::InvalidData,
            format!(
                "string length {len} exceeds maximum supported usize length"
            ),
        )
    })
}

/// Builds an invalid-data error for UTF-8 payloads that exceed their limit.
///
/// # Parameters
///
/// - `len`: Encoded payload length.
/// - `max_len`: Configured maximum payload length.
///
/// # Returns
///
/// Returns an [`ErrorKind::InvalidData`] error describing the violated limit.
#[must_use]
#[inline]
pub(crate) fn length_exceeded_error(len: usize, max_len: usize) -> Error {
    Error::new(
        ErrorKind::InvalidData,
        format!(
            "string length {len} exceeds maximum length of {max_len} bytes"
        ),
    )
}

/// Converts an invalid UTF-8 payload error into an I/O error.
///
/// # Parameters
///
/// - `error`: UTF-8 conversion error to describe.
///
/// # Returns
///
/// Returns an [`ErrorKind::InvalidData`] I/O error.
#[must_use]
#[inline]
pub(crate) fn invalid_utf8_error(error: FromUtf8Error) -> Error {
    Error::new(ErrorKind::InvalidData, error)
}
