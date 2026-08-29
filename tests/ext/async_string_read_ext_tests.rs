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
fn async_string_read_reuses_payload_buffer() {
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
fn async_string_read_covers_every_length_prefix() {
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
fn dropping_string_read_future_retains_consumed_input() {
    let mut input = ChunkedAsyncInput::starts_ready(b"payload".to_vec());

    assert!(matches!(poll_once(input.read_utf8_payload_async(7, 7)), Poll::Pending,));

    assert_eq!(2, input.position());
}

#[test]
fn async_string_read_reports_prefix_payload_and_utf8_errors() {
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
