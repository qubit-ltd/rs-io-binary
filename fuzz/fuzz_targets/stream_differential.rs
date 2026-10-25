// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![no_main]

use std::io;
use std::io::Cursor;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;

use libfuzzer_sys::fuzz_target;
use qubit_codec::ByteOrder;
use qubit_codec::LittleEndian;
use qubit_codec_binary::NonStrict;
use qubit_codec_binary::Strict;
use qubit_io::BufferedOutputRecovery;
use qubit_io::Output;
use qubit_io_binary::BinaryReadExt;
use qubit_io_binary::BinaryReader;
use qubit_io_binary::BinaryWriteExt;
use qubit_io_binary::BinaryWriter;
use qubit_io_binary::BufferedBinaryReader;
use qubit_io_binary::BufferedBinaryWriter;
use qubit_io_binary::BufferedLeb128Reader;
use qubit_io_binary::BufferedLeb128Writer;
use qubit_io_binary::BufferedZigZagReader;
use qubit_io_binary::BufferedZigZagWriter;
use qubit_io_binary::Leb128ReadExt;
use qubit_io_binary::Leb128Reader;
use qubit_io_binary::Leb128WriteExt;
use qubit_io_binary::Leb128Writer;
use qubit_io_binary::StringReadExt;
use qubit_io_binary::StringWriteExt;
use qubit_io_binary::ZigZagReadExt;
use qubit_io_binary::ZigZagReader;
use qubit_io_binary::ZigZagWriteExt;
use qubit_io_binary::ZigZagWriter;

/// Keeps allocations and each target invocation bounded outside CI.
const MAX_FUZZ_INPUT_LEN: usize = 4096;
const MAX_STRING_LEN: usize = MAX_FUZZ_INPUT_LEN * 3;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_FUZZ_INPUT_LEN)];
    let chunk_size = usize::from(data.first().copied().unwrap_or_default() % 8) + 1;
    let payload = data.get(1..).unwrap_or_default();

    fuzz_fixed_width(payload, chunk_size);
    fuzz_binary_writers(payload, chunk_size);
    fuzz_leb128(payload, chunk_size);
    fuzz_varint_widths(payload, chunk_size);
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

    let mut wrapper = BinaryWriter::<_, LittleEndian>::new(ScriptedWriter::new(chunk_size, None));
    let wrapper_result = wrapper.write_u64(value);
    let wrapper_bytes = wrapper.into_inner().into_bytes();

    let mut buffered =
        BufferedBinaryWriter::<_, LittleEndian>::with_capacity(ScriptedWriter::new(chunk_size, None), 16);
    let buffered_result = buffered.write_u64(value).and_then(|()| Output::flush(&mut buffered));
    let (buffered_inner, pending) = buffered.into_parts();
    let buffered_bytes = buffered_inner.into_bytes();

    assert!(matches!(
        pending,
        BufferedOutputRecovery::Pending(buffer) if buffer.is_empty()
    ));
    assert_eq!(Ok(()), result_signature(&extension_result));
    assert_eq!(result_signature(&extension_result), result_signature(&wrapper_result));
    assert_eq!(result_signature(&extension_result), result_signature(&buffered_result));
    assert_eq!(extension_bytes, wrapper_bytes);
    assert_eq!(extension_bytes, buffered_bytes);

    let failure_limit = usize::from(payload.first().copied().unwrap_or_default() % 8);
    let expected = value.to_le_bytes();
    let mut extension = ScriptedWriter::new(chunk_size, Some(failure_limit));
    let extension_result = extension.write_u64_le(value);
    let extension_bytes = extension.into_bytes();

    let mut wrapper = BinaryWriter::<_, LittleEndian>::new(ScriptedWriter::new(chunk_size, Some(failure_limit)));
    let wrapper_result = wrapper.write_u64(value);
    let wrapper_bytes = wrapper.into_inner().into_bytes();

    let mut buffered = BufferedBinaryWriter::<_, LittleEndian>::with_capacity(
        ScriptedWriter::new(chunk_size, Some(failure_limit)),
        16,
    );
    let buffered_write_result = buffered.write_u64(value);
    buffered_write_result.expect("buffered writer should accept the value before flushing");
    let buffered_result = Output::flush(&mut buffered);
    let (buffered_inner, pending) = buffered.into_parts();
    let buffered_bytes = buffered_inner.into_bytes();

    assert_eq!(result_signature(&extension_result), result_signature(&wrapper_result));
    assert_eq!(result_signature(&extension_result), result_signature(&buffered_result));
    assert_eq!(&expected[..failure_limit], extension_bytes);
    assert_eq!(&expected[..failure_limit], wrapper_bytes);
    assert_eq!(&expected[..failure_limit], buffered_bytes);
    let pending_bytes = match pending {
        BufferedOutputRecovery::Pending(buffer) => buffer.readable().to_vec(),
        BufferedOutputRecovery::Indeterminate(_) => {
            panic!("scripted I/O error must preserve determinate pending bytes")
        }
    };
    assert_eq!(&expected[failure_limit..], pending_bytes);
    assert_eq!(8, buffered_bytes.len() + pending_bytes.len());
    assert_eq!(expected.as_slice(), [buffered_bytes, pending_bytes].concat());
}

/// Compares fixed-width extension and wrapper readers across short reads.
fn fuzz_fixed_width(payload: &[u8], chunk_size: usize) {
    let mut extension = ChunkedReader::new(payload, chunk_size);
    let extension_result = extension.read_u32_le();
    let extension_position = extension.position();

    let mut wrapper = BinaryReader::<_, LittleEndian>::new(ChunkedReader::new(payload, chunk_size));
    let wrapper_result = wrapper.read_u32();
    let wrapper_position = wrapper.into_inner().position();

    let mut buffered =
        BufferedBinaryReader::<_, LittleEndian>::with_capacity(ChunkedReader::new(payload, chunk_size), 16);
    let buffered_result = buffered.read_u32();
    let (inner, unread) = buffered.into_parts();
    let buffered_position = inner.position() - unread.available();

    assert_eq!(result_signature(&extension_result), result_signature(&wrapper_result));
    assert_eq!(extension_position, wrapper_position);
    assert_eq!(result_signature(&extension_result), result_signature(&buffered_result));
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
    let mut buffered = BufferedLeb128Reader::<_, Strict>::with_capacity(ChunkedReader::new(&encoded, chunk_size), 16);
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
        .write_string_with_u16_len(&value, ByteOrder::LittleEndian)
        .expect("UTF-8 string should encode");

    let mut extension = ChunkedReader::new(&encoded, chunk_size);
    let extension_result = extension
        .read_string_with_u16_len(ByteOrder::LittleEndian, MAX_STRING_LEN)
        .map(|text| text.into_bytes());
    let mut wrapper = BinaryReader::<_, LittleEndian>::new(ChunkedReader::new(&encoded, chunk_size));
    let wrapper_result = wrapper
        .read_string_with_u16_len(MAX_STRING_LEN)
        .map(|text| text.into_bytes());
    let mut buffered =
        BufferedBinaryReader::<_, LittleEndian>::with_capacity(ChunkedReader::new(&encoded, chunk_size), 16);
    let buffered_result = buffered
        .read_string_with_u16_len(MAX_STRING_LEN)
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
    let width = usize::from(payload.first().copied().unwrap_or_default() % 8) + 2;
    let encoded = vec![0x80; width - 1]
        .into_iter()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();

    let mut extension = ChunkedReader::new(&encoded, chunk_size);
    let extension_result = extension.read_uleb_u64_strict();
    let extension_position = extension.position();

    let mut wrapper = Leb128Reader::<_, Strict>::new(ChunkedReader::new(&encoded, chunk_size));
    let wrapper_result = wrapper.read_u64();
    let wrapper_position = wrapper.into_inner().position();

    let mut buffered = BufferedLeb128Reader::<_, Strict>::with_capacity(ChunkedReader::new(&encoded, chunk_size), 16);
    let buffered_result = buffered.read_u64();
    let (inner, unread) = buffered.into_parts();
    let buffered_position = inner.position() - unread.available();

    assert_eq!(Err(ErrorKind::InvalidData), result_signature(&extension_result));
    assert_eq!(result_signature(&extension_result), result_signature(&wrapper_result));
    assert_eq!(result_signature(&extension_result), result_signature(&buffered_result));
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
            .read_string_with_u16_len(ByteOrder::LittleEndian, max_len)
            .map(String::into_bytes);
        let extension_position = extension.position();

        let mut wrapper = BinaryReader::<_, LittleEndian>::new(ChunkedReader::new(&encoded, chunk_size));
        let wrapper_result = wrapper.read_string_with_u16_len(max_len).map(String::into_bytes);
        let wrapper_position = wrapper.into_inner().position();

        let mut buffered =
            BufferedBinaryReader::<_, LittleEndian>::with_capacity(ChunkedReader::new(&encoded, chunk_size), 16);
        let buffered_result = buffered.read_string_with_u16_len(max_len).map(String::into_bytes);
        let (inner, unread) = buffered.into_parts();
        let buffered_position = inner.position() - unread.available();

        assert_eq!(expected_kind, extension_result.as_ref().unwrap_err().kind());
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

    let mut wrapper = Leb128Reader::<_, NonStrict>::new(ChunkedReader::new(payload, chunk_size));
    let wrapper_result = wrapper.read_u64_non_strict();
    let wrapper_position = wrapper.into_inner().position();

    let mut buffered = BufferedLeb128Reader::<_, NonStrict>::with_capacity(ChunkedReader::new(payload, chunk_size), 16);
    let buffered_result = buffered.read_u64_non_strict();
    let (inner, unread) = buffered.into_parts();
    let buffered_position = inner.position() - unread.available();

    assert_eq!(result_signature(&extension_result), result_signature(&wrapper_result));
    assert_eq!(extension_position, wrapper_position);
    assert_eq!(result_signature(&extension_result), result_signature(&buffered_result));
    assert_eq!(extension_position, buffered_position);
}

/// Compares ZigZag extension and wrapper readers across short reads.
fn fuzz_zig_zag(payload: &[u8], chunk_size: usize) {
    let mut extension = ChunkedReader::new(payload, chunk_size);
    let extension_result = extension.read_zig_zag_i64_non_strict();
    let extension_position = extension.position();

    let mut wrapper = ZigZagReader::<_, NonStrict>::new(ChunkedReader::new(payload, chunk_size));
    let wrapper_result = wrapper.read_i64_non_strict();
    let wrapper_position = wrapper.into_inner().position();

    let mut buffered = BufferedZigZagReader::<_, NonStrict>::with_capacity(ChunkedReader::new(payload, chunk_size), 16);
    let buffered_result = buffered.read_i64_non_strict();
    let (inner, unread) = buffered.into_parts();
    let buffered_position = inner.position() - unread.available();

    assert_eq!(result_signature(&extension_result), result_signature(&wrapper_result));
    assert_eq!(extension_position, wrapper_position);
    assert_eq!(result_signature(&extension_result), result_signature(&buffered_result));
    assert_eq!(extension_position, buffered_position);
}

/// Compares varint widths and writer encodings across public stream APIs.
fn fuzz_varint_widths(payload: &[u8], chunk_size: usize) {
    macro_rules! check_read {
        ($ty:ty, $extension_method:ident, $typed_method:ident, $buffered_type:ty, $buffered_method:ident, $policy:ty) => {{
            let mut extension = ChunkedReader::new(payload, chunk_size);
            let extension_result = extension.$extension_method();
            let extension_position = extension.position();

            let mut typed = Leb128Reader::<_, $policy>::new(ChunkedReader::new(payload, chunk_size));
            let typed_result = typed.$typed_method();
            let typed_position = typed.into_inner().position();

            let mut buffered = <$buffered_type>::with_capacity(ChunkedReader::new(payload, chunk_size), 16);
            let buffered_result = buffered.$buffered_method();
            let (inner, unread) = buffered.into_parts();
            let buffered_position = inner.position() - unread.available();

            assert_eq!(result_signature(&extension_result), result_signature(&typed_result));
            assert_eq!(extension_position, typed_position);
            assert_eq!(result_signature(&extension_result), result_signature(&buffered_result));
            assert_eq!(extension_position, buffered_position);
        }};
    }

    macro_rules! check_zigzag_read {
        ($extension_method:ident, $typed_method:ident, $buffered_method:ident, $ty:ty, $policy:ty) => {{
            let mut extension = ChunkedReader::new(payload, chunk_size);
            let extension_result = extension.$extension_method();
            let extension_position = extension.position();

            let mut typed = ZigZagReader::<_, $policy>::new(ChunkedReader::new(payload, chunk_size));
            let typed_result = typed.$typed_method();
            let typed_position = typed.into_inner().position();

            let mut buffered =
                BufferedZigZagReader::<_, $policy>::with_capacity(ChunkedReader::new(payload, chunk_size), 16);
            let buffered_result = buffered.$buffered_method();
            let (inner, unread) = buffered.into_parts();
            let buffered_position = inner.position() - unread.available();

            assert_eq!(result_signature(&extension_result), result_signature(&typed_result));
            assert_eq!(extension_position, typed_position);
            assert_eq!(result_signature(&extension_result), result_signature(&buffered_result));
            assert_eq!(extension_position, buffered_position);
        }};
    }

    macro_rules! check_writer {
        ($ty:ty, $extension_method:ident, $typed_method:ident, $buffered_method:ident, $extension_strict:ident, $extension_non_strict:ident, $reader_strict:ident, $reader_non_strict:ident) => {{
            let mut raw = [0_u8; std::mem::size_of::<$ty>()];
            let count = payload.len().min(raw.len());
            raw[..count].copy_from_slice(&payload[..count]);
            let value = <$ty>::from_le_bytes(raw);

            let mut extension = Vec::new();
            extension.$extension_method(value).expect("value should encode");

            let mut typed = Vec::new();
            let mut writer = Leb128Writer::<_>::new(&mut typed);
            writer.$typed_method(value).expect("value should encode");

            let mut buffered = BufferedLeb128Writer::<_>::with_capacity(Vec::new(), 16);
            buffered.$buffered_method(value).expect("value should encode");
            Output::flush(&mut buffered).expect("buffered value should flush");
            let (buffered, pending) = buffered.into_parts();
            assert!(matches!(pending, BufferedOutputRecovery::Pending(buffer) if buffer.is_empty()));

            assert_eq!(extension, typed);
            assert_eq!(extension, buffered);

            macro_rules! check_roundtrip {
                ($policy:ty, $decode_extension_method:ident, $reader_method:ident) => {{
                    let mut extension_reader = ChunkedReader::new(&extension, chunk_size);
                    let extension_result = extension_reader.$decode_extension_method();
                    let extension_position = extension_reader.position();

                    let mut reader = Leb128Reader::<_, $policy>::new(ChunkedReader::new(&extension, chunk_size));
                    let reader_result = reader.$reader_method();
                    let reader_position = reader.into_inner().position();

                    let mut buffered_reader = BufferedLeb128Reader::<_, $policy>::with_capacity(
                        ChunkedReader::new(&extension, chunk_size),
                        16,
                    );
                    let buffered_result = buffered_reader.$reader_method();
                    let (inner, unread) = buffered_reader.into_parts();
                    let buffered_position = inner.position() - unread.available();

                    assert_eq!(Ok(value), result_signature(&extension_result));
                    assert_eq!(result_signature(&extension_result), result_signature(&reader_result));
                    assert_eq!(result_signature(&extension_result), result_signature(&buffered_result));
                    assert_eq!(extension.len(), extension_position);
                    assert_eq!(extension_position, reader_position);
                    assert_eq!(extension_position, buffered_position);
                }};
            }

            check_roundtrip!(Strict, $extension_strict, $reader_strict);
            check_roundtrip!(NonStrict, $extension_non_strict, $reader_non_strict);
        }};
    }

    macro_rules! check_zigzag_writer {
        ($ty:ty, $extension_method:ident, $typed_method:ident, $buffered_method:ident, $extension_strict:ident, $extension_non_strict:ident, $reader_strict:ident, $reader_non_strict:ident) => {{
            let mut raw = [0_u8; std::mem::size_of::<$ty>()];
            let count = payload.len().min(raw.len());
            raw[..count].copy_from_slice(&payload[..count]);
            let value = <$ty>::from_le_bytes(raw);

            let mut extension = Vec::new();
            extension.$extension_method(value).expect("value should encode");

            let mut typed = Vec::new();
            let mut writer = ZigZagWriter::<_>::new(&mut typed);
            writer.$typed_method(value).expect("value should encode");

            let mut buffered = BufferedZigZagWriter::<_>::with_capacity(Vec::new(), 16);
            buffered.$buffered_method(value).expect("value should encode");
            Output::flush(&mut buffered).expect("buffered value should flush");
            let (buffered, pending) = buffered.into_parts();
            assert!(matches!(pending, BufferedOutputRecovery::Pending(buffer) if buffer.is_empty()));

            assert_eq!(extension, typed);
            assert_eq!(extension, buffered);

            macro_rules! check_roundtrip {
                ($policy:ty, $decode_extension_method:ident, $reader_method:ident) => {{
                    let mut extension_reader = ChunkedReader::new(&extension, chunk_size);
                    let extension_result = extension_reader.$decode_extension_method();
                    let extension_position = extension_reader.position();

                    let mut reader = ZigZagReader::<_, $policy>::new(ChunkedReader::new(&extension, chunk_size));
                    let reader_result = reader.$reader_method();
                    let reader_position = reader.into_inner().position();

                    let mut buffered_reader = BufferedZigZagReader::<_, $policy>::with_capacity(
                        ChunkedReader::new(&extension, chunk_size),
                        16,
                    );
                    let buffered_result = buffered_reader.$reader_method();
                    let (inner, unread) = buffered_reader.into_parts();
                    let buffered_position = inner.position() - unread.available();

                    assert_eq!(Ok(value), result_signature(&extension_result));
                    assert_eq!(result_signature(&extension_result), result_signature(&reader_result));
                    assert_eq!(result_signature(&extension_result), result_signature(&buffered_result));
                    assert_eq!(extension.len(), extension_position);
                    assert_eq!(extension_position, reader_position);
                    assert_eq!(extension_position, buffered_position);
                }};
            }

            check_roundtrip!(Strict, $extension_strict, $reader_strict);
            check_roundtrip!(NonStrict, $extension_non_strict, $reader_non_strict);
        }};
    }

    check_read!(u8, read_uleb_u8_strict, read_u8, BufferedLeb128Reader<_, Strict>, read_u8, Strict);
    check_read!(u8, read_uleb_u8_non_strict, read_u8_non_strict, BufferedLeb128Reader<_, NonStrict>, read_u8_non_strict, NonStrict);
    check_read!(u16, read_uleb_u16_strict, read_u16, BufferedLeb128Reader<_, Strict>, read_u16, Strict);
    check_read!(u16, read_uleb_u16_non_strict, read_u16_non_strict, BufferedLeb128Reader<_, NonStrict>, read_u16_non_strict, NonStrict);
    check_read!(u64, read_uleb_u64_strict, read_u64, BufferedLeb128Reader<_, Strict>, read_u64, Strict);
    check_read!(u64, read_uleb_u64_non_strict, read_u64_non_strict, BufferedLeb128Reader<_, NonStrict>, read_u64_non_strict, NonStrict);
    check_read!(u128, read_uleb_u128_strict, read_u128, BufferedLeb128Reader<_, Strict>, read_u128, Strict);
    check_read!(u128, read_uleb_u128_non_strict, read_u128_non_strict, BufferedLeb128Reader<_, NonStrict>, read_u128_non_strict, NonStrict);

    check_read!(i8, read_sleb_i8_strict, read_i8, BufferedLeb128Reader<_, Strict>, read_i8, Strict);
    check_read!(i8, read_sleb_i8_non_strict, read_i8_non_strict, BufferedLeb128Reader<_, NonStrict>, read_i8_non_strict, NonStrict);
    check_read!(i16, read_sleb_i16_strict, read_i16, BufferedLeb128Reader<_, Strict>, read_i16, Strict);
    check_read!(i16, read_sleb_i16_non_strict, read_i16_non_strict, BufferedLeb128Reader<_, NonStrict>, read_i16_non_strict, NonStrict);
    check_read!(i64, read_sleb_i64_strict, read_i64, BufferedLeb128Reader<_, Strict>, read_i64, Strict);
    check_read!(i64, read_sleb_i64_non_strict, read_i64_non_strict, BufferedLeb128Reader<_, NonStrict>, read_i64_non_strict, NonStrict);
    check_read!(i128, read_sleb_i128_strict, read_i128, BufferedLeb128Reader<_, Strict>, read_i128, Strict);
    check_read!(i128, read_sleb_i128_non_strict, read_i128_non_strict, BufferedLeb128Reader<_, NonStrict>, read_i128_non_strict, NonStrict);

    check_zigzag_read!(read_zig_zag_i8_strict, read_i8, read_i8, i8, Strict);
    check_zigzag_read!(
        read_zig_zag_i8_non_strict,
        read_i8_non_strict,
        read_i8_non_strict,
        i8,
        NonStrict
    );
    check_zigzag_read!(read_zig_zag_i16_strict, read_i16, read_i16, i16, Strict);
    check_zigzag_read!(
        read_zig_zag_i16_non_strict,
        read_i16_non_strict,
        read_i16_non_strict,
        i16,
        NonStrict
    );
    check_zigzag_read!(read_zig_zag_i64_strict, read_i64, read_i64, i64, Strict);
    check_zigzag_read!(
        read_zig_zag_i64_non_strict,
        read_i64_non_strict,
        read_i64_non_strict,
        i64,
        NonStrict
    );
    check_zigzag_read!(read_zig_zag_i128_strict, read_i128, read_i128, i128, Strict);
    check_zigzag_read!(
        read_zig_zag_i128_non_strict,
        read_i128_non_strict,
        read_i128_non_strict,
        i128,
        NonStrict
    );

    check_writer!(
        u8,
        write_uleb_u8,
        write_u8,
        write_u8,
        read_uleb_u8_strict,
        read_uleb_u8_non_strict,
        read_u8,
        read_u8_non_strict
    );
    check_writer!(
        u16,
        write_uleb_u16,
        write_u16,
        write_u16,
        read_uleb_u16_strict,
        read_uleb_u16_non_strict,
        read_u16,
        read_u16_non_strict
    );
    check_writer!(
        u64,
        write_uleb_u64,
        write_u64,
        write_u64,
        read_uleb_u64_strict,
        read_uleb_u64_non_strict,
        read_u64,
        read_u64_non_strict
    );
    check_writer!(
        u128,
        write_uleb_u128,
        write_u128,
        write_u128,
        read_uleb_u128_strict,
        read_uleb_u128_non_strict,
        read_u128,
        read_u128_non_strict
    );
    check_writer!(
        i8,
        write_sleb_i8,
        write_i8,
        write_i8,
        read_sleb_i8_strict,
        read_sleb_i8_non_strict,
        read_i8,
        read_i8_non_strict
    );
    check_writer!(
        i16,
        write_sleb_i16,
        write_i16,
        write_i16,
        read_sleb_i16_strict,
        read_sleb_i16_non_strict,
        read_i16,
        read_i16_non_strict
    );
    check_writer!(
        i64,
        write_sleb_i64,
        write_i64,
        write_i64,
        read_sleb_i64_strict,
        read_sleb_i64_non_strict,
        read_i64,
        read_i64_non_strict
    );
    check_writer!(
        i128,
        write_sleb_i128,
        write_i128,
        write_i128,
        read_sleb_i128_strict,
        read_sleb_i128_non_strict,
        read_i128,
        read_i128_non_strict
    );
    check_zigzag_writer!(
        i8,
        write_zig_zag_i8,
        write_i8,
        write_i8,
        read_zig_zag_i8_strict,
        read_zig_zag_i8_non_strict,
        read_i8,
        read_i8_non_strict
    );
    check_zigzag_writer!(
        i16,
        write_zig_zag_i16,
        write_i16,
        write_i16,
        read_zig_zag_i16_strict,
        read_zig_zag_i16_non_strict,
        read_i16,
        read_i16_non_strict
    );
    check_zigzag_writer!(
        i64,
        write_zig_zag_i64,
        write_i64,
        write_i64,
        read_zig_zag_i64_strict,
        read_zig_zag_i64_non_strict,
        read_i64,
        read_i64_non_strict
    );
    check_zigzag_writer!(
        i128,
        write_zig_zag_i128,
        write_i128,
        write_i128,
        read_zig_zag_i128_strict,
        read_zig_zag_i128_non_strict,
        read_i128,
        read_i128_non_strict
    );
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
        let count = self.remaining.map_or(requested, |remaining| requested.min(remaining));
        if count == 0 && !input.is_empty() {
            return Err(io::Error::new(ErrorKind::Other, "scripted writer failure"));
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
