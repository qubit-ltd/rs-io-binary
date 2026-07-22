// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow source-test-pair
//! Shared asynchronous byte-stream codec drivers.

use std::io::Result;
use std::pin::Pin;

use qubit_codec::Codec;
use qubit_codec_binary::Leb128DecodeError;
use qubit_io::{
    AsyncInput,
    AsyncOutput,
    ReadExactFuture,
    WriteFullyFuture,
};

use super::streams::{
    decode_leb128_unchecked,
    invalid_utf8_error,
    length_exceeded_error,
    map_leb128_decode_error,
};
use super::try_reserve_vec;

/// Reads exactly enough bytes to fill `output`.
pub(crate) async fn read_exact_async<I>(
    input: &mut I,
    output: &mut [u8],
) -> Result<()>
where
    I: AsyncInput<Item = u8> + Unpin + ?Sized,
{
    ReadExactFuture::new(Pin::new(input), output).await
}

/// Writes every byte in `input`.
pub(crate) async fn write_all_async<O>(
    output: &mut O,
    input: &[u8],
) -> Result<()>
where
    O: AsyncOutput<Item = u8> + Unpin + ?Sized,
{
    WriteFullyFuture::new(Pin::new(output), input).await
}

/// Reads and decodes one LEB128-family payload.
pub(crate) async fn read_leb128_payload_async<const N: usize, C, R>(
    reader: &mut R,
) -> Result<C::Value>
where
    R: AsyncInput<Item = u8> + Unpin + ?Sized,
    C: Codec<Unit = u8, DecodeError = Leb128DecodeError> + Default,
{
    let mut bytes = [0_u8; N];
    for index in 0..N {
        read_exact_async(reader, &mut bytes[index..=index]).await?;
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
pub(crate) async fn read_utf8_payload_async<R>(
    reader: &mut R,
    len: usize,
    max_len: usize,
) -> Result<String>
where
    R: AsyncInput<Item = u8> + Unpin + ?Sized,
{
    if len > max_len {
        return Err(length_exceeded_error(len, max_len));
    }
    let mut bytes = Vec::new();
    try_reserve_vec(&mut bytes, len)?;
    bytes.resize(len, 0);
    read_exact_async(reader, &mut bytes).await?;
    String::from_utf8(bytes).map_err(invalid_utf8_error)
}
