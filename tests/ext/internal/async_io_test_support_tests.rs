// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::future::Future;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

use qubit_io::AsyncInput;
use qubit_io::AsyncOutput;

pub(crate) struct ChunkedAsyncInput {
    bytes: Vec<u8>,
    position: usize,
    pending: bool,
}

impl ChunkedAsyncInput {
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            position: 0,
            pending: true,
        }
    }

    pub(crate) fn starts_ready(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            position: 0,
            pending: false,
        }
    }

    pub(crate) const fn position(&self) -> usize {
        self.position
    }
}

impl AsyncInput for ChunkedAsyncInput {
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

pub(crate) struct ChunkedAsyncOutput {
    bytes: Arc<Mutex<Vec<u8>>>,
    pending: bool,
    error: Option<ErrorKind>,
}

impl ChunkedAsyncOutput {
    pub(crate) fn new() -> Self {
        Self {
            bytes: Arc::new(Mutex::new(Vec::new())),
            pending: true,
            error: None,
        }
    }

    pub(crate) fn starts_ready() -> Self {
        Self {
            bytes: Arc::new(Mutex::new(Vec::new())),
            pending: false,
            error: None,
        }
    }

    pub(crate) fn failing(error: ErrorKind) -> Self {
        Self {
            bytes: Arc::new(Mutex::new(Vec::new())),
            pending: false,
            error: Some(error),
        }
    }

    pub(crate) fn bytes(&self) -> Vec<u8> {
        self.bytes.lock().expect("lock should succeed").clone()
    }
}

impl AsyncOutput for ChunkedAsyncOutput {
    type Item = u8;

    unsafe fn poll_write_unchecked(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        input: &[u8],
        index: usize,
        count: usize,
    ) -> Poll<Result<usize>> {
        if let Some(kind) = self.error {
            return Poll::Ready(Err(Error::new(
                kind,
                "scripted write failure",
            )));
        }
        if self.pending {
            self.pending = false;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        self.pending = true;
        let written = count.min(2);
        self.bytes
            .lock()
            .expect("lock should succeed")
            .extend_from_slice(&input[index..index + written]);
        Poll::Ready(Ok(written))
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }
}

pub(crate) fn complete<F>(future: F) -> F::Output
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

pub(crate) fn poll_once<F>(future: F) -> Poll<F::Output>
where
    F: Future,
{
    let mut context = Context::from_waker(Waker::noop());
    let mut future = std::pin::pin!(future);
    future.as_mut().poll(&mut context)
}
