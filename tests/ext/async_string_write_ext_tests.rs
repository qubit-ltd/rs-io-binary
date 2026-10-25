// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use qubit_codec::ByteOrder;
use qubit_io::AsyncOutput;
use qubit_io_binary::AsyncStringWriteExt;
use qubit_io_binary::StringWriteExt;

use super::internal::async_io_test_support_tests::ChunkedAsyncOutput;
use super::internal::async_io_test_support_tests::complete;
use super::internal::async_io_test_support_tests::poll_once;

#[derive(Clone, Copy)]
enum StopMode {
    Error,
    Zero,
    Pending,
}

/// Stops after a cumulative byte count to distinguish prefix and payload
/// writes.
struct BoundaryOutput {
    bytes: Vec<u8>,
    stop_after: usize,
    mode: StopMode,
}

impl AsyncOutput for BoundaryOutput {
    type Item = u8;

    unsafe fn poll_write_unchecked(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        input: &[u8],
        index: usize,
        count: usize,
    ) -> Poll<Result<usize>> {
        let this = self.get_mut();
        if count == 0 {
            return Poll::Ready(Ok(0));
        }
        if this.bytes.len() == this.stop_after {
            return match this.mode {
                StopMode::Error => Poll::Ready(Err(Error::from(ErrorKind::BrokenPipe))),
                StopMode::Zero => Poll::Ready(Ok(0)),
                StopMode::Pending => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            };
        }
        let accepted = count.min(3).min(this.stop_after - this.bytes.len());
        // SAFETY: The caller guarantees a valid requested range and `accepted <=
        // count`.
        this.bytes.extend_from_slice(&input[index..index + accepted]);
        Poll::Ready(Ok(accepted))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[test]
fn test_async_string_write_covers_every_length_prefix() {
    let mut output = ChunkedAsyncOutput::new();
    complete(async {
        output.write_utf8_payload_async("payload").await?;
        output.write_utf8_string_uleb_usize_async("uleb").await?;
        output.write_utf8_string_uleb_u64_async("uleb-u64").await?;
        output
            .write_string_with_u16_len_async("u16-be", ByteOrder::BigEndian)
            .await?;
        output.write_string_with_u16_len_be_async("fixed-u16-be").await?;
        output.write_string_with_u16_len_le_async("fixed-u16-le").await?;
        output
            .write_string_with_u32_len_async("u32-le", ByteOrder::LittleEndian)
            .await?;
        output.write_string_with_u32_len_be_async("fixed-u32-be").await?;
        output.write_string_with_u32_len_le_async("fixed-u32-le").await?;
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
    expected.write_string_with_u16_len_be("fixed-u16-be").unwrap();
    expected.write_string_with_u16_len_le("fixed-u16-le").unwrap();
    expected
        .write_string_with_u32_len("u32-le", ByteOrder::LittleEndian)
        .unwrap();
    expected.write_string_with_u32_len_be("fixed-u32-be").unwrap();
    expected.write_string_with_u32_len_le("fixed-u32-le").unwrap();

    assert_eq!(expected, output.bytes());
}

#[test]
fn test_dropping_string_write_future_retains_partial_output() {
    let mut output = ChunkedAsyncOutput::starts_ready();

    assert!(matches!(
        poll_once(output.write_utf8_payload_async("payload")),
        Poll::Pending,
    ));

    assert_eq!(b"pa", output.bytes().as_slice());
}

#[test]
fn test_async_string_write_reports_length_and_output_errors() {
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);
    let output_error =
        complete(output.write_utf8_string_uleb_usize_async("x")).expect_err("scripted output should fail");
    assert_eq!(ErrorKind::BrokenPipe, output_error.kind());

    let oversized = "x".repeat(usize::from(u16::MAX) + 1);
    let mut output = ChunkedAsyncOutput::new();
    let length_error = complete(output.write_string_with_u16_len_async(&oversized, ByteOrder::BigEndian))
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
        complete(output.write_string_with_u16_len_async("x", ByteOrder::BigEndian))
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
            .expect_err("little-endian u16 prefix write error should be returned")
            .kind(),
    );
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);
    assert_eq!(
        ErrorKind::BrokenPipe,
        complete(output.write_string_with_u32_len_async("x", ByteOrder::BigEndian))
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
            .expect_err("little-endian u32 prefix write error should be returned")
            .kind(),
    );
}

#[test]
fn test_async_prefixed_string_write_boundaries() {
    macro_rules! check {
        ($method:ident $(, $order:expr)?; $prefix:expr) => {{
            let payload = "é".repeat(65);
            let prefix: Vec<u8> = $prefix;
            let mut expected = prefix.clone();
            expected.extend_from_slice(payload.as_bytes());
            for stop_after in [1, prefix.len(), prefix.len() + 2] {
                for mode in [StopMode::Error, StopMode::Zero, StopMode::Pending] {
                    let mut output = BoundaryOutput {
                        bytes: Vec::new(),
                        stop_after,
                        mode,
                    };
                    match mode {
                        StopMode::Pending => {
                            assert!(matches!(
                                poll_once(output.$method(&payload $(, $order)?)),
                                Poll::Pending,
                            ), "{} at {}", stringify!($method), stop_after);
                        }
                        StopMode::Error | StopMode::Zero => {
                            let error = complete(output.$method(&payload $(, $order)?))
                                .expect_err("stop point must return an error");
                            let kind = match mode {
                                StopMode::Error => ErrorKind::BrokenPipe,
                                StopMode::Zero => ErrorKind::WriteZero,
                                StopMode::Pending => unreachable!(),
                            };
                            assert_eq!(kind, error.kind(), "{} at {}", stringify!($method), stop_after);
                        }
                    }
                    assert_eq!(
                        &expected[..stop_after], output.bytes.as_slice(),
                        "{} at {}", stringify!($method), stop_after,
                    );
                }
            }
        }};
    }

    check!(write_utf8_string_uleb_usize_async; vec![0x82, 0x01]);
    check!(write_utf8_string_uleb_u64_async; vec![0x82, 0x01]);
    check!(write_string_with_u16_len_async, ByteOrder::BigEndian; 130_u16.to_be_bytes().to_vec());
    check!(write_string_with_u16_len_be_async; 130_u16.to_be_bytes().to_vec());
    check!(write_string_with_u16_len_le_async; 130_u16.to_le_bytes().to_vec());
    check!(write_string_with_u32_len_async, ByteOrder::LittleEndian; 130_u32.to_le_bytes().to_vec());
    check!(write_string_with_u32_len_be_async; 130_u32.to_be_bytes().to_vec());
    check!(write_string_with_u32_len_le_async; 130_u32.to_le_bytes().to_vec());
}
