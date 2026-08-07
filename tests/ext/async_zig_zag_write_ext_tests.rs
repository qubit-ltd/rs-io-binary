// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::ErrorKind;
use std::io::Result;
use std::task::Poll;

use qubit_io_binary::AsyncZigZagWriteExt;
use qubit_io_binary::ZigZagWriteExt;

use super::internal::async_io_test_support_tests::ChunkedAsyncOutput;
use super::internal::async_io_test_support_tests::complete;
use super::internal::async_io_test_support_tests::poll_once;

#[test]
fn async_zig_zag_write_covers_supported_integer_widths() {
    let mut output = ChunkedAsyncOutput::new();
    complete(async {
        output.write_zig_zag_i8_async(-100).await?;
        output.write_zig_zag_i16_async(-20_000).await?;
        output.write_zig_zag_i32_async(-2_000_000).await?;
        output.write_zig_zag_i64_async(-20_000_000_000).await?;
        output.write_zig_zag_i128_async(i128::MIN).await?;
        output.write_zig_zag_isize_async(isize::MIN).await?;
        Result::<()>::Ok(())
    })
    .expect("all ZigZag values should write");

    let mut expected = Vec::new();
    expected.write_zig_zag_i8(-100).unwrap();
    expected.write_zig_zag_i16(-20_000).unwrap();
    expected.write_zig_zag_i32(-2_000_000).unwrap();
    expected.write_zig_zag_i64(-20_000_000_000).unwrap();
    expected.write_zig_zag_i128(i128::MIN).unwrap();
    expected.write_zig_zag_isize(isize::MIN).unwrap();

    assert_eq!(expected, output.bytes());
}

#[test]
fn dropping_zig_zag_write_future_retains_partial_output() {
    let mut output = ChunkedAsyncOutput::starts_ready();

    assert!(matches!(
        poll_once(output.write_zig_zag_i128_async(i128::MIN)),
        Poll::Pending,
    ));

    assert_eq!(2, output.bytes().len());
}

#[test]
fn async_zig_zag_write_propagates_output_errors() {
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);

    let error = complete(output.write_zig_zag_i64_async(-42))
        .expect_err("scripted output should fail");

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}
