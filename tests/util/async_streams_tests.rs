// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::future::Future;
use std::io::Result;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

use qubit_io::AsyncInput;
use qubit_io::AsyncOutput;
use qubit_io_binary::AsyncBinaryReadExt;
use qubit_io_binary::AsyncBinaryWriteExt;
use qubit_io_binary::AsyncLeb128ReadExt;
use qubit_io_binary::AsyncLeb128WriteExt;
use qubit_io_binary::AsyncStringReadExt;
use qubit_io_binary::AsyncStringWriteExt;

struct PartialAsyncInput {
    bytes: Vec<u8>,
    position: usize,
    pending: bool,
}

impl PartialAsyncInput {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            position: 0,
            pending: true,
        }
    }
}

impl AsyncInput for PartialAsyncInput {
    type Item = u8;

    unsafe fn poll_read_unchecked(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        output: &mut [u8],
        index: usize,
        count: usize,
    ) -> Poll<Result<usize>> {
        if self.pending {
            self.pending = false;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        self.pending = true;
        let remaining = self.bytes.len().saturating_sub(self.position);
        let read = remaining.min(count).min(2);
        if read == 0 {
            return Poll::Ready(Ok(0));
        }
        output[index..index + read]
            .copy_from_slice(&self.bytes[self.position..self.position + read]);
        self.position += read;
        Poll::Ready(Ok(read))
    }
}

struct PartialAsyncOutput {
    bytes: Vec<u8>,
    pending: bool,
}

impl PartialAsyncOutput {
    const fn new() -> Self {
        Self {
            bytes: Vec::new(),
            pending: true,
        }
    }
}

impl AsyncOutput for PartialAsyncOutput {
    type Item = u8;

    unsafe fn poll_write_unchecked(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        input: &[u8],
        index: usize,
        count: usize,
    ) -> Poll<Result<usize>> {
        if self.pending {
            self.pending = false;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        self.pending = true;
        let written = count.min(2);
        self.bytes.extend_from_slice(&input[index..index + written]);
        Poll::Ready(Ok(written))
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }
}

fn complete<F>(future: F) -> F::Output
where
    F: Future,
{
    let mut context = Context::from_waker(Waker::noop());
    let mut future = std::pin::pin!(future);
    for _ in 0..256 {
        if let Poll::Ready(output) = future.as_mut().poll(&mut context) {
            return output;
        }
    }
    panic!("test future did not complete");
}

#[test]
fn async_stream_drivers_handle_pending_and_partial_io() {
    let mut output = PartialAsyncOutput::new();
    complete(output.write_u32_be_async(0x1234_5678))
        .expect("fixed-width payload should write");
    complete(output.write_uleb_u64_async(300))
        .expect("LEB128 payload should write");
    complete(output.write_utf8_payload_async("payload"))
        .expect("UTF-8 payload should write");

    let mut input = PartialAsyncInput::new(output.bytes);
    assert_eq!(
        0x1234_5678,
        complete(input.read_u32_be_async())
            .expect("fixed-width payload should read"),
    );
    assert_eq!(
        300,
        complete(input.read_uleb_u64_strict_async())
            .expect("LEB128 payload should read"),
    );
    assert_eq!(
        "payload",
        complete(input.read_utf8_payload_async(7, 7))
            .expect("UTF-8 payload should read"),
    );
}
