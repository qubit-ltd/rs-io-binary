// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::Cursor;
use std::io::ErrorKind;
use std::io::SeekFrom;

use qubit_codec::ByteOrder;
use qubit_codec::LittleEndian;
use qubit_codec_binary::NonStrict;
use qubit_io_binary::BufferedBinaryReader;
use qubit_io_binary::BufferedLeb128Reader;
use qubit_io_binary::BufferedLeb128Writer;

#[test]
fn test_transcode_decode_input_ext_delegates_public_raw_and_codec_reads() {
    let mut reader = BufferedBinaryReader::<_, LittleEndian>::with_capacity(
        Cursor::new(vec![0x34, 0x12, 0x56, 0x78]),
        1,
    );

    assert_eq!(ByteOrder::LittleEndian, reader.byte_order());
    let mut buffer = [0_u8; 1];
    assert_eq!(
        1,
        reader
            .read(&mut buffer)
            .expect("raw read should return one byte"),
    );
    assert_eq!(0x34, buffer[0]);
    assert_eq!(
        1,
        reader
            .seek_to(SeekFrom::Start(1))
            .expect("seek should succeed"),
    );
    assert_eq!(
        0x5612,
        reader
            .read_u16()
            .expect("buffered codec read should succeed"),
    );
}

#[test]
fn test_transcode_decode_input_ext_maps_incomplete_public_decode() {
    let mut reader = BufferedLeb128Reader::<_, NonStrict>::with_capacity(
        Cursor::new(vec![0x80]),
        1,
    );

    let error = reader
        .read_i8_non_strict()
        .expect_err("truncated LEB128 value should fail");

    assert_eq!(ErrorKind::UnexpectedEof, error.kind());
}

#[test]
fn test_transcode_decode_input_ext_reads_public_utf8_payload() {
    let value = "hello";
    let mut writer = BufferedLeb128Writer::new(Vec::new());
    writer
        .write_utf8_string_usize(value)
        .expect("UTF-8 payload should encode");
    writer.flush().expect("encoded bytes should flush");
    let bytes = writer.inner().clone();

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::with_capacity(
        Cursor::new(bytes),
        1,
    );
    let decoded = reader
        .read_utf8_string_usize_non_strict(10)
        .expect("UTF-8 payload should decode");

    assert_eq!(value, decoded);
}
