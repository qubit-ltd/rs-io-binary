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
};
use qubit_io::{
    Input,
    Seekable,
};
use qubit_io_binary::{
    Leb128Reader,
    Leb128Writer,
};

#[test]
fn test_leb128_reader_reads_all_methods() {
    let mut writer = Leb128Writer::new(Vec::new());
    writer.write_u8(u8::MAX).expect("u8 should be written");
    writer.write_u16(300).expect("u16 should be written");
    writer.write_u32(0x1f600).expect("u32 should be written");
    writer
        .write_u64(0x0102_0304_0506_0708)
        .expect("u64 should be written");
    writer
        .write_u128(0x0102_0304_0506_0708_1112_1314_1516_1718)
        .expect("u128 should be written");
    writer
        .write_usize(usize::MAX)
        .expect("usize should be written");
    writer.write_i8(i8::MIN).expect("i8 should be written");
    writer.write_i16(-300).expect("i16 should be written");
    writer.write_i32(-0x1f600).expect("i32 should be written");
    writer
        .write_i64(-0x0102_0304_0506_0708)
        .expect("i64 should be written");
    writer
        .write_i128(-0x0102_0304_0506_0708_1112_1314_1516_1718)
        .expect("i128 should be written");
    writer
        .write_isize(isize::MIN)
        .expect("isize should be written");

    let mut reader =
        Leb128Reader::<_, NonStrict>::new(Cursor::new(writer.into_inner()));
    assert!(!reader.is_strict());
    assert_eq!(u8::MAX, reader.read_u8_non_strict().expect("u8 should be read"));
    assert_eq!(300, reader.read_u16_non_strict().expect("u16 should be read"));
    assert_eq!(0x1f600, reader.read_u32_non_strict().expect("u32 should be read"));
    assert_eq!(
        0x0102_0304_0506_0708,
        reader.read_u64_non_strict().expect("u64 should be read")
    );
    assert_eq!(
        0x0102_0304_0506_0708_1112_1314_1516_1718,
        reader.read_u128_non_strict().expect("u128 should be read")
    );
    assert_eq!(
        usize::MAX,
        reader.read_usize_non_strict().expect("usize should be read")
    );
    assert_eq!(i8::MIN, reader.read_i8_non_strict().expect("i8 should be read"));
    assert_eq!(-300, reader.read_i16_non_strict().expect("i16 should be read"));
    assert_eq!(-0x1f600, reader.read_i32_non_strict().expect("i32 should be read"));
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
        reader.read_isize_non_strict().expect("isize should be read")
    );
}

#[test]
fn test_leb128_reader_exposes_accessors_and_reports_errors() {
    let mut reader = Leb128Reader::<_, Strict>::new(Cursor::new(vec![1]));
    assert_eq!(1, reader.read_u16().expect("strict u16 should be read"));

    let mut reader =
        Leb128Reader::<_, Strict>::new(Cursor::new(vec![0x80, 0x00]));
    assert!(reader.is_strict());
    assert_eq!(0, reader.inner().position());
    reader.inner_mut().set_position(0);
    assert_eq!(
        ErrorKind::InvalidData,
        reader
            .read_u16()
            .expect_err("non-canonical value should fail")
            .kind()
    );
    assert_eq!(2, reader.into_inner().position());

    let mut reader = Leb128Reader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader
            .read_u64_non_strict()
            .expect_err("truncated value should report EOF")
            .kind()
    );

    let mut reader =
        Leb128Reader::<_, NonStrict>::new(Cursor::new(vec![0x80, 0x80, 0x80]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader
            .read_u16_non_strict()
            .expect_err("unterminated max-width value should fail")
            .kind()
    );
}

#[test]
fn test_leb128_reader_read_utf8_string_reads_length_prefixed_payload() {
    let bytes = vec![3, b'h', 0xC3, 0xA9];
    let mut reader = qubit_io_binary::Leb128Reader::<
        _,
        qubit_codec_binary::NonStrict,
    >::new(std::io::Cursor::new(bytes));

    let text = reader
        .read_utf8_string_usize_non_strict(3)
        .expect("reading a length-prefixed UTF-8 string should succeed");

    assert_eq!(text, "hé");
}

#[test]
fn test_leb128_reader_read_utf8_string_u64_reads_portable_length_prefix() {
    let bytes = vec![3, b'h', 0xC3, 0xA9];
    let mut reader = qubit_io_binary::Leb128Reader::<
        _,
        qubit_codec_binary::NonStrict,
    >::new(std::io::Cursor::new(bytes));

    let text = reader
        .read_utf8_string_u64_non_strict(3)
        .expect("reading a u64 length-prefixed UTF-8 string should succeed");

    assert_eq!(text, "hé");
}

#[test]
fn test_leb128_reader_read_utf8_string_covers_strict_policy_paths() {
    let mut reader =
        Leb128Reader::<_, Strict>::new(Cursor::new(vec![3, b'a', b'b', b'c']));

    let text = reader
        .read_utf8_string_usize(3)
        .expect("strict length-prefixed UTF-8 string should succeed");

    assert_eq!("abc", text);

    let mut reader =
        Leb128Reader::<_, Strict>::new(Cursor::new(vec![3, b'd', b'e', b'f']));
    let text = reader
        .read_utf8_string_u64(3)
        .expect("strict u64 length-prefixed UTF-8 string should succeed");

    assert_eq!("def", text);

    let mut reader =
        Leb128Reader::<_, Strict>::new(Cursor::new(vec![0x80, 0x00]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader
            .read_utf8_string_usize(3)
            .expect_err("non-canonical strict string length should fail")
            .kind()
    );

    let mut reader =
        Leb128Reader::<_, Strict>::new(Cursor::new(vec![0x80, 0x00]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader
            .read_utf8_string_u64(3)
            .expect_err("non-canonical strict length should fail")
            .kind()
    );
}

#[test]
fn test_leb128_reader_read_and_seek_delegate_to_inner_reader() {
    let mut reader = qubit_io_binary::Leb128Reader::<
        _,
        qubit_codec_binary::NonStrict,
    >::new(std::io::Cursor::new(vec![1, 2, 3, 4]));

    Seekable::seek_to(&mut reader, std::io::SeekFrom::Start(1))
        .expect("seeking through Leb128Reader should succeed");
    let mut bytes = [0_u8; 2];
    Input::read_fully(&mut reader, &mut bytes)
        .expect("reading through Leb128Reader should succeed");

    assert_eq!(bytes, [2, 3]);
}
