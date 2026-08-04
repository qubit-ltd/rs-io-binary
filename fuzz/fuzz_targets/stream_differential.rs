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
    Write,
};

use libfuzzer_sys::fuzz_target;
use qubit_codec::LittleEndian;
use qubit_codec_binary::{
    NonStrict,
    Strict,
};
use qubit_io_binary::{
    BinaryReadExt,
    BinaryReader,
    BinaryWriteExt,
    BinaryWriter,
    BufferedBinaryReader,
    BufferedBinaryWriter,
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
    fuzz_binary_writers(payload, chunk_size);
    fuzz_leb128(payload, chunk_size);
    fuzz_strict_leb128(payload, chunk_size);
    fuzz_zig_zag(payload, chunk_size);
    fuzz_string(payload, chunk_size);
    fuzz_malformed_strict_leb128(payload, chunk_size);
    fuzz_malformed_strings(payload, chunk_size);
});

/// Compares fixed-width writers across short writes and injected failures.
fn fuzz_binary_writers(payload: &[u8], chunk_size: usize) {
    let mut bytes = [0_u8; 8];
    let count = payload.len().min(bytes.len());
    bytes[..count].copy_from_slice(&payload[..count]);
    let value = u64::from_le_bytes(bytes);

    let mut extension = ScriptedWriter::new(chunk_size, None);
    let extension_result = extension.write_u64_le(value);
    let extension_bytes = extension.into_bytes();

    let mut wrapper = BinaryWriter::<_, LittleEndian>::new(
        ScriptedWriter::new(chunk_size, None),
    );
    let wrapper_result = wrapper.write_u64(value);
    let wrapper_bytes = wrapper.into_inner().into_bytes();

    let mut buffered = BufferedBinaryWriter::<_, LittleEndian>::with_capacity(
        ScriptedWriter::new(chunk_size, None),
        16,
    );
    let buffered_result = buffered
        .write_u64(value)
        .and_then(|()| qubit_io::Output::flush(&mut buffered));
    let (buffered_inner, pending) = buffered.into_parts();
    let buffered_bytes = buffered_inner.into_bytes();

    assert!(pending.is_empty());
    assert_eq!(Ok(()), result_signature(&extension_result));
    assert_eq!(
        result_signature(&extension_result),
        result_signature(&wrapper_result)
    );
    assert_eq!(
        result_signature(&extension_result),
        result_signature(&buffered_result)
    );
    assert_eq!(extension_bytes, wrapper_bytes);
    assert_eq!(extension_bytes, buffered_bytes);

    let failure_limit =
        usize::from(payload.first().copied().unwrap_or_default() % 8);
    let mut extension = ScriptedWriter::new(chunk_size, Some(failure_limit));
    let extension_result = extension.write_u64_le(value);

    let mut wrapper = BinaryWriter::<_, LittleEndian>::new(
        ScriptedWriter::new(chunk_size, Some(failure_limit)),
    );
    let wrapper_result = wrapper.write_u64(value);

    let mut buffered = BufferedBinaryWriter::<_, LittleEndian>::with_capacity(
        ScriptedWriter::new(chunk_size, Some(failure_limit)),
        16,
    );
    let buffered_result = buffered
        .write_u64(value)
        .and_then(|()| qubit_io::Output::flush(&mut buffered));

    assert_eq!(
        result_signature(&extension_result),
        result_signature(&wrapper_result)
    );
    assert_eq!(
        result_signature(&extension_result),
        result_signature(&buffered_result)
    );
}

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
    encoded
        .write_uleb_u64(value)
        .expect("LEB128 value should encode");

    let mut extension = ChunkedReader::new(&encoded, chunk_size);
    let extension_result = extension.read_uleb_u64_strict();
    let mut wrapper = Leb128Reader::<_, Strict>::new(ChunkedReader::new(
        &encoded, chunk_size,
    ));
    let wrapper_result = wrapper.read_u64();
    let mut buffered = BufferedLeb128Reader::<_, Strict>::with_capacity(
        ChunkedReader::new(&encoded, chunk_size),
        16,
    );
    let buffered_result = buffered.read_u64();

    assert_eq!(Ok(value), result_signature(&extension_result));
    assert_eq!(
        result_signature(&extension_result),
        result_signature(&wrapper_result)
    );
    assert_eq!(
        result_signature(&extension_result),
        result_signature(&buffered_result)
    );
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
        .read_string_with_u16_len(
            qubit_codec::ByteOrder::LittleEndian,
            MAX_FUZZ_INPUT_LEN,
        )
        .map(|text| text.into_bytes());
    let mut wrapper = BinaryReader::<_, LittleEndian>::new(ChunkedReader::new(
        &encoded, chunk_size,
    ));
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

    assert_eq!(
        Ok(value.as_bytes().to_vec()),
        string_result_signature(&extension_result)
    );
    assert_eq!(
        string_result_signature(&extension_result),
        string_result_signature(&wrapper_result)
    );
    assert_eq!(
        string_result_signature(&extension_result),
        string_result_signature(&buffered_result)
    );
}

/// Checks strict readers on non-canonical encodings and logical positions.
fn fuzz_malformed_strict_leb128(payload: &[u8], chunk_size: usize) {
    let width =
        usize::from(payload.first().copied().unwrap_or_default() % 8) + 2;
    let encoded = vec![0x80; width - 1]
        .into_iter()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();

    let mut extension = ChunkedReader::new(&encoded, chunk_size);
    let extension_result = extension.read_uleb_u64_strict();
    let extension_position = extension.position();

    let mut wrapper = Leb128Reader::<_, Strict>::new(ChunkedReader::new(
        &encoded, chunk_size,
    ));
    let wrapper_result = wrapper.read_u64();
    let wrapper_position = wrapper.into_inner().position();

    let mut buffered = BufferedLeb128Reader::<_, Strict>::with_capacity(
        ChunkedReader::new(&encoded, chunk_size),
        16,
    );
    let buffered_result = buffered.read_u64();
    let (inner, unread) = buffered.into_parts();
    let buffered_position = inner.position() - unread.available();

    assert_eq!(
        Err(ErrorKind::InvalidData),
        result_signature(&extension_result)
    );
    assert_eq!(
        result_signature(&extension_result),
        result_signature(&wrapper_result)
    );
    assert_eq!(
        result_signature(&extension_result),
        result_signature(&buffered_result)
    );
    assert_eq!(extension_position, wrapper_position);
    assert_eq!(extension_position, buffered_position);
}

/// Checks malformed, truncated, and oversized UTF-8 frames through all readers.
fn fuzz_malformed_strings(payload: &[u8], chunk_size: usize) {
    let truncated_len = payload.len().saturating_add(1).min(u16::MAX as usize);
    let mut truncated = Vec::with_capacity(2 + payload.len());
    truncated.extend_from_slice(&(truncated_len as u16).to_le_bytes());
    truncated.extend_from_slice(payload);

    let cases = [
        (vec![1, 0, 0xff], 1, ErrorKind::InvalidData),
        (truncated, MAX_FUZZ_INPUT_LEN, ErrorKind::UnexpectedEof),
        (
            u16::MAX.to_le_bytes().to_vec(),
            MAX_FUZZ_INPUT_LEN,
            ErrorKind::InvalidData,
        ),
    ];

    for (encoded, max_len, expected_kind) in cases {
        let mut extension = ChunkedReader::new(&encoded, chunk_size);
        let extension_result = extension
            .read_string_with_u16_len(
                qubit_codec::ByteOrder::LittleEndian,
                max_len,
            )
            .map(String::into_bytes);
        let extension_position = extension.position();

        let mut wrapper = BinaryReader::<_, LittleEndian>::new(
            ChunkedReader::new(&encoded, chunk_size),
        );
        let wrapper_result = wrapper
            .read_string_with_u16_len(max_len)
            .map(String::into_bytes);
        let wrapper_position = wrapper.into_inner().position();

        let mut buffered =
            BufferedBinaryReader::<_, LittleEndian>::with_capacity(
                ChunkedReader::new(&encoded, chunk_size),
                16,
            );
        let buffered_result = buffered
            .read_string_with_u16_len(max_len)
            .map(String::into_bytes);
        let (inner, unread) = buffered.into_parts();
        let buffered_position = inner.position() - unread.available();

        assert_eq!(
            expected_kind,
            extension_result.as_ref().unwrap_err().kind()
        );
        assert_eq!(
            string_result_signature(&extension_result),
            string_result_signature(&wrapper_result)
        );
        assert_eq!(
            string_result_signature(&extension_result),
            string_result_signature(&buffered_result)
        );
        assert_eq!(extension_position, wrapper_position);
        assert_eq!(extension_position, buffered_position);
    }
}

/// Compares unsigned LEB128 extension and wrapper readers across short reads.
fn fuzz_leb128(payload: &[u8], chunk_size: usize) {
    let mut extension = ChunkedReader::new(payload, chunk_size);
    let extension_result = extension.read_uleb_u64_non_strict();
    let extension_position = extension.position();

    let mut wrapper = Leb128Reader::<_, NonStrict>::new(ChunkedReader::new(
        payload, chunk_size,
    ));
    let wrapper_result = wrapper.read_u64_non_strict();
    let wrapper_position = wrapper.into_inner().position();

    let mut buffered = BufferedLeb128Reader::<_, NonStrict>::with_capacity(
        ChunkedReader::new(payload, chunk_size),
        16,
    );
    let buffered_result = buffered.read_u64_non_strict();
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
    let extension_result = extension.read_zig_zag_i64_non_strict();
    let extension_position = extension.position();

    let mut wrapper = ZigZagReader::<_, NonStrict>::new(ChunkedReader::new(
        payload, chunk_size,
    ));
    let wrapper_result = wrapper.read_i64_non_strict();
    let wrapper_position = wrapper.into_inner().position();

    let mut buffered = BufferedZigZagReader::<_, NonStrict>::with_capacity(
        ChunkedReader::new(payload, chunk_size),
        16,
    );
    let buffered_result = buffered.read_i64_non_strict();
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
fn string_result_signature(
    result: &io::Result<Vec<u8>>,
) -> Result<Vec<u8>, ErrorKind> {
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

/// A writer that bounds each write and can fail after a byte budget.
struct ScriptedWriter {
    bytes: Vec<u8>,
    chunk_size: usize,
    remaining: Option<usize>,
}

impl ScriptedWriter {
    /// Creates a short-write writer with an optional failure budget.
    fn new(chunk_size: usize, failure_limit: Option<usize>) -> Self {
        Self {
            bytes: Vec::new(),
            chunk_size,
            remaining: failure_limit,
        }
    }

    /// Returns bytes accepted by this writer.
    fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

impl Write for ScriptedWriter {
    /// Accepts at most `chunk_size` bytes, then optionally fails at the budget.
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        let requested = input.len().min(self.chunk_size);
        let count = self
            .remaining
            .map_or(requested, |remaining| requested.min(remaining));
        if count == 0 && !input.is_empty() {
            return Err(io::Error::new(
                ErrorKind::Other,
                "scripted writer failure",
            ));
        }
        self.bytes.extend_from_slice(&input[..count]);
        if let Some(remaining) = &mut self.remaining {
            *remaining -= count;
        }
        Ok(count)
    }

    /// Reports success because failures are injected by `write`.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
