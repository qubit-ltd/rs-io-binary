// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::ErrorKind;
use std::task::Poll;

use qubit_io_binary::AsyncLeb128ReadExt;
use qubit_io_binary::Leb128WriteExt;

use super::internal::async_io_test_support_tests::ChunkedAsyncInput;
use super::internal::async_io_test_support_tests::complete;
use super::internal::async_io_test_support_tests::poll_once;

fn leb128_fixture() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.write_uleb_u8(200).unwrap();
    bytes.write_uleb_u16(30_000).unwrap();
    bytes.write_uleb_u32(3_000_000).unwrap();
    bytes.write_uleb_u64(30_000_000_000).unwrap();
    bytes.write_uleb_u128(u128::MAX).unwrap();
    bytes.write_uleb_usize(usize::MAX).unwrap();
    bytes.write_sleb_i8(-100).unwrap();
    bytes.write_sleb_i16(-20_000).unwrap();
    bytes.write_sleb_i32(-2_000_000).unwrap();
    bytes.write_sleb_i64(-20_000_000_000).unwrap();
    bytes.write_sleb_i128(i128::MIN).unwrap();
    bytes.write_sleb_isize(isize::MIN).unwrap();
    bytes
}

#[test]
fn async_leb128_read_covers_supported_integer_widths() {
    let mut input = ChunkedAsyncInput::new(leb128_fixture());

    assert_eq!(200, complete(input.read_uleb_u8_strict_async()).unwrap());
    assert_eq!(
        30_000,
        complete(input.read_uleb_u16_non_strict_async()).unwrap(),
    );
    assert_eq!(
        3_000_000,
        complete(input.read_uleb_u32_strict_async()).unwrap(),
    );
    assert_eq!(
        30_000_000_000,
        complete(input.read_uleb_u64_non_strict_async()).unwrap(),
    );
    assert_eq!(
        u128::MAX,
        complete(input.read_uleb_u128_strict_async()).unwrap(),
    );
    assert_eq!(
        usize::MAX,
        complete(input.read_uleb_usize_non_strict_async()).unwrap(),
    );
    assert_eq!(-100, complete(input.read_sleb_i8_strict_async()).unwrap(),);
    assert_eq!(
        -20_000,
        complete(input.read_sleb_i16_non_strict_async()).unwrap(),
    );
    assert_eq!(
        -2_000_000,
        complete(input.read_sleb_i32_strict_async()).unwrap(),
    );
    assert_eq!(
        -20_000_000_000,
        complete(input.read_sleb_i64_non_strict_async()).unwrap(),
    );
    assert_eq!(
        i128::MIN,
        complete(input.read_sleb_i128_strict_async()).unwrap(),
    );
    assert_eq!(
        isize::MIN,
        complete(input.read_sleb_isize_non_strict_async()).unwrap(),
    );
}

#[test]
fn dropping_leb128_read_future_retains_consumed_input() {
    let mut bytes = Vec::new();
    bytes.write_uleb_u128(u128::MAX).unwrap();
    let mut input = ChunkedAsyncInput::starts_ready(bytes);

    assert!(matches!(
        poll_once(input.read_uleb_u128_non_strict_async()),
        Poll::Pending,
    ));

    assert_eq!(1, input.position());
}

#[test]
fn async_leb128_read_reports_invalid_and_truncated_payloads() {
    let mut input = ChunkedAsyncInput::new(vec![0x80; 19]);
    let invalid = complete(input.read_uleb_u128_non_strict_async())
        .expect_err("unterminated maximum payload should fail");
    assert_eq!(ErrorKind::InvalidData, invalid.kind());

    let mut input = ChunkedAsyncInput::new(vec![0x80]);
    let truncated = complete(input.read_uleb_u128_non_strict_async())
        .expect_err("truncated payload should fail");
    assert_eq!(ErrorKind::UnexpectedEof, truncated.kind());
}
