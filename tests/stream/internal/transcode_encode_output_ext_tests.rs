// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::{
    Cursor,
    SeekFrom,
};

use qubit_codec::LittleEndian;
use qubit_codec_binary::NonStrict;
use qubit_io::{
    Output,
    Seekable,
};
use qubit_io_binary::{
    BufferedBinaryWriter,
    BufferedLeb128Reader,
    BufferedLeb128Writer,
};

#[test]
fn test_transcode_encode_output_ext_writes_scalar_and_raw_bytes() {
    let mut writer =
        BufferedBinaryWriter::<_, LittleEndian>::with_capacity(Vec::new(), 4);

    writer.write_u16(0x1234).expect("u16 should encode");
    writer
        .write_fully(b"ab")
        .expect("raw bytes should enter the same buffer");
    writer.flush().expect("all buffered bytes should flush");

    assert_eq!(vec![0x34, 0x12, b'a', b'b'], writer.inner().clone());
}

#[test]
fn test_transcode_encode_output_ext_writes_values_with_tiny_capacity() {
    let mut writer =
        BufferedBinaryWriter::<_, LittleEndian>::with_capacity(Vec::new(), 1);

    writer.write_u16(0x1234).expect("first u16 should encode");
    writer.write_u32(0x89AB_CDEF).expect("u32 should encode");
    writer.write_u16(0x0102).expect("second u16 should encode");
    writer.write_u8(0xFF).expect("u8 should encode");
    writer.flush().expect("encoded bytes should flush");

    let mut expected = Vec::new();
    expected.extend_from_slice(&0x1234_u16.to_le_bytes());
    expected.extend_from_slice(&0x89AB_CDEF_u32.to_le_bytes());
    expected.extend_from_slice(&0x0102_u16.to_le_bytes());
    expected.push(0xFF);
    assert_eq!(expected, writer.inner().clone());
}

#[test]
fn test_transcode_encode_output_ext_flushes_before_public_seek() {
    let mut writer = BufferedLeb128Writer::new(Cursor::new(Vec::new()));

    writer.write_u8(1).expect("u8 should encode");
    writer
        .seek_to(SeekFrom::Start(0))
        .expect("seek should flush before moving");
    writer.flush().expect("wrapped output should flush");

    assert_eq!(vec![1], writer.inner().clone().into_inner());
}

#[test]
fn test_transcode_encode_output_ext_writes_public_utf8_payload() {
    let mut writer = BufferedLeb128Writer::new(Vec::new());

    writer
        .write_utf8_string_usize("hello")
        .expect("UTF-8 string should encode");
    writer.flush().expect("encoded UTF-8 bytes should flush");

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(
        writer.inner().clone(),
    ));
    assert_eq!(
        "hello",
        reader
            .read_utf8_string_usize_non_strict(10)
            .expect("UTF-8 string should decode"),
    );
}
