// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![no_main]

use std::io::{
    self,
    Cursor,
    ErrorKind,
    Read,
};

use libfuzzer_sys::fuzz_target;
use qubit_codec::LittleEndian;
use qubit_codec_binary::{NonStrict, Strict};
use qubit_io_binary::{
    BinaryReadExt,
    BinaryReader,
    BufferedBinaryReader,
    BufferedLeb128Reader,
    BufferedZigZagReader,
    Leb128ReadExt,
    Leb128Reader,
    Leb128WriteExt,
    StringReadExt,
    StringWriteExt,
    ZigZagReadExt,
    ZigZagReader,
};

/// Keeps allocations and each target invocation bounded outside CI.
const MAX_FUZZ_INPUT_LEN: usize = 4096;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_FUZZ_INPUT_LEN)];
    let chunk_size =
        usize::from(data.first().copied().unwrap_or_default() % 8) + 1;
    let payload = data.get(1..).unwrap_or_default();

    fuzz_fixed_width(payload, chunk_size);
    fuzz_leb128(payload, chunk_size);
    fuzz_strict_leb128(payload, chunk_size);
    fuzz_zig_zag(payload, chunk_size);
    fuzz_string(payload, chunk_size);
});

/// Compares fixed-width extension and wrapper readers across short reads.
fn fuzz_fixed_width(payload: &[u8], chunk_size: usize) {
    let mut extension = ChunkedReader::new(payload, chunk_size);
    let extension_result = extension.read_u32_le();
    let extension_position = extension.position();

    let mut wrapper = BinaryReader::<_, LittleEndian>::new(ChunkedReader::new(
        payload, chunk_size,
    ));
    let wrapper_result = wrapper.read_u32();
    let wrapper_position = wrapper.into_inner().position();

    let mut buffered = BufferedBinaryReader::<_, LittleEndian>::with_capacity(
        ChunkedReader::new(payload, chunk_size),
        16,
    );
    let buffered_result = buffered.read_u32();
    let (inner, unread) = buffered.into_parts();
    let buffered_position = inner.position() - unread.available();

    assert_eq!(
        result_signature(&extension_result),
        result_signature(&wrapper_result)
    );
    assert_eq!(extension_position, wrapper_position);
    assert_eq!(
        result_signature(&extension_result),
        result_signature(&buffered_result)
    );
    assert_eq!(extension_position, buffered_position);
}

/// Checks canonical LEB128 values through the strict public reader APIs.
fn fuzz_strict_leb128(payload: &[u8], chunk_size: usize) {
    let mut bytes = [0_u8; 8];
    let count = payload.len().min(bytes.len());
    bytes[..count].copy_from_slice(&payload[..count]);
    let value = u64::from_le_bytes(bytes);
    let mut encoded = Vec::new();
    encoded.write_uleb_u64(value).expect("LEB128 value should encode");

    let mut extension = ChunkedReader::new(&encoded, chunk_size);
    let extension_result = extension.read_uleb_u64_strict();
    let mut wrapper = Leb128Reader::<_, Strict>::new(ChunkedReader::new(&encoded, chunk_size));
    let wrapper_result = wrapper.read_u64();
    let mut buffered = BufferedLeb128Reader::<_, Strict>::with_capacity(
        ChunkedReader::new(&encoded, chunk_size),
        16,
    );
    let buffered_result = buffered.read_u64();

    assert_eq!(Ok(value), result_signature(&extension_result));
    assert_eq!(result_signature(&extension_result), result_signature(&wrapper_result));
    assert_eq!(result_signature(&extension_result), result_signature(&buffered_result));
}

/// Checks fixed-length UTF-8 string framing through all synchronous readers.
fn fuzz_string(payload: &[u8], chunk_size: usize) {
    let value = String::from_utf8_lossy(payload);
    let mut encoded = Vec::new();
    encoded
        .write_string_with_u16_len(&value, qubit_codec::ByteOrder::LittleEndian)
        .expect("UTF-8 string should encode");

    let mut extension = ChunkedReader::new(&encoded, chunk_size);
    let extension_result = extension
        .read_string_with_u16_len(qubit_codec::ByteOrder::LittleEndian, MAX_FUZZ_INPUT_LEN)
        .map(|text| text.into_bytes());
    let mut wrapper = BinaryReader::<_, LittleEndian>::new(ChunkedReader::new(&encoded, chunk_size));
    let wrapper_result = wrapper
        .read_string_with_u16_len(MAX_FUZZ_INPUT_LEN)
        .map(|text| text.into_bytes());
    let mut buffered = BufferedBinaryReader::<_, LittleEndian>::with_capacity(
        ChunkedReader::new(&encoded, chunk_size),
        16,
    );
    let buffered_result = buffered
        .read_string_with_u16_len(MAX_FUZZ_INPUT_LEN)
        .map(|text| text.into_bytes());

    assert_eq!(Ok(value.as_bytes().to_vec()), string_result_signature(&extension_result));
    assert_eq!(
        string_result_signature(&extension_result),
        string_result_signature(&wrapper_result)
    );
    assert_eq!(
        string_result_signature(&extension_result),
        string_result_signature(&buffered_result)
    );
}

/// Compares unsigned LEB128 extension and wrapper readers across short reads.
fn fuzz_leb128(payload: &[u8], chunk_size: usize) {
    let mut extension = ChunkedReader::new(payload, chunk_size);
    let extension_result = extension.read_uleb_u64();
    let extension_position = extension.position();

    let mut wrapper = Leb128Reader::<_, NonStrict>::new(ChunkedReader::new(
        payload, chunk_size,
    ));
    let wrapper_result = wrapper.read_u64();
    let wrapper_position = wrapper.into_inner().position();

    let mut buffered = BufferedLeb128Reader::<_, NonStrict>::with_capacity(
        ChunkedReader::new(payload, chunk_size),
        16,
    );
    let buffered_result = buffered.read_u64();
    let (inner, unread) = buffered.into_parts();
    let buffered_position = inner.position() - unread.available();

    assert_eq!(
        result_signature(&extension_result),
        result_signature(&wrapper_result)
    );
    assert_eq!(extension_position, wrapper_position);
    assert_eq!(
        result_signature(&extension_result),
        result_signature(&buffered_result)
    );
    assert_eq!(extension_position, buffered_position);
}

/// Compares ZigZag extension and wrapper readers across short reads.
fn fuzz_zig_zag(payload: &[u8], chunk_size: usize) {
    let mut extension = ChunkedReader::new(payload, chunk_size);
    let extension_result = extension.read_zig_zag_i64();
    let extension_position = extension.position();

    let mut wrapper = ZigZagReader::<_, NonStrict>::new(ChunkedReader::new(
        payload, chunk_size,
    ));
    let wrapper_result = wrapper.read_i64();
    let wrapper_position = wrapper.into_inner().position();

    let mut buffered = BufferedZigZagReader::<_, NonStrict>::with_capacity(
        ChunkedReader::new(payload, chunk_size),
        16,
    );
    let buffered_result = buffered.read_i64();
    let (inner, unread) = buffered.into_parts();
    let buffered_position = inner.position() - unread.available();

    assert_eq!(
        result_signature(&extension_result),
        result_signature(&wrapper_result)
    );
    assert_eq!(extension_position, wrapper_position);
    assert_eq!(
        result_signature(&extension_result),
        result_signature(&buffered_result)
    );
    assert_eq!(extension_position, buffered_position);
}

/// Checks values and error kinds without relying on error-message wording.
fn result_signature<T>(result: &io::Result<T>) -> Result<T, ErrorKind>
where
    T: Copy,
{
    match result {
        Ok(value) => Ok(*value),
        Err(error) => Err(error.kind()),
    }
}

/// Converts owned string-read outcomes into assertion-friendly signatures.
fn string_result_signature(result: &io::Result<Vec<u8>>) -> Result<Vec<u8>, ErrorKind> {
    match result {
        Ok(value) => Ok(value.clone()),
        Err(error) => Err(error.kind()),
    }
}

/// A `Read` implementation that makes progress in bounded chunks.
struct ChunkedReader<'a> {
    inner: Cursor<&'a [u8]>,
    chunk_size: usize,
}

impl<'a> ChunkedReader<'a> {
    /// Creates a short-read source over `bytes`.
    const fn new(bytes: &'a [u8], chunk_size: usize) -> Self {
        Self {
            inner: Cursor::new(bytes),
            chunk_size,
        }
    }

    /// Returns the number of bytes consumed by this source.
    fn position(&self) -> usize {
        self.inner.position() as usize
    }
}

impl Read for ChunkedReader<'_> {
    /// Reads no more than the configured short-read chunk.
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let count = output.len().min(self.chunk_size);
        self.inner.read(&mut output[..count])
    }
}
