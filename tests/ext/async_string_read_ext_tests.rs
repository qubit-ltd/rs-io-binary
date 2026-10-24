// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::ErrorKind;
use std::task::Poll;

use qubit_codec::ByteOrder;
use qubit_io_binary::AsyncStringReadExt;
use qubit_io_binary::StringWriteExt;

use super::internal::async_io_test_support_tests::ChunkedAsyncInput;
use super::internal::async_io_test_support_tests::complete;
use super::internal::async_io_test_support_tests::poll_once;

fn string_fixture() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.write_utf8_payload("payload").unwrap();
    bytes.write_utf8_string_uleb_usize("uleb").unwrap();
    bytes.write_utf8_string_uleb_usize("uleb-strict").unwrap();
    bytes.write_utf8_string_uleb_u64("uleb-u64").unwrap();
    bytes.write_utf8_string_uleb_u64("uleb-u64-strict").unwrap();
    bytes.write_string_with_u16_len("u16-be", ByteOrder::BigEndian).unwrap();
    bytes.write_string_with_u16_len_be("fixed-u16-be").unwrap();
    bytes.write_string_with_u16_len_le("fixed-u16-le").unwrap();
    bytes
        .write_string_with_u32_len("u32-le", ByteOrder::LittleEndian)
        .unwrap();
    bytes.write_string_with_u32_len_be("fixed-u32-be").unwrap();
    bytes.write_string_with_u32_len_le("fixed-u32-le").unwrap();
    bytes
}

#[test]
fn test_async_string_read_reuses_payload_buffer() {
    let mut input = ChunkedAsyncInput::new(b"hello world".to_vec());
    let mut payload = Vec::with_capacity(32);

    complete(input.read_utf8_payload_into_async(&mut payload, 5, 32)).expect("first payload should be read");
    let allocation = payload.as_ptr();
    assert_eq!(b"hello", payload.as_slice());

    complete(input.read_utf8_payload_into_async(&mut payload, 6, 32)).expect("second payload should be read");
    assert_eq!(allocation, payload.as_ptr());
    assert_eq!(b" world", payload.as_slice());

    let error = complete(input.read_utf8_payload_into_async(&mut payload, 1, 0))
        .expect_err("payload over the configured limit should fail");
    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert_eq!(b" world", payload.as_slice());

    let mut input = ChunkedAsyncInput::new(vec![0xff]);
    let error =
        complete(input.read_utf8_payload_into_async(&mut payload, 1, 1)).expect_err("invalid UTF-8 should fail");
    assert_eq!(ErrorKind::InvalidData, error.kind());
    assert_eq!([0xff], payload.as_slice());
}

#[test]
fn test_async_string_read_covers_every_length_prefix() {
    let mut input = ChunkedAsyncInput::new(string_fixture());

    assert_eq!(
        "payload",
        complete(input.read_utf8_payload_async("payload".len(), 32)).unwrap(),
    );
    assert_eq!("uleb", complete(input.read_utf8_string_uleb_usize_async(32)).unwrap(),);
    assert_eq!(
        "uleb-strict",
        complete(input.read_utf8_string_uleb_usize_strict_async(32)).unwrap(),
    );
    assert_eq!("uleb-u64", complete(input.read_utf8_string_uleb_u64_async(32)).unwrap(),);
    assert_eq!(
        "uleb-u64-strict",
        complete(input.read_utf8_string_uleb_u64_strict_async(32)).unwrap(),
    );
    assert_eq!(
        "u16-be",
        complete(input.read_string_with_u16_len_async(ByteOrder::BigEndian, 32,)).unwrap(),
    );
    assert_eq!(
        "fixed-u16-be",
        complete(input.read_string_with_u16_len_be_async(32)).unwrap(),
    );
    assert_eq!(
        "fixed-u16-le",
        complete(input.read_string_with_u16_len_le_async(32)).unwrap(),
    );
    assert_eq!(
        "u32-le",
        complete(input.read_string_with_u32_len_async(ByteOrder::LittleEndian, 32,)).unwrap(),
    );
    assert_eq!(
        "fixed-u32-be",
        complete(input.read_string_with_u32_len_be_async(32)).unwrap(),
    );
    assert_eq!(
        "fixed-u32-le",
        complete(input.read_string_with_u32_len_le_async(32)).unwrap(),
    );
}

#[test]
fn test_dropping_string_read_future_retains_consumed_input() {
    let mut input = ChunkedAsyncInput::starts_ready(b"payload".to_vec());

    assert!(matches!(poll_once(input.read_utf8_payload_async(7, 7)), Poll::Pending,));

    assert_eq!(2, input.position());
}

#[test]
fn test_async_string_read_reports_prefix_payload_and_utf8_errors() {
    let mut input = ChunkedAsyncInput::new(Vec::new());
    let prefix_error = complete(input.read_string_with_u32_len_be_async(32)).expect_err("missing prefix should fail");
    assert_eq!(ErrorKind::UnexpectedEof, prefix_error.kind());

    let mut input = ChunkedAsyncInput::new(Vec::new());
    let length_error = complete(input.read_utf8_payload_async(2, 1)).expect_err("oversized payload should fail");
    assert_eq!(ErrorKind::InvalidData, length_error.kind());

    let mut input = ChunkedAsyncInput::new(vec![0xFF]);
    let utf8_error = complete(input.read_utf8_payload_async(1, 1)).expect_err("invalid UTF-8 should fail");
    assert_eq!(ErrorKind::InvalidData, utf8_error.kind());

    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_eq!(
        ErrorKind::UnexpectedEof,
        complete(input.read_utf8_string_uleb_usize_async(8))
            .expect_err("missing usize LEB128 prefix should fail")
            .kind(),
    );
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_eq!(
        ErrorKind::UnexpectedEof,
        complete(input.read_utf8_string_uleb_usize_strict_async(8))
            .expect_err("missing strict usize LEB128 prefix should fail")
            .kind(),
    );
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_eq!(
        ErrorKind::UnexpectedEof,
        complete(input.read_utf8_string_uleb_u64_async(8))
            .expect_err("missing u64 LEB128 prefix should fail")
            .kind(),
    );
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_eq!(
        ErrorKind::UnexpectedEof,
        complete(input.read_utf8_string_uleb_u64_strict_async(8))
            .expect_err("missing strict u64 LEB128 prefix should fail")
            .kind(),
    );
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_eq!(
        ErrorKind::UnexpectedEof,
        complete(input.read_string_with_u16_len_async(ByteOrder::BigEndian, 8))
            .expect_err("missing runtime u16 prefix should fail")
            .kind(),
    );
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_eq!(
        ErrorKind::UnexpectedEof,
        complete(input.read_string_with_u16_len_be_async(8))
            .expect_err("missing big-endian u16 prefix should fail")
            .kind(),
    );
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_eq!(
        ErrorKind::UnexpectedEof,
        complete(input.read_string_with_u16_len_le_async(8))
            .expect_err("missing little-endian u16 prefix should fail")
            .kind(),
    );
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_eq!(
        ErrorKind::UnexpectedEof,
        complete(input.read_string_with_u32_len_async(ByteOrder::BigEndian, 8))
            .expect_err("missing runtime u32 prefix should fail")
            .kind(),
    );
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_eq!(
        ErrorKind::UnexpectedEof,
        complete(input.read_string_with_u32_len_be_async(8))
            .expect_err("missing big-endian u32 prefix should fail")
            .kind(),
    );
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_eq!(
        ErrorKind::UnexpectedEof,
        complete(input.read_string_with_u32_len_le_async(8))
            .expect_err("missing little-endian u32 prefix should fail")
            .kind(),
    );

    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_eq!(
        ErrorKind::UnexpectedEof,
        complete(input.read_utf8_payload_async(1, 1))
            .expect_err("missing payload should fail")
            .kind(),
    );
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_eq!(
        ErrorKind::OutOfMemory,
        complete(input.read_utf8_payload_async(usize::MAX, usize::MAX))
            .expect_err("impossible payload allocation should fail")
            .kind(),
    );
}

#[test]
fn test_async_payload_eof_then_independent_input() {
    let mut input = ChunkedAsyncInput::new(b"ab".to_vec());
    let mut payload = Vec::with_capacity(16);
    payload.extend_from_slice(b"old");
    let error =
        complete(input.read_utf8_payload_into_async(&mut payload, 5, 5)).expect_err("partial payload must report EOF");
    assert_eq!(ErrorKind::UnexpectedEof, error.kind());
    assert_eq!(2, input.position());
    assert!(payload.starts_with(b"ab"), "received prefix must be observable");

    let mut next_frame = ChunkedAsyncInput::new(b"new".to_vec());
    complete(next_frame.read_utf8_payload_into_async(&mut payload, 3, 5))
        .expect("storage must be reusable on an independent frame");
    assert_eq!(b"new", payload.as_slice());
    assert_eq!(3, next_frame.position());
}

#[test]
fn test_async_payload_cancel_then_independent_input() {
    let mut input = ChunkedAsyncInput::starts_ready(b"abcde".to_vec());
    let mut payload = Vec::with_capacity(16);
    payload.extend_from_slice(b"old");
    assert!(matches!(
        poll_once(input.read_utf8_payload_into_async(&mut payload, 5, 5)),
        Poll::Pending
    ));
    assert_eq!(2, input.position());
    assert!(payload.starts_with(b"ab"), "cancelled read retains received bytes");

    let mut next_frame = ChunkedAsyncInput::new(b"new".to_vec());
    complete(next_frame.read_utf8_payload_into_async(&mut payload, 3, 5))
        .expect("reuse must replace cancelled payload contents");
    assert_eq!(b"new", payload.as_slice());
    assert_eq!(3, next_frame.position());
}

#[test]
fn test_async_payload_length_policy_and_positions() {
    macro_rules! check_policy {
        ($permissive:ident, $strict:ident) => {{
            let mut input = ChunkedAsyncInput::new(vec![0x81, 0x00, b'a', b'!']);
            assert_eq!("a", complete(input.$permissive(1)).expect("permissive prefix"));
            assert_eq!(3, input.position());

            let mut input = ChunkedAsyncInput::new(vec![0x81, 0x00, b'a', b'!']);
            let error = complete(input.$strict(1)).expect_err("overlong prefix");
            assert_eq!(ErrorKind::InvalidData, error.kind());
            assert_eq!(2, input.position(), "strict failure must leave payload unread");

            let mut input = ChunkedAsyncInput::new(vec![1, b'a', b'!']);
            assert_eq!("a", complete(input.$permissive(1)).expect("canonical prefix"));
            assert_eq!(2, input.position());
            let mut input = ChunkedAsyncInput::new(vec![1, b'a', b'!']);
            assert_eq!("a", complete(input.$strict(1)).expect("canonical strict prefix"));
            assert_eq!(2, input.position());

            let mut input = ChunkedAsyncInput::new(vec![2, b'a', b'b', b'!']);
            let error = complete(input.$permissive(1)).expect_err("length limit");
            assert_eq!(ErrorKind::InvalidData, error.kind());
            assert_eq!(1, input.position(), "excessive payload must remain unread");
            let mut input = ChunkedAsyncInput::new(vec![2, b'a', b'b', b'!']);
            let error = complete(input.$strict(1)).expect_err("strict length limit");
            assert_eq!(ErrorKind::InvalidData, error.kind());
            assert_eq!(1, input.position(), "excessive payload must remain unread");
        }};
    }
    check_policy!(
        read_utf8_string_uleb_usize_async,
        read_utf8_string_uleb_usize_strict_async
    );
    check_policy!(read_utf8_string_uleb_u64_async, read_utf8_string_uleb_u64_strict_async);
}
