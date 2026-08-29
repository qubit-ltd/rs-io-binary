// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared asynchronous byte-stream codec drivers.

use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::pin::Pin;

use qubit_codec::Codec;
use qubit_codec_binary::Leb128DecodeError;
use qubit_io::AsyncInput;
use qubit_io::AsyncOutput;
use qubit_io::PinnedAsyncInputExt;
use qubit_io::WriteFullyFuture;

use super::streams::decode_leb128_unchecked;
use super::streams::invalid_utf8_error;
use super::streams::length_exceeded_error;
use super::streams::map_leb128_decode_error;
use super::try_reserve_vec;

/// Reads exactly enough bytes to fill `output`.
///
/// # Type Parameters
///
/// - `I`: Runtime-neutral asynchronous byte input.
///
/// # Parameters
///
/// - `input`: Source from which bytes are read.
/// - `output`: Destination slice that must be filled.
///
/// # Returns
///
/// Returns after every destination byte has been initialized.
///
/// # Errors
///
/// Returns an input error, including an unexpected-end-of-input error when the
/// source ends before `output` is full.
///
/// # Cancellation safety
///
/// This operation is not cancellation safe. Dropping it retains bytes already
/// consumed from `input` and modifications already made to `output`.
#[inline(always)]
pub(crate) async fn read_exactly_async<I>(input: &mut I, output: &mut [u8]) -> Result<()>
where
    I: AsyncInput<Item = u8> + Unpin + ?Sized,
{
    Pin::new(input).read_exactly_async(output).await
}

/// Writes every byte in `input`.
///
/// # Type Parameters
///
/// - `O`: Runtime-neutral asynchronous byte output.
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
/// Returns an output error, including a write-zero error when the output stops
/// making progress.
///
/// # Cancellation safety
///
/// This operation is not cancellation safe. Dropping it leaves any
/// already-written prefix in `output`.
#[inline(always)]
pub(crate) async fn write_all_async<O>(output: &mut O, input: &[u8]) -> Result<()>
where
    O: AsyncOutput<Item = u8> + Unpin + ?Sized,
{
    WriteFullyFuture::new(Pin::new(output), input).await
}

/// Reads and decodes one LEB128-family payload.
///
/// # Type Parameters
///
/// - `N`: Maximum encoded payload length in bytes.
/// - `C`: LEB128-family codec used to decode the payload.
/// - `R`: Runtime-neutral asynchronous byte input.
///
/// # Parameters
///
/// - `reader`: Source from which the encoded payload is read.
///
/// # Returns
///
/// Returns the decoded codec value.
///
/// # Errors
///
/// Returns an input error or an invalid-data error when the payload is
/// malformed or incomplete at its maximum width.
///
/// # Cancellation safety
///
/// This operation is not cancellation safe. Dropping it retains any bytes
/// already consumed from `reader`.
pub(crate) async fn read_leb128_payload_async<const N: usize, C, R>(reader: &mut R) -> Result<C::Value>
where
    R: AsyncInput<Item = u8> + Unpin + ?Sized,
    C: Codec<Unit = u8, DecodeError = Leb128DecodeError> + Default,
{
    let mut bytes = [0_u8; N];
    for index in 0..N {
        read_exactly_async(reader, &mut bytes[index..=index]).await?;
        if bytes[index] & 0x80 == 0 {
            // SAFETY: At least one byte has been read and decoding starts at
            // the beginning of the local payload buffer.
            return unsafe { decode_leb128_unchecked::<C>(&bytes, 0) }
                .map(|(value, _)| value)
                .map_err(map_leb128_decode_error);
        }
    }
    // SAFETY: The buffer contains the codec-declared maximum payload length.
    unsafe { decode_leb128_unchecked::<C>(&bytes, 0) }
        .map(|(value, _)| value)
        .map_err(map_leb128_decode_error)
}

/// Reads and validates an already length-delimited UTF-8 payload.
///
/// # Type Parameters
///
/// - `R`: Runtime-neutral asynchronous byte input.
///
/// # Parameters
///
/// - `reader`: Source from which payload bytes are read.
/// - `len`: Encoded payload length in bytes.
/// - `max_len`: Maximum accepted payload length in bytes.
///
/// # Returns
///
/// Returns the decoded UTF-8 string.
///
/// # Errors
///
/// Returns an input or allocation error, or an invalid-data error when `len`
/// exceeds `max_len` or the payload is not valid UTF-8.
///
/// # Cancellation safety
///
/// This operation is not cancellation safe. Dropping it retains bytes already
/// consumed from `reader`.
pub(crate) async fn read_utf8_payload_async<R>(reader: &mut R, len: usize, max_len: usize) -> Result<String>
where
    R: AsyncInput<Item = u8> + Unpin + ?Sized,
{
    let mut bytes = Vec::new();
    read_utf8_payload_bytes_async(reader, &mut bytes, len, max_len).await?;
    String::from_utf8(bytes).map_err(invalid_utf8_error)
}

/// Reads and validates a UTF-8 payload into reusable caller-owned storage.
///
/// # Type Parameters
///
/// - `R`: Runtime-neutral asynchronous byte input.
///
/// # Parameters
///
/// - `reader`: Source from which payload bytes are read.
/// - `bytes`: Reusable destination buffer. It is cleared before a permitted
///   read and contains the received bytes when the read succeeds or when UTF-8
///   validation fails.
/// - `len`: Encoded payload length in bytes.
/// - `max_len`: Maximum accepted payload length in bytes.
///
/// # Returns
///
/// Returns after `bytes` contains one valid UTF-8 payload.
///
/// # Errors
///
/// Returns an input or allocation error, or an invalid-data error when `len`
/// exceeds `max_len` or the payload is not valid UTF-8.
///
/// # Cancellation safety
///
/// This operation is not cancellation safe. Dropping it retains bytes already
/// consumed from `reader` and modifications already made to `bytes`.
pub(crate) async fn read_utf8_payload_into_async<R>(
    reader: &mut R,
    bytes: &mut Vec<u8>,
    len: usize,
    max_len: usize,
) -> Result<()>
where
    R: AsyncInput<Item = u8> + Unpin + ?Sized,
{
    read_utf8_payload_bytes_async(reader, bytes, len, max_len).await?;
    std::str::from_utf8(bytes)
        .map(|_| ())
        .map_err(|error| Error::new(ErrorKind::InvalidData, error))
}

/// Reads a bounded payload into a caller-owned byte buffer.
///
/// The helper performs length validation, fallible capacity reservation, and
/// exact input transfer but leaves UTF-8 validation to its caller.
async fn read_utf8_payload_bytes_async<R>(reader: &mut R, bytes: &mut Vec<u8>, len: usize, max_len: usize) -> Result<()>
where
    R: AsyncInput<Item = u8> + Unpin + ?Sized,
{
    if len > max_len {
        return Err(length_exceeded_error(len, max_len));
    }
    bytes.clear();
    try_reserve_vec(bytes, len)?;
    bytes.resize(len, 0);
    read_exactly_async(reader, bytes).await
}
