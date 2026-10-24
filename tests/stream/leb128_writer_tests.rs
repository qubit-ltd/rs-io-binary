// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::io::Error;
use std::io::ErrorKind;
use std::io::Write;

use qubit_io::Output;
use qubit_io::Seekable;
use qubit_io_binary::Leb128Writer;

struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buffer: &[u8]) -> std::io::Result<usize> {
        Err(Error::other("write failed"))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_leb128_writer_writes_all_methods_and_exposes_accessors() {
    let mut writer = Leb128Writer::new(Vec::new());

    assert_eq!(0, writer.inner().len());
    writer.inner_mut().extend_from_slice(&[]);
    writer.write_u8(u8::MAX).expect("u8 should be written");
    writer.write_u16(u16::MAX).expect("u16 should be written");
    writer.write_u32(u32::MAX).expect("u32 should be written");
    writer.write_u64(u64::MAX).expect("u64 should be written");
    writer.write_u128(u128::MAX).expect("u128 should be written");
    writer.write_usize(usize::MAX).expect("usize should be written");
    writer.write_i8(i8::MIN).expect("i8 should be written");
    writer.write_i16(i16::MIN).expect("i16 should be written");
    writer.write_i32(i32::MIN).expect("i32 should be written");
    writer.write_i64(i64::MIN).expect("i64 should be written");
    writer.write_i128(i128::MIN).expect("i128 should be written");
    writer.write_isize(isize::MIN).expect("isize should be written");

    let mut expected = Vec::new();
    expected.extend_from_slice(&[0xff, 0x01]); // u8::MAX
    expected.extend_from_slice(&[0xff, 0xff, 0x03]); // u16::MAX
    expected.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x0f]); // u32::MAX
    expected.extend_from_slice(&[0xff; 9]);
    expected.push(0x01); // u64::MAX
    expected.extend_from_slice(&[0xff; 18]);
    expected.push(0x03); // u128::MAX
    #[cfg(target_pointer_width = "16")]
    expected.extend_from_slice(&[0xff, 0xff, 0x03]);
    #[cfg(target_pointer_width = "32")]
    expected.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x0f]);
    #[cfg(target_pointer_width = "64")]
    {
        expected.extend_from_slice(&[0xff; 9]);
        expected.push(0x01);
    }
    expected.extend_from_slice(&[0x80, 0x7f]); // i8::MIN
    expected.extend_from_slice(&[0x80, 0x80, 0x7e]); // i16::MIN
    expected.extend_from_slice(&[0x80, 0x80, 0x80, 0x80, 0x78]); // i32::MIN
    expected.extend_from_slice(&[0x80; 9]);
    expected.push(0x7f); // i64::MIN
    expected.extend_from_slice(&[0x80; 18]);
    expected.push(0x7e); // i128::MIN
    #[cfg(target_pointer_width = "16")]
    expected.extend_from_slice(&[0x80, 0x80, 0x7e]);
    #[cfg(target_pointer_width = "32")]
    expected.extend_from_slice(&[0x80, 0x80, 0x80, 0x80, 0x78]);
    #[cfg(target_pointer_width = "64")]
    {
        expected.extend_from_slice(&[0x80; 9]);
        expected.push(0x7f);
    }

    assert_eq!(expected, writer.into_inner());
}

#[test]
fn test_leb128_writer_signed_canonical_boundaries() {
    let mut writer = Leb128Writer::new(Vec::new());
    for value in [-65_i16, -64, 63, 64] {
        writer.write_i16(value).expect("signed boundary should encode");
    }

    assert_eq!(vec![0xbf, 0x7f, 0x40, 0x3f, 0xc0, 0x00], writer.into_inner());
}

#[test]
fn test_leb128_writer_returns_writer_error() {
    let mut writer = Leb128Writer::new(FailingWriter);

    let error = writer.write_u16(300).expect_err("writer error should be returned");

    assert_eq!(ErrorKind::Other, error.kind());
}

#[test]
fn test_leb128_writer_write_utf8_string_writes_length_prefixed_payload() {
    let mut writer = Leb128Writer::new(Vec::new());

    writer
        .write_utf8_string_usize("hé")
        .expect("writing a length-prefixed UTF-8 string should succeed");

    assert_eq!(writer.into_inner(), vec![3, b'h', 0xC3, 0xA9]);
}

#[test]
fn test_leb128_writer_write_utf8_string_u64_writes_portable_length_prefix() {
    let mut writer = Leb128Writer::new(Vec::new());

    writer
        .write_utf8_string_u64("hé")
        .expect("writing a u64 length-prefixed UTF-8 string should succeed");

    assert_eq!(writer.into_inner(), vec![3, b'h', 0xC3, 0xA9]);
}

#[test]
fn test_leb128_writer_write_and_seek_delegate_to_inner_writer() {
    let mut writer = Leb128Writer::new(std::io::Cursor::new(vec![0; 4]));

    Seekable::seek_to(&mut writer, std::io::SeekFrom::Start(1)).expect("seeking through Leb128Writer should succeed");
    Output::write_fully(&mut writer, b"xy").expect("writing through Leb128Writer should succeed");
    Output::flush(&mut writer).expect("flushing through Leb128Writer should succeed");

    let cursor = writer.into_inner();
    assert_eq!(cursor.into_inner(), vec![0, b'x', b'y', 0]);
}
