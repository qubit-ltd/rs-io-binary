// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Checks rejection of portable u64 lengths that do not fit a 32-bit usize.
#![cfg(target_pointer_width = "32")]

use std::future::Future;
use std::io;
use std::io::Cursor;
use std::io::ErrorKind;
use std::io::Read;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

use qubit_codec_binary::NonStrict;
use qubit_codec_binary::Strict;
use qubit_io::AsyncInput;
use qubit_io_binary::AsyncStringReadExt;
use qubit_io_binary::BufferedLeb128Reader;
use qubit_io_binary::Leb128Reader;
use qubit_io_binary::StringReadExt;

/// Returns canonical prefixes for 2^32, 2^32 + 1, and u64::MAX.
fn overflow_prefixes() -> [Vec<u8>; 3] {
    [
        vec![0x80, 0x80, 0x80, 0x80, 0x10],
        vec![0x81, 0x80, 0x80, 0x80, 0x10],
        vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
    ]
}

fn frame(prefix: &[u8]) -> Vec<u8> {
    let mut bytes = prefix.to_vec();
    bytes.extend_from_slice(b"x!");
    bytes
}

fn assert_overflow(result: io::Result<String>) {
    assert_eq!(
        ErrorKind::InvalidData,
        result.expect_err("u64 length must not fit a 32-bit usize").kind()
    );
}

/// Provides immediately ready asynchronous reads without an executor.
struct ReadyInput(Cursor<Vec<u8>>);

impl AsyncInput for ReadyInput {
    type Item = u8;

    unsafe fn poll_read_unchecked(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        output: &mut [u8],
        index: usize,
        count: usize,
    ) -> Poll<io::Result<usize>> {
        // SAFETY: The caller provides a valid indexed destination range.
        Poll::Ready(Read::read(&mut self.get_mut().0, &mut output[index..index + count]))
    }
}

fn complete_ready<F: Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(result) => result,
        Poll::Pending => panic!("immediately ready input must finish in one poll"),
    }
}

#[test]
fn test_u64_length_overflow_sync_extensions() {
    for prefix in overflow_prefixes() {
        macro_rules! check {
            ($method:ident) => {{
                let mut input = Cursor::new(frame(&prefix));
                assert_overflow(input.$method(usize::MAX));
                assert_eq!(prefix.len() as u64, input.position());
                assert_eq!(b"x!", &input.get_ref()[input.position() as usize..]);
            }};
        }
        check!(read_utf8_string_uleb_u64);
        check!(read_utf8_string_uleb_u64_strict);
    }
}

#[test]
fn test_u64_length_overflow_async_extensions() {
    for prefix in overflow_prefixes() {
        macro_rules! check {
            ($method:ident) => {{
                let mut input = ReadyInput(Cursor::new(frame(&prefix)));
                assert_overflow(complete_ready(input.$method(usize::MAX)));
                assert_eq!(prefix.len() as u64, input.0.position());
                assert_eq!(b"x!", &input.0.get_ref()[input.0.position() as usize..]);
            }};
        }
        check!(read_utf8_string_uleb_u64_async);
        check!(read_utf8_string_uleb_u64_strict_async);
    }
}

#[test]
fn test_u64_length_overflow_typed_readers() {
    for prefix in overflow_prefixes() {
        macro_rules! check {
            ($policy:ty, $method:ident) => {{
                let mut reader = Leb128Reader::<_, $policy>::new(Cursor::new(frame(&prefix)));
                assert_overflow(reader.$method(usize::MAX));
                let input = reader.into_inner();
                assert_eq!(prefix.len() as u64, input.position());
                assert_eq!(b"x!", &input.get_ref()[input.position() as usize..]);
            }};
        }
        check!(Strict, read_utf8_string_u64);
        check!(NonStrict, read_utf8_string_u64_non_strict);
    }
}

#[test]
fn test_u64_length_overflow_buffered_readers() {
    for prefix in overflow_prefixes() {
        macro_rules! check {
            ($policy:ty, $method:ident) => {{
                let mut reader = BufferedLeb128Reader::<_, $policy>::new(Cursor::new(frame(&prefix)));
                assert_overflow(reader.$method(usize::MAX));
                let (input, unread) = reader.into_parts();
                assert_eq!(
                    prefix.len() as u64,
                    input.position() - unread.readable().len() as u64
                );
                let mut remainder = unread.readable().to_vec();
                remainder.extend_from_slice(&input.get_ref()[input.position() as usize..]);
                assert_eq!(b"x!", remainder.as_slice());
            }};
        }
        check!(Strict, read_utf8_string_u64);
        check!(NonStrict, read_utf8_string_u64_non_strict);
    }
}
