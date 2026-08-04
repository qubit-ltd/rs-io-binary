// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::io::{
    Cursor,
    ErrorKind,
};

use qubit_codec_binary::{
    NonStrict,
    Strict,
    ZigZagCodec,
};
use qubit_io::{
    Input,
    Seekable,
};
use qubit_io_binary::{
    BufferedZigZagReader,
    ZigZagWriteExt,
};

#[test]
fn test_buffered_zig_zag_reader_reads_values_across_buffer_boundaries() {
    let mut bytes = Vec::new();
    bytes
        .write_zig_zag_i8(i8::MIN)
        .expect("i8 should be encoded");
    bytes
        .write_zig_zag_i16(-300)
        .expect("i16 should be encoded");
    bytes
        .write_zig_zag_i32(-0x1f600)
        .expect("i32 should be encoded");
    bytes
        .write_zig_zag_i64(-0x0102_0304_0506_0708)
        .expect("i64 should be encoded");
    bytes
        .write_zig_zag_i128(-0x0102_0304_0506_0708_1112_1314_1516_1718)
        .expect("i128 should be encoded");
    bytes
        .write_zig_zag_isize(isize::MIN)
        .expect("isize should be encoded");

    let mut reader = BufferedZigZagReader::<_, NonStrict>::with_capacity(
        Cursor::new(bytes),
        3,
    );

    assert!(!reader.is_strict());
    assert_eq!(
        i8::MIN,
        reader.read_i8_non_strict().expect("i8 should be read")
    );
    assert_eq!(
        -300,
        reader.read_i16_non_strict().expect("i16 should be read")
    );
    assert_eq!(
        -0x1f600,
        reader.read_i32_non_strict().expect("i32 should be read")
    );
    assert_eq!(
        -0x0102_0304_0506_0708,
        reader.read_i64_non_strict().expect("i64 should be read")
    );
    assert_eq!(
        -0x0102_0304_0506_0708_1112_1314_1516_1718,
        reader.read_i128_non_strict().expect("i128 should be read")
    );
    assert_eq!(
        isize::MIN,
        reader
            .read_isize_non_strict()
            .expect("isize should be read")
    );
}

#[test]
fn test_buffered_zig_zag_reader_accessors_raw_read_and_seek() {
    let mut reader =
        BufferedZigZagReader::<_, NonStrict>::new(Cursor::new(vec![1, 9]));

    assert!(!reader.is_strict());
    assert_eq!(0, reader.inner().position());
    assert_eq!(
        -1,
        reader
            .read_i8_non_strict()
            .expect("ZigZag value should be read")
    );
    assert_eq!(
        1,
        reader
            .seek_to(std::io::SeekFrom::Current(0))
            .expect("current seek should succeed")
    );
    let mut byte = [0_u8; 1];
    reader
        .read_fully(&mut byte)
        .expect("raw byte should be read");
    assert_eq!([9], byte);

    assert!(reader.inner().position() >= 2);
}

#[test]
fn test_buffered_zig_zag_reader_recovers_inner_and_unread_buffer() {
    let mut reader = BufferedZigZagReader::<_, NonStrict>::with_capacity(
        Cursor::new(vec![1, 9, 10]),
        19,
    );

    assert_eq!(
        -1,
        reader
            .read_i8_non_strict()
            .expect("ZigZag i8 should be read")
    );
    assert_eq!(3, reader.inner().position());

    let (inner, unread) = reader.into_parts();
    assert_eq!(3, inner.position());
    assert_eq!(&[9, 10], unread.readable());

    let mut reader = BufferedZigZagReader::<_, NonStrict>::with_capacity(
        Cursor::new(vec![1, 9]),
        19,
    );
    assert_eq!(
        -1,
        reader
            .read_i8_non_strict()
            .expect("ZigZag i8 should be read")
    );

    let (inner, unread) = reader.into_parts();
    assert_eq!(2, inner.position());
    assert_eq!(&[9], unread.readable());
}

#[test]
fn test_buffered_zig_zag_reader_reports_invalid_and_truncated_values() {
    let mut reader = BufferedZigZagReader::<_, Strict>::with_capacity(
        Cursor::new(vec![0x80, 0x00]),
        2,
    );
    assert!(reader.is_strict());
    assert_eq!(
        ErrorKind::InvalidData,
        reader
            .read_i16()
            .expect_err("non-canonical value should fail")
            .kind()
    );

    let mut reader = BufferedZigZagReader::<_, NonStrict>::with_capacity(
        Cursor::new(vec![0x80]),
        2,
    );
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader
            .read_i64_non_strict()
            .expect_err("truncated value should report EOF")
            .kind()
    );
}

#[test]
fn test_buffered_zig_zag_reader_consumes_invalid_payload_before_reporting_error()
 {
    let mut reader = BufferedZigZagReader::<_, Strict>::with_capacity(
        Cursor::new(vec![0x80, 0x00, 0x02]),
        2,
    );

    assert_eq!(
        ErrorKind::InvalidData,
        reader
            .read_i16()
            .expect_err("non-canonical value should fail")
            .kind()
    );
    assert_eq!(
        1,
        reader.read_i8().expect("next value should remain readable")
    );

    let mut reader = BufferedZigZagReader::<_, NonStrict>::with_capacity(
        Cursor::new(vec![0x80, 0x02, 0x02]),
        2,
    );
    assert_eq!(
        ErrorKind::InvalidData,
        reader
            .read_i8_non_strict()
            .expect_err("out-of-range ZigZag i8 encoding should fail")
            .kind()
    );
    assert_eq!(
        1,
        reader
            .read_i8_non_strict()
            .expect("next value should remain readable")
    );
}

#[test]
fn test_buffered_zig_zag_reader_reports_all_instantiated_error_paths() {
    let mut reader =
        BufferedZigZagReader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader
            .read_i8_non_strict()
            .expect_err("truncated i8")
            .kind()
    );

    let mut reader =
        BufferedZigZagReader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader
            .read_i16_non_strict()
            .expect_err("truncated i16")
            .kind()
    );

    let mut reader =
        BufferedZigZagReader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader
            .read_i32_non_strict()
            .expect_err("truncated i32")
            .kind()
    );

    let mut reader =
        BufferedZigZagReader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader
            .read_i64_non_strict()
            .expect_err("truncated i64")
            .kind()
    );

    let mut reader =
        BufferedZigZagReader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader
            .read_i128_non_strict()
            .expect_err("truncated i128")
            .kind()
    );

    let mut reader =
        BufferedZigZagReader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader
            .read_isize_non_strict()
            .expect_err("truncated isize")
            .kind()
    );

    let mut reader =
        BufferedZigZagReader::<_, NonStrict>::new(Cursor::new(vec![
        0x80u8;
        ZigZagCodec::<i8, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    ]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader
            .read_i8_non_strict()
            .expect_err("unterminated i8")
            .kind()
    );

    let mut reader =
        BufferedZigZagReader::<_, NonStrict>::new(Cursor::new(vec![
        0x80u8;
        ZigZagCodec::<i16, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    ]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader
            .read_i16_non_strict()
            .expect_err("unterminated i16")
            .kind()
    );

    let mut reader =
        BufferedZigZagReader::<_, NonStrict>::new(Cursor::new(vec![
        0x80u8;
        ZigZagCodec::<i32, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    ]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader
            .read_i32_non_strict()
            .expect_err("unterminated i32")
            .kind()
    );

    let mut reader =
        BufferedZigZagReader::<_, NonStrict>::new(Cursor::new(vec![
        0x80u8;
        ZigZagCodec::<i64, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    ]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader
            .read_i64_non_strict()
            .expect_err("unterminated i64")
            .kind()
    );

    let mut reader =
        BufferedZigZagReader::<_, NonStrict>::new(Cursor::new(vec![
        0x80u8;
        ZigZagCodec::<i128, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    ]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader
            .read_i128_non_strict()
            .expect_err("unterminated i128")
            .kind()
    );

    let mut reader =
        BufferedZigZagReader::<_, NonStrict>::new(Cursor::new(vec![
        0x80u8;
        ZigZagCodec::<isize, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    ]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader
            .read_isize_non_strict()
            .expect_err("unterminated isize")
            .kind()
    );
}
