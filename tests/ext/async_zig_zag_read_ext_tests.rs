// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::ErrorKind;
use std::task::Poll;

use qubit_io::AsyncInput;
use qubit_io_binary::{
    AsyncZigZagReadExt,
    ZigZagWriteExt,
};

use super::internal::async_io_test_support_tests::{
    ChunkedAsyncInput,
    assert_send,
    complete,
    poll_once,
};

#[allow(dead_code)]
fn assert_zig_zag_read_future_is_send<T>(input: &mut T)
where
    T: AsyncInput<Item = u8> + Send + Unpin + ?Sized,
{
    assert_send(input.read_zig_zag_i64_async());
}

fn zig_zag_fixture() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.write_zig_zag_i8(-100).unwrap();
    bytes.write_zig_zag_i16(-20_000).unwrap();
    bytes.write_zig_zag_i32(-2_000_000).unwrap();
    bytes.write_zig_zag_i64(-20_000_000_000).unwrap();
    bytes.write_zig_zag_i128(i128::MIN).unwrap();
    bytes.write_zig_zag_isize(isize::MIN).unwrap();
    bytes
}

#[test]
fn async_zig_zag_read_covers_supported_integer_widths() {
    let mut input = ChunkedAsyncInput::new(zig_zag_fixture());

    assert_eq!(
        -100,
        complete(input.read_zig_zag_i8_strict_async()).unwrap(),
    );
    assert_eq!(-20_000, complete(input.read_zig_zag_i16_async()).unwrap(),);
    assert_eq!(
        -2_000_000,
        complete(input.read_zig_zag_i32_strict_async()).unwrap(),
    );
    assert_eq!(
        -20_000_000_000,
        complete(input.read_zig_zag_i64_async()).unwrap(),
    );
    assert_eq!(
        i128::MIN,
        complete(input.read_zig_zag_i128_strict_async()).unwrap(),
    );
    assert_eq!(
        isize::MIN,
        complete(input.read_zig_zag_isize_async()).unwrap(),
    );
}

#[test]
fn dropping_zig_zag_read_future_retains_consumed_input() {
    let mut bytes = Vec::new();
    bytes.write_zig_zag_i128(i128::MIN).unwrap();
    let mut input = ChunkedAsyncInput::starts_ready(bytes);

    assert!(matches!(
        poll_once(input.read_zig_zag_i128_async()),
        Poll::Pending,
    ));

    assert_eq!(1, input.position());
}

#[test]
fn async_zig_zag_read_reports_invalid_and_truncated_payloads() {
    let mut input = ChunkedAsyncInput::new(vec![0x80; 19]);
    let invalid = complete(input.read_zig_zag_i128_async())
        .expect_err("unterminated maximum payload should fail");
    assert_eq!(ErrorKind::InvalidData, invalid.kind());

    let mut input = ChunkedAsyncInput::new(vec![0x80]);
    let truncated = complete(input.read_zig_zag_i128_async())
        .expect_err("truncated payload should fail");
    assert_eq!(ErrorKind::UnexpectedEof, truncated.kind());
}
