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
use std::io::Read;
use std::io::SeekFrom;

use qubit_codec_binary::Leb128Codec;
use qubit_codec_binary::Leb128DecodeError;
use qubit_codec_binary::NonStrict;
use qubit_codec_binary::Strict;
use qubit_io_binary::BufferedLeb128Reader;
use qubit_io_binary::Leb128WriteExt;

use crate::stream::internal::seek_test_support_tests::RejectingCurrentSeekReader;

#[test]
fn test_seek_within_buffer_preserves_logical_position() {
    let mut reader = BufferedLeb128Reader::<_, NonStrict>::with_capacity(Cursor::new(vec![1, 2, 3, 4, 5]), 19);
    reader.read_exactly(&mut [0_u8; 1]).unwrap();
    assert_eq!(5, reader.inner().position());
    assert_eq!(1, reader.seek_to(SeekFrom::Current(0)).unwrap());
    assert_eq!(2, reader.seek_to(SeekFrom::Current(1)).unwrap());
    assert_eq!(1, reader.seek_to(SeekFrom::Current(-1)).unwrap());
    assert_eq!(5, reader.inner().position());
    let (_, unread) = reader.into_parts();
    assert_eq!(&[2, 3, 4, 5], unread.readable());
}

struct ByteThenErrorReader {
    byte: u8,
    returned: bool,
}

struct InterruptedOnceReader {
    bytes: Vec<u8>,
    position: usize,
    interrupted: bool,
}

struct ChunkedReader {
    bytes: Vec<u8>,
    position: usize,
    max_chunk_len: usize,
}

#[test]
fn test_seek_window_failure_preserves_unread() {
    for (consume_count, offset, expected_unread) in [(1, 1, &[2, 3, 4, 5][..]), (2, -1, &[3, 4, 5][..])] {
        let mut reader = BufferedLeb128Reader::<_, NonStrict>::with_capacity(
            RejectingCurrentSeekReader::new(vec![1, 2, 3, 4, 5]),
            5,
        );
        let mut consumed = vec![0; consume_count];
        reader.read_exactly(&mut consumed).expect("bytes should be consumed");
        let error = reader
            .seek_to(SeekFrom::Current(offset))
            .expect_err("current query must fail");
        assert_eq!(ErrorKind::Other, error.kind());
        let (_, unread): (_, qubit_io::Buffer<u8>) = reader.into_parts();
        assert_eq!(expected_unread, unread.readable());
    }

    for (consume_count, offset, expected_next) in [(1, 1, 2), (2, -1, 3)] {
        let mut reader = BufferedLeb128Reader::<_, NonStrict>::with_capacity(
            RejectingCurrentSeekReader::new(vec![1, 2, 3, 4, 5]),
            5,
        );
        let mut consumed = vec![0; consume_count];
        reader.read_exactly(&mut consumed).expect("bytes should be consumed");
        reader
            .seek_to(SeekFrom::Current(offset))
            .expect_err("current query must fail");
        let mut next = [0];
        reader
            .read_exactly(&mut next)
            .expect("unread byte should remain readable");
        assert_eq!([expected_next], next);
    }
}

impl ByteThenErrorReader {
    const fn new(byte: u8) -> Self {
        Self { byte, returned: false }
    }
}

impl InterruptedOnceReader {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            position: 0,
            interrupted: false,
        }
    }
}

impl ChunkedReader {
    fn new(bytes: Vec<u8>, max_chunk_len: usize) -> Self {
        Self {
            bytes,
            position: 0,
            max_chunk_len,
        }
    }
}

impl Read for ByteThenErrorReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if self.returned {
            return Err(Error::other("read failed"));
        }
        self.returned = true;
        buffer[0] = self.byte;
        Ok(1)
    }
}

impl Read for InterruptedOnceReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if !self.interrupted {
            self.interrupted = true;
            return Err(Error::new(ErrorKind::Interrupted, "interrupted once"));
        }
        let remaining = self.bytes.len() - self.position;
        let count = remaining.min(buffer.len());
        buffer[..count].copy_from_slice(&self.bytes[self.position..self.position + count]);
        self.position += count;
        Ok(count)
    }
}

impl Read for ChunkedReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let remaining = self.bytes.len() - self.position;
        let count = remaining.min(buffer.len()).min(self.max_chunk_len);
        buffer[..count].copy_from_slice(&self.bytes[self.position..self.position + count]);
        self.position += count;
        Ok(count)
    }
}

#[test]
fn test_buffered_leb128_reader_reads_values_across_buffer_boundaries() {
    let mut bytes = Vec::new();
    bytes.write_uleb_u8(u8::MAX).expect("u8 should be encoded");
    bytes.write_uleb_u16(300).expect("u16 should be encoded");
    bytes.write_uleb_u32(0x1f600).expect("u32 should be encoded");
    bytes
        .write_uleb_u64(0x0102_0304_0506_0708)
        .expect("u64 should be encoded");
    bytes
        .write_uleb_u128(0x0102_0304_0506_0708_1112_1314_1516_1718)
        .expect("u128 should be encoded");
    bytes.write_uleb_usize(usize::MAX).expect("usize should be encoded");
    bytes.write_sleb_i8(i8::MIN).expect("i8 should be encoded");
    bytes.write_sleb_i16(-300).expect("i16 should be encoded");
    bytes.write_sleb_i32(-0x1f600).expect("i32 should be encoded");
    bytes
        .write_sleb_i64(-0x0102_0304_0506_0708)
        .expect("i64 should be encoded");
    bytes
        .write_sleb_i128(-0x0102_0304_0506_0708_1112_1314_1516_1718)
        .expect("i128 should be encoded");
    bytes.write_sleb_isize(isize::MIN).expect("isize should be encoded");

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::with_capacity(Cursor::new(bytes), 3);

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
fn test_buffered_leb128_reader_accessors_raw_seek_and_string() {
    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![3, b'a', b'b', b'c', 9]));

    assert!(!reader.is_strict());
    assert_eq!(0, reader.inner().position());
    assert_eq!(
        "abc",
        reader
            .read_utf8_string_usize_non_strict(3)
            .expect("string should be read")
    );
    assert_eq!(
        4,
        reader
            .seek_to(std::io::SeekFrom::Current(0))
            .expect("current seek should succeed")
    );
    let mut byte = [0_u8; 1];
    reader.read_fully(&mut byte).expect("raw byte should be read");
    assert_eq!([9], byte);

    assert!(reader.inner().position() >= 5);
}

#[test]
fn test_buffered_leb128_reader_recovers_inner_and_unread_buffer() {
    let mut reader = BufferedLeb128Reader::<_, NonStrict>::with_capacity(Cursor::new(vec![1, 2, 3]), 19);

    assert_eq!(1, reader.read_u8_non_strict().expect("u8 should be read"));
    assert_eq!(3, reader.inner().position());

    let (inner, unread) = reader.into_parts();
    assert_eq!(3, inner.position());
    assert_eq!(&[2, 3], unread.readable());

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::with_capacity(Cursor::new(vec![4, 5]), 19);
    assert_eq!(4, reader.read_u8_non_strict().expect("u8 should be read"));

    let (inner, unread) = reader.into_parts();
    assert_eq!(2, inner.position());
    assert_eq!(&[5], unread.readable());
}

#[test]
fn test_buffered_leb128_reader_read_utf8_string_u64_reads_portable_length_prefix() {
    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![3, b'x', b'y', b'z']));

    assert_eq!(
        "xyz",
        reader
            .read_utf8_string_u64_non_strict(3)
            .expect("u64 length-prefixed string should be read")
    );
}

#[test]
fn test_buffered_leb128_reader_read_utf8_string_covers_strict_policy_paths() {
    let mut reader = BufferedLeb128Reader::<_, Strict>::with_capacity(Cursor::new(vec![3, b'a', b'b', b'c']), 2);

    assert_eq!(
        "abc",
        reader
            .read_utf8_string_usize(3)
            .expect("strict length-prefixed UTF-8 string should be read")
    );

    let mut reader = BufferedLeb128Reader::<_, Strict>::with_capacity(Cursor::new(vec![3, b'd', b'e', b'f']), 2);

    assert_eq!(
        "def",
        reader
            .read_utf8_string_u64(3)
            .expect("strict u64 length-prefixed UTF-8 string should be read")
    );

    let mut reader = BufferedLeb128Reader::<_, Strict>::with_capacity(Cursor::new(vec![0x80, 0x00]), 2);

    assert_eq!(
        ErrorKind::InvalidData,
        reader
            .read_utf8_string_usize(3)
            .expect_err("non-canonical strict string length should fail")
            .kind()
    );

    let mut reader = BufferedLeb128Reader::<_, Strict>::with_capacity(Cursor::new(vec![0x80, 0x00]), 2);

    assert_eq!(
        ErrorKind::InvalidData,
        reader
            .read_utf8_string_u64(3)
            .expect_err("non-canonical strict u64 string length should fail")
            .kind()
    );
}

#[test]
fn test_buffered_leb128_reader_reports_invalid_and_truncated_values() {
    let mut reader = BufferedLeb128Reader::<_, Strict>::with_capacity(Cursor::new(vec![0x80, 0x00]), 2);
    assert!(reader.is_strict());
    assert_eq!(
        ErrorKind::InvalidData,
        reader.read_u16().expect_err("non-canonical value should fail").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, Strict>::with_capacity(Cursor::new(vec![0x80, 0x00]), 2);
    let error = reader.read_u16().expect_err("non-canonical value should fail");
    let source = error.get_ref().expect("decode error should be preserved");
    assert!(
        source.downcast_ref::<Leb128DecodeError>().is_some(),
        "I/O error should preserve the original LEB128 decode error"
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::with_capacity(Cursor::new(vec![0x80]), 2);
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader
            .read_u64_non_strict()
            .expect_err("truncated value should report EOF")
            .kind()
    );
}

macro_rules! check_buffered_string_error_position {
    ($factory:expr, $method:ident) => {{
        for capacity in [19, 23] {
            for padding in [0, 18] {
                for (length, payload, max_len, kind, consumed, remainder) in [
                    (3, &b"abcQB"[..], 2, ErrorKind::InvalidData, 0, &b"abcQB"[..]),
                    (3, &b"\xffabQB"[..], 3, ErrorKind::InvalidData, 3, &b"QB"[..]),
                    (4, &b"ab"[..], 4, ErrorKind::UnexpectedEof, 2, &b""[..]),
                ] {
                    let mut bytes = vec![0xa5; padding];
                    bytes.push(length);
                    bytes.extend_from_slice(payload);
                    let expected_position = padding + 1 + consumed;

                    let mut reader = ($factory)(Cursor::new(bytes.clone()), capacity);
                    reader
                        .read_exactly(&mut vec![0; padding])
                        .expect("padding should be readable");
                    let error = reader.$method(max_len).expect_err("invalid string input should fail");
                    assert_eq!(kind, error.kind());
                    let (inner, unread) = reader.into_parts();
                    assert_eq!(
                        expected_position as u64,
                        inner.position() - unread.readable().len() as u64,
                        "logical position after string failure"
                    );
                    let mut actual_remainder = unread.readable().to_vec();
                    actual_remainder.extend_from_slice(&inner.get_ref()[inner.position() as usize..]);
                    assert_eq!(
                        remainder,
                        actual_remainder.as_slice(),
                        "unread bytes after string failure"
                    );

                    let mut reader = ($factory)(Cursor::new(bytes), capacity);
                    reader
                        .read_exactly(&mut vec![0; padding])
                        .expect("padding should be readable");
                    assert_eq!(
                        kind,
                        reader
                            .$method(max_len)
                            .expect_err("invalid string input should fail")
                            .kind()
                    );
                    let mut raw = vec![0; remainder.len()];
                    reader
                        .read_exactly(&mut raw)
                        .expect("remaining raw bytes should be readable");
                    assert_eq!(remainder, raw.as_slice(), "remaining bytes through raw API");
                }
            }
        }
    }};
}

#[test]
fn test_buffered_string_error_position_leb128() {
    check_buffered_string_error_position!(BufferedLeb128Reader::<_, Strict>::with_capacity, read_utf8_string_u64);
    check_buffered_string_error_position!(BufferedLeb128Reader::<_, Strict>::with_capacity, read_utf8_string_usize);
    check_buffered_string_error_position!(
        BufferedLeb128Reader::<_, NonStrict>::with_capacity,
        read_utf8_string_u64_non_strict
    );
    check_buffered_string_error_position!(
        BufferedLeb128Reader::<_, NonStrict>::with_capacity,
        read_utf8_string_usize_non_strict
    );
}

#[test]
fn test_buffered_leb128_reader_reports_refill_error_after_partial_payload() {
    let mut reader = BufferedLeb128Reader::<_, NonStrict>::with_capacity(ByteThenErrorReader::new(0x80), 2);

    let error = reader
        .read_u16_non_strict()
        .expect_err("reader error after a partial payload should be returned");

    assert_eq!(ErrorKind::Other, error.kind());
}

#[test]
fn test_buffered_leb128_reader_refills_without_backshift_when_tail_has_room() {
    let inner = ChunkedReader::new(vec![0x80, 0x01], 1);
    let mut reader = BufferedLeb128Reader::<_, NonStrict>::with_capacity(inner, 19);

    assert_eq!(128, reader.read_u16_non_strict().expect("split value should be read"));
}

#[test]
fn test_buffered_leb128_reader_retries_interrupted_refill() {
    let inner = InterruptedOnceReader::new(vec![0x80, 0x01]);
    let mut reader = BufferedLeb128Reader::<_, NonStrict>::with_capacity(inner, 19);

    assert_eq!(
        128,
        reader.read_u16_non_strict().expect("read should retry interruption")
    );
}

#[test]
fn test_buffered_leb128_reader_consumes_invalid_payload_before_reporting_error() {
    let mut reader = BufferedLeb128Reader::<_, Strict>::with_capacity(Cursor::new(vec![0x80, 0x00, 0x01]), 2);

    assert_eq!(
        ErrorKind::InvalidData,
        reader.read_u16().expect_err("non-canonical value should fail").kind()
    );
    assert_eq!(1, reader.read_u8().expect("next value should remain readable"));

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::with_capacity(Cursor::new(vec![0x80, 0x02, 0x01]), 2);
    assert_eq!(
        ErrorKind::InvalidData,
        reader
            .read_u8_non_strict()
            .expect_err("out-of-range u8 encoding should fail")
            .kind()
    );
    assert_eq!(
        1,
        reader.read_u8_non_strict().expect("next value should remain readable")
    );
}

#[test]
fn test_buffered_leb128_reader_reports_all_instantiated_error_paths() {
    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader.read_u8_non_strict().expect_err("truncated u8").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader.read_u16_non_strict().expect_err("truncated u16").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader.read_u32_non_strict().expect_err("truncated u32").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader.read_u64_non_strict().expect_err("truncated u64").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader.read_u128_non_strict().expect_err("truncated u128").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader.read_usize_non_strict().expect_err("truncated usize").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader.read_i8_non_strict().expect_err("truncated i8").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader.read_i16_non_strict().expect_err("truncated i16").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader.read_i32_non_strict().expect_err("truncated i32").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader.read_i64_non_strict().expect_err("truncated i64").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader.read_i128_non_strict().expect_err("truncated i128").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![0x80]));
    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader.read_isize_non_strict().expect_err("truncated isize").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![
        0x80u8;
        Leb128Codec::<u8, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    ]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader.read_u8_non_strict().expect_err("unterminated u8").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![
        0x80u8;
        Leb128Codec::<u16, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    ]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader.read_u16_non_strict().expect_err("unterminated u16").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![
        0x80u8;
        Leb128Codec::<u32, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    ]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader.read_u32_non_strict().expect_err("unterminated u32").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![
        0x80u8;
        Leb128Codec::<u64, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    ]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader.read_u64_non_strict().expect_err("unterminated u64").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![
        0x80u8;
        Leb128Codec::<u128, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    ]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader.read_u128_non_strict().expect_err("unterminated u128").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![
        0x80u8;
        Leb128Codec::<usize, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    ]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader.read_usize_non_strict().expect_err("unterminated usize").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![
        0x80u8;
        Leb128Codec::<i8, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    ]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader.read_i8_non_strict().expect_err("unterminated i8").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![
        0x80u8;
        Leb128Codec::<i16, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    ]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader.read_i16_non_strict().expect_err("unterminated i16").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![
        0x80u8;
        Leb128Codec::<i32, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    ]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader.read_i32_non_strict().expect_err("unterminated i32").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![
        0x80u8;
        Leb128Codec::<i64, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    ]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader.read_i64_non_strict().expect_err("unterminated i64").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![
        0x80u8;
        Leb128Codec::<i128, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    ]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader.read_i128_non_strict().expect_err("unterminated i128").kind()
    );

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![
        0x80u8;
        Leb128Codec::<isize, NonStrict>::MAX_DECODE_UNITS_PER_VALUE
    ]));
    assert_eq!(
        ErrorKind::InvalidData,
        reader.read_isize_non_strict().expect_err("unterminated isize").kind()
    );
}
