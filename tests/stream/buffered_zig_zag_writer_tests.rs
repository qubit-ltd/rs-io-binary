// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::io::Cursor;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Write;

use qubit_io_binary::BufferedZigZagWriter;
use qubit_io_binary::ZigZagWriteExt;

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
fn test_buffered_zig_zag_writer_writes_values_across_buffer_boundaries() {
    let mut expected = Vec::new();
    expected
        .write_zig_zag_i8(i8::MIN)
        .expect("i8 should be encoded");
    expected
        .write_zig_zag_i16(-300)
        .expect("i16 should be encoded");
    expected
        .write_zig_zag_i32(-0x1f600)
        .expect("i32 should be encoded");
    expected
        .write_zig_zag_i64(-0x0102_0304_0506_0708)
        .expect("i64 should be encoded");
    expected
        .write_zig_zag_i128(-0x0102_0304_0506_0708_1112_1314_1516_1718)
        .expect("i128 should be encoded");
    expected
        .write_zig_zag_isize(isize::MIN)
        .expect("isize should be encoded");

    let mut writer = BufferedZigZagWriter::with_capacity(Vec::new(), 3);
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

    writer.flush().expect("writer should flush");
    assert_eq!(expected, writer.inner().clone());
}

#[test]
fn test_buffered_zig_zag_writer_accessors_write_all_and_seek() {
    let mut writer = BufferedZigZagWriter::new(Cursor::new(Vec::new()));

    assert_eq!(0, writer.inner().position());
    writer
        .write_i8(-1)
        .expect("ZigZag value should be buffered");
    assert_eq!(1, writer.write(&[9]).expect("raw byte should be buffered"));
    writer
        .write_fully(&[10])
        .expect("raw byte should be buffered");
    assert_eq!(
        3,
        writer
            .seek_to(std::io::SeekFrom::Current(0))
            .expect("seek should flush pending bytes")
    );

    writer.flush().expect("flush should write all bytes");

    assert_eq!(vec![1, 9, 10], writer.inner().clone().into_inner());
}

#[test]
fn test_buffered_zig_zag_writer_into_parts_returns_pending_bytes_without_flushing()
 {
    let mut writer = BufferedZigZagWriter::new(Cursor::new(Vec::new()));

    writer.write_i64(-300).expect("i64 should be buffered");
    assert_eq!(0, writer.inner().position());

    let (inner, pending) = writer.into_parts();
    assert_eq!(b"\xD7\x04", pending.readable());
    assert!(inner.into_inner().is_empty());
}

#[test]
fn test_buffered_zig_zag_writer_flush_error_leaves_writer_available_for_retry()
{
    let mut writer = BufferedZigZagWriter::with_capacity(FailingWriter, 8);
    writer.write_i64(-300).expect("i64 should be buffered");

    let error = writer
        .flush()
        .expect_err("flush should report write failure");

    assert_eq!(ErrorKind::Other, error.kind());
    let retry_error = writer
        .flush()
        .expect_err("writer should remain retryable after a flush failure");
    assert_eq!(ErrorKind::Other, retry_error.kind());
}

#[test]
fn test_buffered_zig_zag_writer_returns_writer_error() {
    let mut writer = BufferedZigZagWriter::with_capacity(FailingWriter, 8);

    writer.write_i64(-300).expect("value should be buffered");
    let error = writer.flush().expect_err("flush should fail");

    assert_eq!(ErrorKind::Other, error.kind());
}

#[test]
fn test_buffered_zig_zag_writer_flushes_before_encoded_value_when_full() {
    let mut writer = BufferedZigZagWriter::with_capacity(Vec::new(), 19);

    writer
        .write_fully(&[1; 18])
        .expect("initial bytes should be buffered");
    writer
        .write_i8(-1)
        .expect("encoded value should flush then buffer");

    let mut expected = vec![1; 18];
    expected.push(1);
    writer.flush().expect("writer should flush");
    assert_eq!(expected, writer.inner().clone());
}
