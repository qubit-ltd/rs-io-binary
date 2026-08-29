// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::io::Cursor;
use std::io::ErrorKind;

use qubit_codec::BigEndian;
use qubit_codec::ByteOrder;
use qubit_codec::LittleEndian;
use qubit_codec::NativeEndian;
use qubit_io::Output;
use qubit_io::Seekable;
use qubit_io_binary::BinaryWriter;

#[test]
fn test_binary_writer_supports_native_endian() {
    let mut writer = BinaryWriter::<_, NativeEndian>::new(Vec::new());
    writer.write_u32(0x1234_5678).expect("native value should write");
    assert_eq!(0x1234_5678_u32.to_ne_bytes(), writer.into_inner().as_slice());
}

#[test]
fn test_binary_writer_writes_all_big_endian_methods() {
    let mut writer = BinaryWriter::<_, BigEndian>::new(Vec::new());

    assert_eq!(ByteOrder::BigEndian, writer.byte_order());
    Output::write_fully(&mut writer, &[0xaa, 0xbb]).expect("bytes should be written");
    writer.write_u8(0x12).expect("u8 should be written");
    writer.write_i8(-2).expect("i8 should be written");
    writer.write_u16(0x1234).expect("u16 should be written");
    writer.write_u32(0x1234_5678).expect("u32 should be written");
    writer.write_u64(0x0123_4567_89ab_cdef).expect("u64 should be written");
    writer
        .write_u128(0x0123_4567_89ab_cdef_fedc_ba98_7654_3210)
        .expect("u128 should be written");
    writer.write_i16(-0x1234).expect("i16 should be written");
    writer.write_i32(-0x0123_4567).expect("i32 should be written");
    writer.write_i64(-0x0123_4567_89ab_cdef).expect("i64 should be written");
    writer
        .write_i128(-0x0123_4567_89ab_cdef_fedc_ba98_7654_3210)
        .expect("i128 should be written");
    writer.write_f32(12.5).expect("f32 should be written");
    writer.write_f64(-25.25).expect("f64 should be written");
    writer
        .write_string_with_u16_len("hi")
        .expect("u16 string should be written");
    writer
        .write_string_with_u32_len("ok")
        .expect("u32 string should be written");

    assert!(!writer.into_inner().is_empty());
}

#[test]
fn test_binary_writer_writes_little_endian_and_exposes_accessors() {
    let mut writer = BinaryWriter::<_, LittleEndian>::new(Cursor::new(Vec::new()));

    assert_eq!(ByteOrder::LittleEndian, writer.byte_order());
    assert_eq!(0, writer.inner().position());
    writer.inner_mut().set_position(0);
    writer.write_u8(0x12).expect("u8 should be written");
    writer.write_i8(-2).expect("i8 should be written");
    writer.write_u16(0x1234).expect("u16 should be written");
    writer.write_u32(0x1234_5678).expect("u32 should be written");
    writer.write_u64(0x0123_4567_89ab_cdef).expect("u64 should be written");
    writer
        .write_u128(0x0123_4567_89ab_cdef_fedc_ba98_7654_3210)
        .expect("u128 should be written");
    writer.write_i16(-0x1234).expect("i16 should be written");
    writer.write_i32(-0x0123_4567).expect("i32 should be written");
    writer.write_i64(-0x0123_4567_89ab_cdef).expect("i64 should be written");
    writer
        .write_i128(-0x0123_4567_89ab_cdef_fedc_ba98_7654_3210)
        .expect("i128 should be written");
    writer.write_f32(12.5).expect("f32 should be written");
    writer.write_f64(-25.25).expect("f64 should be written");
    writer
        .write_string_with_u16_len("hi")
        .expect("u16 string should be written");
    writer
        .write_string_with_u32_len("ok")
        .expect("u32 string should be written");

    let mut expected = Vec::new();
    expected.push(0x12);
    expected.push((-2_i8) as u8);
    expected.extend_from_slice(&0x1234_u16.to_le_bytes());
    expected.extend_from_slice(&0x1234_5678_u32.to_le_bytes());
    expected.extend_from_slice(&0x0123_4567_89ab_cdef_u64.to_le_bytes());
    expected.extend_from_slice(&0x0123_4567_89ab_cdef_fedc_ba98_7654_3210_u128.to_le_bytes());
    expected.extend_from_slice(&(-0x1234_i16).to_le_bytes());
    expected.extend_from_slice(&(-0x0123_4567_i32).to_le_bytes());
    expected.extend_from_slice(&(-0x0123_4567_89ab_cdef_i64).to_le_bytes());
    expected.extend_from_slice(&(-0x0123_4567_89ab_cdef_fedc_ba98_7654_3210_i128).to_le_bytes());
    expected.extend_from_slice(&12.5_f32.to_bits().to_le_bytes());
    expected.extend_from_slice(&(-25.25_f64).to_bits().to_le_bytes());
    expected.extend_from_slice(&2_u16.to_le_bytes());
    expected.extend_from_slice(b"hi");
    expected.extend_from_slice(&2_u32.to_le_bytes());
    expected.extend_from_slice(b"ok");
    assert_eq!(expected, writer.into_inner().into_inner());
}

#[test]
fn test_binary_writer_reports_length_errors() {
    let mut writer = BinaryWriter::<_, BigEndian>::new(Vec::new());
    let value = "x".repeat(usize::from(u16::MAX) + 1);

    assert_eq!(
        ErrorKind::InvalidInput,
        writer
            .write_string_with_u16_len(&value)
            .expect_err("oversized u16 string should fail")
            .kind()
    );
}

#[test]
fn test_binary_writer_write_and_seek_delegate_to_inner_writer() {
    let mut writer = BinaryWriter::<_, LittleEndian>::new(std::io::Cursor::new(vec![0; 4]));

    Seekable::seek_to(&mut writer, std::io::SeekFrom::Start(1)).expect("seeking through BinaryWriter should succeed");
    Output::write_fully(&mut writer, b"xy").expect("writing through BinaryWriter should succeed");
    Output::flush(&mut writer).expect("flushing through BinaryWriter should succeed");

    let cursor = writer.into_inner();
    assert_eq!(cursor.into_inner(), vec![0, b'x', b'y', 0]);
}
