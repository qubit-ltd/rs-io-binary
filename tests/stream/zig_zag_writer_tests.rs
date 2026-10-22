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
use qubit_io_binary::ZigZagWriter;

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
fn test_zig_zag_writer_writes_all_methods_and_exposes_accessors() {
    let mut writer = ZigZagWriter::new(Vec::new());

    assert_eq!(0, writer.inner().len());
    writer.write_i8(i8::MIN).expect("i8 should be written");
    writer.write_i16(-300).expect("i16 should be written");
    writer.write_i32(-0x1f600).expect("i32 should be written");
    writer.write_i64(i64::MIN).expect("i64 should be written");
    writer.write_i128(i128::MIN).expect("i128 should be written");
    writer.write_isize(isize::MIN).expect("isize should be written");

    let mut expected = vec![
        0xff, 0x01, // i8::MIN
        0xd7, 0x04, // -300
        0xff, 0xd7, 0x0f, // -0x1f600
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01, // i64::MIN
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0x03, // i128::MIN
    ];
    if usize::BITS == 64 {
        expected.extend([0xff; 9]);
        expected.push(0x01);
    } else {
        expected.extend([0xff; 4]);
        expected.push(0x0f);
    }

    assert_eq!(expected, writer.into_inner());
}

#[test]
fn test_zig_zag_writer_returns_writer_error() {
    let mut writer = ZigZagWriter::new(FailingWriter);

    let error = writer.write_i16(-300).expect_err("writer error should be returned");

    assert_eq!(ErrorKind::Other, error.kind());
}

#[test]
fn test_zig_zag_writer_write_and_seek_delegate_to_inner_writer() {
    let mut writer = ZigZagWriter::new(std::io::Cursor::new(vec![0; 4]));

    Seekable::seek_to(&mut writer, std::io::SeekFrom::Start(1)).expect("seeking through ZigZagWriter should succeed");
    Output::write_fully(&mut writer, b"xy").expect("writing through ZigZagWriter should succeed");
    Output::flush(&mut writer).expect("flushing through ZigZagWriter should succeed");

    let cursor = writer.into_inner();
    assert_eq!(cursor.into_inner(), vec![0, b'x', b'y', 0]);
}
