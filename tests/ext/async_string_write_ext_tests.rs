// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::{
    ErrorKind,
    Result,
};
use std::task::Poll;

use qubit_codec::ByteOrder;
use qubit_io::AsyncOutput;
use qubit_io_binary::{
    AsyncStringWriteExt,
    StringWriteExt,
};

use super::internal::async_io_test_support_tests::{
    ChunkedAsyncOutput,
    assert_send,
    complete,
    poll_once,
};

#[allow(dead_code)]
fn assert_string_write_future_is_send<T>(output: &mut T)
where
    T: AsyncOutput<Item = u8> + Send + Unpin + ?Sized,
{
    assert_send(output.write_utf8_payload_async(""));
}

#[test]
fn async_string_write_covers_every_length_prefix() {
    let mut output = ChunkedAsyncOutput::new();
    complete(async {
        output.write_utf8_payload_async("payload").await?;
        output.write_utf8_string_uleb_usize_async("uleb").await?;
        output.write_utf8_string_uleb_u64_async("uleb-u64").await?;
        output
            .write_string_with_u16_len_async("u16-be", ByteOrder::BigEndian)
            .await?;
        output
            .write_string_with_u16_len_be_async("fixed-u16-be")
            .await?;
        output
            .write_string_with_u16_len_le_async("fixed-u16-le")
            .await?;
        output
            .write_string_with_u32_len_async("u32-le", ByteOrder::LittleEndian)
            .await?;
        output
            .write_string_with_u32_len_be_async("fixed-u32-be")
            .await?;
        output
            .write_string_with_u32_len_le_async("fixed-u32-le")
            .await?;
        Result::<()>::Ok(())
    })
    .expect("all string forms should write");

    let mut expected = Vec::new();
    expected.write_utf8_payload("payload").unwrap();
    expected.write_utf8_string_uleb_usize("uleb").unwrap();
    expected.write_utf8_string_uleb_u64("uleb-u64").unwrap();
    expected
        .write_string_with_u16_len("u16-be", ByteOrder::BigEndian)
        .unwrap();
    expected
        .write_string_with_u16_len_be("fixed-u16-be")
        .unwrap();
    expected
        .write_string_with_u16_len_le("fixed-u16-le")
        .unwrap();
    expected
        .write_string_with_u32_len("u32-le", ByteOrder::LittleEndian)
        .unwrap();
    expected
        .write_string_with_u32_len_be("fixed-u32-be")
        .unwrap();
    expected
        .write_string_with_u32_len_le("fixed-u32-le")
        .unwrap();

    assert_eq!(expected, output.bytes());
}

#[test]
fn dropping_string_write_future_retains_partial_output() {
    let mut output = ChunkedAsyncOutput::starts_ready();

    assert!(matches!(
        poll_once(output.write_utf8_payload_async("payload")),
        Poll::Pending,
    ));

    assert_eq!(b"pa", output.bytes().as_slice());
}

#[test]
fn async_string_write_reports_length_and_output_errors() {
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);
    let output_error = complete(output.write_utf8_string_uleb_usize_async("x"))
        .expect_err("scripted output should fail");
    assert_eq!(ErrorKind::BrokenPipe, output_error.kind());

    let oversized = "x".repeat(usize::from(u16::MAX) + 1);
    let mut output = ChunkedAsyncOutput::new();
    let length_error = complete(
        output
            .write_string_with_u16_len_async(&oversized, ByteOrder::BigEndian),
    )
    .expect_err("oversized u16 string should fail");
    assert_eq!(ErrorKind::InvalidInput, length_error.kind());

    let mut output = ChunkedAsyncOutput::new();
    assert_eq!(
        ErrorKind::InvalidInput,
        complete(output.write_string_with_u16_len_be_async(&oversized))
            .expect_err("oversized big-endian u16 string should fail")
            .kind(),
    );
    let mut output = ChunkedAsyncOutput::new();
    assert_eq!(
        ErrorKind::InvalidInput,
        complete(output.write_string_with_u16_len_le_async(&oversized))
            .expect_err("oversized little-endian u16 string should fail")
            .kind(),
    );

    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);
    assert_eq!(
        ErrorKind::BrokenPipe,
        complete(output.write_utf8_string_uleb_u64_async("x"))
            .expect_err("u64 prefix write error should be returned")
            .kind(),
    );
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);
    assert_eq!(
        ErrorKind::BrokenPipe,
        complete(
            output.write_string_with_u16_len_async("x", ByteOrder::BigEndian)
        )
        .expect_err("runtime u16 prefix write error should be returned")
        .kind(),
    );
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);
    assert_eq!(
        ErrorKind::BrokenPipe,
        complete(output.write_string_with_u16_len_be_async("x"))
            .expect_err("big-endian u16 prefix write error should be returned")
            .kind(),
    );
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);
    assert_eq!(
        ErrorKind::BrokenPipe,
        complete(output.write_string_with_u16_len_le_async("x"))
            .expect_err(
                "little-endian u16 prefix write error should be returned"
            )
            .kind(),
    );
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);
    assert_eq!(
        ErrorKind::BrokenPipe,
        complete(
            output.write_string_with_u32_len_async("x", ByteOrder::BigEndian)
        )
        .expect_err("runtime u32 prefix write error should be returned")
        .kind(),
    );
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);
    assert_eq!(
        ErrorKind::BrokenPipe,
        complete(output.write_string_with_u32_len_be_async("x"))
            .expect_err("big-endian u32 prefix write error should be returned")
            .kind(),
    );
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);
    assert_eq!(
        ErrorKind::BrokenPipe,
        complete(output.write_string_with_u32_len_le_async("x"))
            .expect_err(
                "little-endian u32 prefix write error should be returned"
            )
            .kind(),
    );
}
