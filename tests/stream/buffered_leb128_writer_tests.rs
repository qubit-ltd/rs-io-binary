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
use std::io::SeekFrom;
use std::io::Write;

use qubit_io::BufferedOutputRecovery;
use qubit_io::Output;
use qubit_io_binary::BufferedLeb128Writer;
use qubit_io_binary::Leb128WriteExt;

use super::internal::string_write_test_support_tests::PrefixErrorOutput;

#[test]
fn test_seek_current_zero_preserves_pending_bytes() {
    let mut writer = BufferedLeb128Writer::new(Cursor::new(Vec::new()));
    writer.write_u8(1).unwrap();
    assert_eq!(1, writer.seek_to(SeekFrom::Current(0)).unwrap());
    assert_eq!(0, writer.inner().position());
    assert!(writer.inner().get_ref().is_empty());
    let (_, recovery) = writer.into_parts();
    match recovery {
        BufferedOutputRecovery::Pending(bytes) => assert_eq!(&[1], bytes.readable()),
        BufferedOutputRecovery::Indeterminate(_) => {
            panic!("position query must preserve determinate pending bytes")
        }
    }
}

struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buffer: &[u8]) -> std::io::Result<usize> {
        Err(Error::other("write failed"))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Default)]
struct PanicOutput {
    bytes: Vec<u8>,
}

impl Output for PanicOutput {
    type Item = u8;

    unsafe fn write_unchecked(&mut self, input: &[u8], index: usize, count: usize) -> std::io::Result<usize> {
        let committed = count.min(1);
        self.bytes.extend_from_slice(&input[index..index + committed]);
        panic!("scripted output panic after committing a prefix");
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_buffered_leb128_writer_writes_values_across_buffer_boundaries() {
    let mut expected = Vec::new();
    expected.write_uleb_u8(u8::MAX).expect("u8 should be encoded");
    expected.write_uleb_u16(300).expect("u16 should be encoded");
    expected.write_uleb_u32(0x1f600).expect("u32 should be encoded");
    expected
        .write_uleb_u64(0x0102_0304_0506_0708)
        .expect("u64 should be encoded");
    expected
        .write_uleb_u128(0x0102_0304_0506_0708_1112_1314_1516_1718)
        .expect("u128 should be encoded");
    expected.write_uleb_usize(usize::MAX).expect("usize should be encoded");
    expected.write_sleb_i8(i8::MIN).expect("i8 should be encoded");
    expected.write_sleb_i16(-300).expect("i16 should be encoded");
    expected.write_sleb_i32(-0x1f600).expect("i32 should be encoded");
    expected
        .write_sleb_i64(-0x0102_0304_0506_0708)
        .expect("i64 should be encoded");
    expected
        .write_sleb_i128(-0x0102_0304_0506_0708_1112_1314_1516_1718)
        .expect("i128 should be encoded");
    expected.write_sleb_isize(isize::MIN).expect("isize should be encoded");

    let mut writer = BufferedLeb128Writer::with_capacity(Vec::new(), 3);
    writer.write_u8(u8::MAX).expect("u8 should be written");
    writer.write_u16(300).expect("u16 should be written");
    writer.write_u32(0x1f600).expect("u32 should be written");
    writer.write_u64(0x0102_0304_0506_0708).expect("u64 should be written");
    writer
        .write_u128(0x0102_0304_0506_0708_1112_1314_1516_1718)
        .expect("u128 should be written");
    writer.write_usize(usize::MAX).expect("usize should be written");
    writer.write_i8(i8::MIN).expect("i8 should be written");
    writer.write_i16(-300).expect("i16 should be written");
    writer.write_i32(-0x1f600).expect("i32 should be written");
    writer.write_i64(-0x0102_0304_0506_0708).expect("i64 should be written");
    writer
        .write_i128(-0x0102_0304_0506_0708_1112_1314_1516_1718)
        .expect("i128 should be written");
    writer.write_isize(isize::MIN).expect("isize should be written");

    writer.flush().expect("writer should flush");
    assert_eq!(expected, writer.inner().clone());
}

#[test]
fn test_buffered_leb128_writer_accessors_write_all_seek_and_string() {
    let mut writer = BufferedLeb128Writer::new(Cursor::new(Vec::new()));

    assert_eq!(0, writer.inner().position());
    writer
        .write_utf8_string_usize("abc")
        .expect("string should be buffered");
    assert_eq!(1, writer.write(&[9]).expect("raw byte should be buffered"));
    writer.write_fully(&[10]).expect("raw byte should be buffered");
    assert_eq!(
        6,
        writer
            .seek_to(std::io::SeekFrom::Current(0))
            .expect("logical position query should succeed")
    );

    writer.flush().expect("flush should write all bytes");

    assert_eq!(vec![3, b'a', b'b', b'c', 9, 10], writer.inner().clone().into_inner());
}

#[test]
fn test_buffered_leb128_writer_into_parts_returns_pending_bytes_without_flushing() {
    let mut writer = BufferedLeb128Writer::new(Cursor::new(Vec::new()));

    writer.write_u64(300).expect("u64 should be buffered");
    assert_eq!(0, writer.inner().position());

    let (inner, pending) = writer.into_parts();
    let pending = match pending {
        qubit_io::BufferedOutputRecovery::Pending(buffer) => buffer,
        qubit_io::BufferedOutputRecovery::Indeterminate(_) => {
            panic!("unexpected indeterminate recovery")
        }
    };
    assert_eq!(b"\xAC\x02", pending.readable());
    assert!(inner.into_inner().is_empty());
}

#[test]
fn test_buffered_leb128_writer_flush_error_leaves_writer_available_for_retry() {
    let mut writer = BufferedLeb128Writer::with_capacity(FailingWriter, 8);
    writer.write_u64(300).expect("u64 should be buffered");

    let error = writer.flush().expect_err("flush should report write failure");

    assert_eq!(ErrorKind::Other, error.kind());
    let retry_error = writer
        .flush()
        .expect_err("writer should remain retryable after a flush failure");
    assert_eq!(ErrorKind::Other, retry_error.kind());
}

#[test]
fn test_buffered_leb128_writer_returns_writer_error() {
    let mut writer = BufferedLeb128Writer::with_capacity(FailingWriter, 8);

    writer.write_u64(300).expect("value should be buffered");
    let error = writer.flush().expect_err("flush should fail");

    assert_eq!(ErrorKind::Other, error.kind());
}

#[test]
fn test_buffered_leb128_writer_flushes_before_encoded_value_when_full() {
    let mut writer = BufferedLeb128Writer::with_capacity(Vec::new(), 19);

    writer.write_fully(&[1; 18]).expect("initial bytes should be buffered");
    writer.write_u8(1).expect("encoded value should flush then buffer");
    assert_eq!(&[1; 18], writer.inner().as_slice());

    let mut expected = vec![1; 18];
    expected.push(1);
    writer.flush().expect("writer should flush");
    assert_eq!(expected, writer.inner().clone());
}

#[test]
fn test_buffered_leb128_writer_reports_automatic_flush_error_before_string_prefix() {
    let mut writer = BufferedLeb128Writer::with_capacity(FailingWriter, 19);

    writer.write_fully(&[1; 18]).expect("initial bytes should be buffered");
    let error = writer
        .write_utf8_string_usize("a")
        .expect_err("automatic flush should fail before the prefix");

    assert_eq!(ErrorKind::Other, error.kind());
    let (_, recovery) = writer.into_parts();
    match recovery {
        BufferedOutputRecovery::Pending(buffer) => assert_eq!(&[1; 18], buffer.readable()),
        BufferedOutputRecovery::Indeterminate(_) => {
            panic!("I/O error must retain determinate pending bytes")
        }
    }
}

#[test]
fn test_buffered_leb128_writer_write_utf8_string_u64_writes_portable_length_prefix() {
    let mut writer = BufferedLeb128Writer::new(Vec::new());

    writer
        .write_utf8_string_u64("hé")
        .expect("u64 length-prefixed string should be written");

    writer.flush().expect("writer should flush");
    assert_eq!(vec![3, b'h', 0xC3, 0xA9], writer.inner().clone());
}

#[test]
fn test_buffered_leb128_writer_reports_string_prefix_and_capacity_errors() {
    let value = "x".repeat(300);
    let mut writer = BufferedLeb128Writer::with_capacity(FailingWriter, 19);
    writer
        .write_fully(&[1; 18])
        .expect("the buffer should accept the initial bytes");
    let error = writer
        .write_utf8_string_usize(&value)
        .expect_err("usize prefix flush error should be returned");
    assert_eq!(ErrorKind::Other, error.kind());

    let mut writer = BufferedLeb128Writer::with_capacity(FailingWriter, 19);
    writer
        .write_fully(&[1; 18])
        .expect("the buffer should accept the initial bytes");
    let error = writer
        .write_utf8_string_u64(&value)
        .expect_err("u64 prefix flush error should be returned");
    assert_eq!(ErrorKind::Other, error.kind());

    let result = BufferedLeb128Writer::try_with_capacity(Vec::<u8>::new(), usize::MAX);
    assert!(result.is_err());
}

#[test]
fn test_into_parts_preserves_indeterminate_after_inner_write_panics() {
    let mut writer = BufferedLeb128Writer::new(PanicOutput::default());
    writer.write_u64(300).expect("the encoded value should be buffered");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| qubit_io::Output::flush(&mut writer)));
    assert!(result.is_err());

    let (inner, recovery) = writer.into_parts();
    assert_eq!([0xAC], inner.bytes.as_slice());
    assert!(matches!(recovery, qubit_io::BufferedOutputRecovery::Indeterminate(_)));
}

#[test]
fn test_buffered_leb128_string_payload_failure_is_not_completed_by_flush() {
    for use_u64 in [false, true] {
        let mut writer = BufferedLeb128Writer::with_capacity(PrefixErrorOutput::new(3), 19);
        let payload = "x".repeat(40);
        let result = if use_u64 {
            writer.write_utf8_string_u64(&payload)
        } else {
            writer.write_utf8_string_usize(&payload)
        };
        assert_eq!(ErrorKind::Other, result.expect_err("payload should fail").kind());
        assert_eq!(&[40, b'x', b'x'], writer.inner().bytes());
        writer
            .flush()
            .expect("an empty buffer can flush without completing the field");
        let (output, recovery) = writer.into_parts();
        assert_eq!(&[40, b'x', b'x'], output.bytes());
        match recovery {
            BufferedOutputRecovery::Pending(buffer) => assert!(buffer.readable().is_empty()),
            BufferedOutputRecovery::Indeterminate(_) => {
                panic!("ordinary errors must not imply a panic")
            }
        }
    }
}
