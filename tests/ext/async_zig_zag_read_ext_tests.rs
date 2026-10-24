// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::ErrorKind;
use std::task::Poll;

use qubit_io_binary::AsyncZigZagReadExt;
use qubit_io_binary::ZigZagWriteExt;

use super::internal::async_io_test_support_tests::ChunkedAsyncInput;
use super::internal::async_io_test_support_tests::complete;
use super::internal::async_io_test_support_tests::poll_once;

fn zig_zag_fixture() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.write_zig_zag_i8(-100).expect("i8 should encode");
    bytes.write_zig_zag_i16(-20_000).expect("i16 should encode");
    bytes.write_zig_zag_i32(-2_000_000).expect("i32 should encode");
    bytes.write_zig_zag_i64(-20_000_000_000).expect("i64 should encode");
    bytes.write_zig_zag_i128(i128::MIN).expect("i128 should encode");
    bytes.write_zig_zag_isize(isize::MIN).expect("isize should encode");
    bytes
}

#[test]
fn test_async_zig_zag_read_covers_supported_integer_widths() {
    let mut input = ChunkedAsyncInput::new(zig_zag_fixture());

    assert_eq!(
        -100,
        complete(input.read_zig_zag_i8_strict_async()).expect("i8 should decode"),
    );
    assert_eq!(
        -20_000,
        complete(input.read_zig_zag_i16_non_strict_async()).expect("i16 should decode"),
    );
    assert_eq!(
        -2_000_000,
        complete(input.read_zig_zag_i32_strict_async()).expect("i32 should decode"),
    );
    assert_eq!(
        -20_000_000_000,
        complete(input.read_zig_zag_i64_non_strict_async()).expect("i64 should decode"),
    );
    assert_eq!(
        i128::MIN,
        complete(input.read_zig_zag_i128_strict_async()).expect("i128 should decode"),
    );
    assert_eq!(
        isize::MIN,
        complete(input.read_zig_zag_isize_non_strict_async()).expect("isize should decode"),
    );
}

#[test]
fn test_async_zig_zag_noncanonical_zero_distinguishes_policies() {
    macro_rules! check_policy {
        ($strict:ident, $non_strict:ident) => {{
            let mut strict = ChunkedAsyncInput::new(vec![0x80, 0x00, 0x55]);
            let error = complete(strict.$strict()).expect_err("strict must reject noncanonical zero");
            assert_eq!(ErrorKind::InvalidData, error.kind(), stringify!($strict));
            assert_eq!(2, strict.position(), "strict must not consume the sentinel");
            let mut non_strict = ChunkedAsyncInput::new(vec![0x80, 0x00, 0x55]);
            assert_eq!(
                0,
                complete(non_strict.$non_strict()).expect("non-strict must accept zero")
            );
            assert_eq!(2, non_strict.position(), "non-strict must not consume the sentinel");
        }};
    }
    check_policy!(read_zig_zag_i8_strict_async, read_zig_zag_i8_non_strict_async);
    check_policy!(read_zig_zag_i16_strict_async, read_zig_zag_i16_non_strict_async);
    check_policy!(read_zig_zag_i32_strict_async, read_zig_zag_i32_non_strict_async);
    check_policy!(read_zig_zag_i64_strict_async, read_zig_zag_i64_non_strict_async);
    check_policy!(read_zig_zag_i128_strict_async, read_zig_zag_i128_non_strict_async);
    check_policy!(read_zig_zag_isize_strict_async, read_zig_zag_isize_non_strict_async);
}

#[test]
fn test_dropping_zig_zag_read_future_retains_consumed_input() {
    let mut bytes = Vec::new();
    bytes.write_zig_zag_i128(i128::MIN).expect("i128 should encode");
    let mut input = ChunkedAsyncInput::starts_ready(bytes);

    assert!(matches!(
        poll_once(input.read_zig_zag_i128_non_strict_async()),
        Poll::Pending,
    ));

    assert_eq!(1, input.position(), "polling should consume one byte");
}

#[test]
fn test_async_zig_zag_read_reports_invalid_and_truncated_payloads() {
    let mut input = ChunkedAsyncInput::new(vec![0x80; 19]);
    let invalid =
        complete(input.read_zig_zag_i128_non_strict_async()).expect_err("unterminated maximum payload should fail");
    assert_eq!(ErrorKind::InvalidData, invalid.kind());

    let mut input = ChunkedAsyncInput::new(vec![0x80]);
    let truncated = complete(input.read_zig_zag_i128_non_strict_async()).expect_err("truncated payload should fail");
    assert_eq!(ErrorKind::UnexpectedEof, truncated.kind());
}

#[test]
fn test_async_zig_zag_read_golden_vector() {
    macro_rules! check_read {
        ($strict:ident, $loose:ident, $value:expr, $bytes:expr) => {{
            let bytes: &[u8] = &$bytes;
            let mut strict = ChunkedAsyncInput::new(bytes.to_vec());
            assert_eq!(
                $value,
                complete(strict.$strict()).expect("canonical vector must decode")
            );
            assert_eq!(bytes.len(), strict.position());
            let mut loose = ChunkedAsyncInput::new(bytes.to_vec());
            assert_eq!($value, complete(loose.$loose()).expect("canonical vector must decode"));
            assert_eq!(bytes.len(), loose.position());
        }};
    }
    check_read!(read_zig_zag_i8_strict_async, read_zig_zag_i8_non_strict_async, -1, [1]);
    check_read!(
        read_zig_zag_i16_strict_async,
        read_zig_zag_i16_non_strict_async,
        -1,
        [1]
    );
    check_read!(
        read_zig_zag_i32_strict_async,
        read_zig_zag_i32_non_strict_async,
        -1,
        [1]
    );
    check_read!(
        read_zig_zag_i64_strict_async,
        read_zig_zag_i64_non_strict_async,
        -1,
        [1]
    );
    check_read!(
        read_zig_zag_i128_strict_async,
        read_zig_zag_i128_non_strict_async,
        -1,
        [1]
    );
    check_read!(
        read_zig_zag_isize_strict_async,
        read_zig_zag_isize_non_strict_async,
        -1,
        [1]
    );
    check_read!(
        read_zig_zag_i16_strict_async,
        read_zig_zag_i16_non_strict_async,
        -300,
        [0xd7, 4]
    );
    check_read!(
        read_zig_zag_i32_strict_async,
        read_zig_zag_i32_non_strict_async,
        -300,
        [0xd7, 4]
    );
    check_read!(
        read_zig_zag_i64_strict_async,
        read_zig_zag_i64_non_strict_async,
        -300,
        [0xd7, 4]
    );
    check_read!(
        read_zig_zag_i128_strict_async,
        read_zig_zag_i128_non_strict_async,
        -300,
        [0xd7, 4]
    );
    check_read!(
        read_zig_zag_isize_strict_async,
        read_zig_zag_isize_non_strict_async,
        -300,
        [0xd7, 4]
    );
}
