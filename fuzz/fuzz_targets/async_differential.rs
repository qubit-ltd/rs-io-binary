// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![no_main]

use std::future::Future;
use std::io;
use std::io::Cursor;
use std::io::ErrorKind;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

use libfuzzer_sys::fuzz_target;
use qubit_io::AsyncInput;
use qubit_io::AsyncOutput;
use qubit_io_binary::AsyncBinaryReadExt;
use qubit_io_binary::AsyncBinaryWriteExt;
use qubit_io_binary::AsyncLeb128ReadExt;
use qubit_io_binary::AsyncLeb128WriteExt;
use qubit_io_binary::AsyncStringReadExt;
use qubit_io_binary::AsyncStringWriteExt;
use qubit_io_binary::AsyncZigZagReadExt;
use qubit_io_binary::AsyncZigZagWriteExt;
use qubit_io_binary::BinaryReadExt;
use qubit_io_binary::BinaryWriteExt;
use qubit_io_binary::Leb128ReadExt;
use qubit_io_binary::Leb128WriteExt;
use qubit_io_binary::StringReadExt;
use qubit_io_binary::StringWriteExt;
use qubit_io_binary::ZigZagReadExt;
use qubit_io_binary::ZigZagWriteExt;

/// Keeps each target invocation bounded before allocating test buffers.
const MAX_FUZZ_INPUT_LEN: usize = 4096;
const MAX_STRING_LEN: usize = MAX_FUZZ_INPUT_LEN * 4;
/// Bounds polling for this target's deterministic async input and output
/// streams. This budget is not a time guarantee for general-purpose futures.
const MAX_ASYNC_POLLS: usize = 128;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_FUZZ_INPUT_LEN)];
    let chunk_size = usize::from(data.first().copied().unwrap_or_default() % 8) + 1;
    let payload = data.get(1..).unwrap_or_default();

    fuzz_fixed_width(payload, chunk_size);
    fuzz_scripted_fixed_width(payload, chunk_size);
    fuzz_leb128(payload, chunk_size);
    fuzz_zig_zag(payload, chunk_size);
    fuzz_varint_widths(payload, chunk_size);
    fuzz_string(payload, chunk_size);
    fuzz_malformed_reads(payload, chunk_size);
});

/// Compares fixed-width asynchronous writes and reads with synchronous APIs.
fn fuzz_fixed_width(payload: &[u8], chunk_size: usize) {
    let mut value_bytes = [0_u8; 8];
    let count = payload.len().min(value_bytes.len());
    value_bytes[..count].copy_from_slice(&payload[..count]);
    let value = u64::from_le_bytes(value_bytes);

    let mut expected = Vec::new();
    expected.write_u64_le(value).expect("value should encode");

    let mut output = FuzzAsyncOutput::new(chunk_size);
    let write_result = run_to_completion(async {
        output.write_u64_le_async(value).await?;
        output.flush_async().await
    });
    assert_eq!(Ok(()), result_signature(&write_result));
    assert_eq!(expected, output.bytes);

    let mut sync_input = Cursor::new(expected.clone());
    let sync_result = sync_input.read_u64_le();
    let sync_position = sync_input.position() as usize;
    let mut async_input = FuzzAsyncInput::new(expected, chunk_size);
    let async_result = run_to_completion(async_input.read_u64_le_async());
    assert_eq!(result_signature(&sync_result), result_signature(&async_result));
    assert_eq!(sync_position, async_input.position);
}

/// Checks fixed-width async I/O against bounded scripted failures.
fn fuzz_scripted_fixed_width(payload: &[u8], chunk_size: usize) {
    let mut value_bytes = [0_u8; 8];
    let count = payload.len().min(value_bytes.len());
    value_bytes[..count].copy_from_slice(&payload[..count]);

    for (events, chunk_size) in [(&[0, 1, 0, 3][..], 1), (&[0, 2, 1, 4][..], 1)] {
        check_scripted_fixed_width(events, chunk_size, value_bytes);
    }
    check_scripted_fixed_width(&payload[..payload.len().min(16)], chunk_size, value_bytes);
}

fn check_scripted_fixed_width(events: &[u8], chunk_size: usize, value_bytes: [u8; 8]) {
    let chunk_size = chunk_size.clamp(1, 8);

    let (expected_read, read_count) = scripted_expected(events, chunk_size, false);
    let mut input = ScriptedAsyncInput::new(value_bytes.to_vec(), events, chunk_size);
    let read_result = run_scripted_to_completion(input.read_u64_le_async());
    assert_eq!(
        expected_read.map(|()| u64::from_le_bytes(value_bytes)),
        read_result.map_err(|error| error.kind()),
    );
    assert_eq!(read_count, input.position);

    let (expected_write, write_count) = scripted_expected(events, chunk_size, true);
    let mut output = ScriptedAsyncOutput::new(events, chunk_size);
    let write_result = run_scripted_to_completion(async {
        output.write_u64_le_async(u64::from_le_bytes(value_bytes)).await?;
        output.flush_async().await
    });
    assert_eq!(expected_write, write_result.map_err(|error| error.kind()));
    assert_eq!(&value_bytes[..write_count], output.bytes.as_slice());
}

fn scripted_expected(events: &[u8], chunk_size: usize, writing: bool) -> (Result<(), ErrorKind>, usize) {
    let mut transferred = 0;
    for event in events.iter().take(16).map(|byte| byte % 5) {
        if transferred == 8 {
            return (Ok(()), transferred);
        }
        match event {
            0 => transferred += chunk_size.min(8 - transferred),
            1 | 2 => {}
            3 => return (Err(ErrorKind::BrokenPipe), transferred),
            4 => {
                return (
                    Err(if writing {
                        ErrorKind::WriteZero
                    } else {
                        ErrorKind::UnexpectedEof
                    }),
                    transferred,
                );
            }
            _ => unreachable!(),
        }
    }
    (Ok(()), 8)
}

fn run_scripted_to_completion<F>(future: F) -> F::Output
where
    F: Future,
{
    let mut future = std::pin::pin!(future);
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    for _ in 0..128 {
        if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
            return value;
        }
    }
    panic!("scripted fixed-width future exceeded poll budget");
}

struct ScriptedAsyncInput<'a> {
    bytes: Vec<u8>,
    position: usize,
    events: &'a [u8],
    event_index: usize,
    chunk_size: usize,
}

impl<'a> ScriptedAsyncInput<'a> {
    fn new(bytes: Vec<u8>, events: &'a [u8], chunk_size: usize) -> Self {
        Self {
            bytes,
            position: 0,
            events,
            event_index: 0,
            chunk_size,
        }
    }

    fn next_event(&mut self) -> u8 {
        let event = self.events.get(self.event_index).map_or(0, |byte| byte % 5);
        self.event_index += usize::from(self.event_index < self.events.len());
        event
    }
}

impl AsyncInput for ScriptedAsyncInput<'_> {
    type Item = u8;

    /// Applies one script event to a non-empty read request.
    unsafe fn poll_read_unchecked(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        output: &mut [u8],
        index: usize,
        count: usize,
    ) -> Poll<io::Result<usize>> {
        // SAFETY: The AsyncInput caller supplies a valid output range `[index, index +
        // count)`. This implementation writes only within that range and
        // reports the number initialized.
        let this = self.get_mut();
        if count == 0 {
            return Poll::Ready(Ok(0));
        }
        loop {
            match this.next_event() {
                0 => {
                    let count = count.min(this.chunk_size).min(this.bytes.len() - this.position);
                    output[index..index + count].copy_from_slice(&this.bytes[this.position..this.position + count]);
                    this.position += count;
                    return Poll::Ready(Ok(count));
                }
                1 => {
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                2 => continue,
                3 => return Poll::Ready(Err(io::Error::from(ErrorKind::BrokenPipe))),
                4 => return Poll::Ready(Ok(0)),
                _ => unreachable!(),
            }
        }
    }
}

struct ScriptedAsyncOutput<'a> {
    bytes: Vec<u8>,
    events: &'a [u8],
    event_index: usize,
    chunk_size: usize,
}

impl<'a> ScriptedAsyncOutput<'a> {
    fn new(events: &'a [u8], chunk_size: usize) -> Self {
        Self {
            bytes: Vec::new(),
            events,
            event_index: 0,
            chunk_size,
        }
    }

    fn next_event(&mut self) -> u8 {
        let event = self.events.get(self.event_index).map_or(0, |byte| byte % 5);
        self.event_index += usize::from(self.event_index < self.events.len());
        event
    }
}

impl AsyncOutput for ScriptedAsyncOutput<'_> {
    type Item = u8;

    /// Applies one script event to a non-empty write request.
    unsafe fn poll_write_unchecked(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        input: &[u8],
        index: usize,
        count: usize,
    ) -> Poll<io::Result<usize>> {
        // SAFETY: The AsyncOutput caller supplies a valid input range `[index, index +
        // count)`. This implementation reads only within that range and reports
        // the number accepted.
        let this = self.get_mut();
        if count == 0 {
            return Poll::Ready(Ok(0));
        }
        loop {
            match this.next_event() {
                0 => {
                    let count = count.min(this.chunk_size);
                    this.bytes.extend_from_slice(&input[index..index + count]);
                    return Poll::Ready(Ok(count));
                }
                1 => {
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                2 => continue,
                3 => return Poll::Ready(Err(io::Error::from(ErrorKind::BrokenPipe))),
                4 => return Poll::Ready(Ok(0)),
                _ => unreachable!(),
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

/// Compares canonical LEB128 asynchronous writes and strict reads.
fn fuzz_leb128(payload: &[u8], chunk_size: usize) {
    let mut value_bytes = [0_u8; 8];
    let count = payload.len().min(value_bytes.len());
    value_bytes[..count].copy_from_slice(&payload[..count]);
    let value = u64::from_le_bytes(value_bytes);

    let mut expected = Vec::new();
    expected.write_uleb_u64(value).expect("value should encode");
    let mut output = FuzzAsyncOutput::new(chunk_size);
    let write_result = run_to_completion(async {
        output.write_uleb_u64_async(value).await?;
        output.flush_async().await
    });
    assert_eq!(Ok(()), result_signature(&write_result));
    assert_eq!(expected, output.bytes);

    let mut sync_input = Cursor::new(expected.clone());
    let sync_result = sync_input.read_uleb_u64_strict();
    let sync_position = sync_input.position() as usize;
    let mut async_input = FuzzAsyncInput::new(expected, chunk_size);
    let async_result = run_to_completion(async_input.read_uleb_u64_strict_async());
    assert_eq!(result_signature(&sync_result), result_signature(&async_result));
    assert_eq!(sync_position, async_input.position);
}

/// Compares ZigZag asynchronous writes and reads with their synchronous forms.
fn fuzz_zig_zag(payload: &[u8], chunk_size: usize) {
    let mut value_bytes = [0_u8; 8];
    let count = payload.len().min(value_bytes.len());
    value_bytes[..count].copy_from_slice(&payload[..count]);
    let value = i64::from_le_bytes(value_bytes);

    let mut expected = Vec::new();
    expected.write_zig_zag_i64(value).expect("value should encode");
    let mut output = FuzzAsyncOutput::new(chunk_size);
    let write_result = run_to_completion(async {
        output.write_zig_zag_i64_async(value).await?;
        output.flush_async().await
    });
    assert_eq!(Ok(()), result_signature(&write_result));
    assert_eq!(expected, output.bytes);

    let mut sync_input = Cursor::new(expected.clone());
    let sync_result = sync_input.read_zig_zag_i64_strict();
    let sync_position = sync_input.position() as usize;
    let mut async_input = FuzzAsyncInput::new(expected, chunk_size);
    let async_result = run_to_completion(async_input.read_zig_zag_i64_strict_async());
    assert_eq!(result_signature(&sync_result), result_signature(&async_result));
    assert_eq!(sync_position, async_input.position);
}

/// Compares variable-width integer APIs across their supported fixed widths.
fn fuzz_varint_widths(payload: &[u8], chunk_size: usize) {
    macro_rules! check_codec {
        ($value:expr, $write_sync:ident, $write_async:ident,
         $read_sync_strict:ident, $read_sync_non_strict:ident,
         $read_async_strict:ident, $read_async_non_strict:ident) => {{
            let mut sync_input = Cursor::new(payload);
            let sync_strict = sync_input.$read_sync_strict();
            let sync_strict_position = sync_input.position() as usize;
            let mut async_input = FuzzAsyncInput::new(payload.to_vec(), chunk_size);
            let async_strict = run_to_completion(async_input.$read_async_strict());
            assert_eq!(result_signature(&sync_strict), result_signature(&async_strict));
            assert_eq!(sync_strict_position, async_input.position);

            let mut sync_input = Cursor::new(payload);
            let sync_non_strict = sync_input.$read_sync_non_strict();
            let sync_non_strict_position = sync_input.position() as usize;
            let mut async_input = FuzzAsyncInput::new(payload.to_vec(), chunk_size);
            let async_non_strict = run_to_completion(async_input.$read_async_non_strict());
            assert_eq!(result_signature(&sync_non_strict), result_signature(&async_non_strict));
            assert_eq!(sync_non_strict_position, async_input.position);

            let value = $value;
            let mut expected = Vec::new();
            expected.$write_sync(value).expect("value should encode");
            let mut output = FuzzAsyncOutput::new(chunk_size);
            let write_result = run_to_completion(async {
                output.$write_async(value).await?;
                output.flush_async().await
            });
            assert_eq!(Ok(()), result_signature(&write_result));
            assert_eq!(expected, output.bytes);

            let mut strict_input = FuzzAsyncInput::new(expected.clone(), chunk_size);
            let strict_result = run_to_completion(strict_input.$read_async_strict());
            assert_eq!(Ok(value), result_signature(&strict_result));
            assert_eq!(expected.len(), strict_input.position);
            let mut non_strict_input = FuzzAsyncInput::new(expected.clone(), chunk_size);
            let non_strict_result = run_to_completion(non_strict_input.$read_async_non_strict());
            assert_eq!(Ok(value), result_signature(&non_strict_result));
            assert_eq!(expected.len(), non_strict_input.position);
        }};
    }

    let mut bytes = [0_u8; 16];
    let count = payload.len().min(bytes.len());
    bytes[..count].copy_from_slice(&payload[..count]);

    check_codec!(
        u8::from_le_bytes([bytes[0]]),
        write_uleb_u8,
        write_uleb_u8_async,
        read_uleb_u8_strict,
        read_uleb_u8_non_strict,
        read_uleb_u8_strict_async,
        read_uleb_u8_non_strict_async
    );
    check_codec!(
        u16::from_le_bytes([bytes[0], bytes[1]]),
        write_uleb_u16,
        write_uleb_u16_async,
        read_uleb_u16_strict,
        read_uleb_u16_non_strict,
        read_uleb_u16_strict_async,
        read_uleb_u16_non_strict_async
    );
    check_codec!(
        u128::from_le_bytes(bytes),
        write_uleb_u128,
        write_uleb_u128_async,
        read_uleb_u128_strict,
        read_uleb_u128_non_strict,
        read_uleb_u128_strict_async,
        read_uleb_u128_non_strict_async
    );
    check_codec!(
        i8::from_le_bytes([bytes[0]]),
        write_sleb_i8,
        write_sleb_i8_async,
        read_sleb_i8_strict,
        read_sleb_i8_non_strict,
        read_sleb_i8_strict_async,
        read_sleb_i8_non_strict_async
    );
    check_codec!(
        i16::from_le_bytes([bytes[0], bytes[1]]),
        write_sleb_i16,
        write_sleb_i16_async,
        read_sleb_i16_strict,
        read_sleb_i16_non_strict,
        read_sleb_i16_strict_async,
        read_sleb_i16_non_strict_async
    );
    check_codec!(
        i64::from_le_bytes(bytes[..8].try_into().expect("slice has eight bytes")),
        write_sleb_i64,
        write_sleb_i64_async,
        read_sleb_i64_strict,
        read_sleb_i64_non_strict,
        read_sleb_i64_strict_async,
        read_sleb_i64_non_strict_async
    );
    check_codec!(
        i128::from_le_bytes(bytes),
        write_sleb_i128,
        write_sleb_i128_async,
        read_sleb_i128_strict,
        read_sleb_i128_non_strict,
        read_sleb_i128_strict_async,
        read_sleb_i128_non_strict_async
    );
    check_codec!(
        i8::from_le_bytes([bytes[0]]),
        write_zig_zag_i8,
        write_zig_zag_i8_async,
        read_zig_zag_i8_strict,
        read_zig_zag_i8_non_strict,
        read_zig_zag_i8_strict_async,
        read_zig_zag_i8_non_strict_async
    );
    check_codec!(
        i16::from_le_bytes([bytes[0], bytes[1]]),
        write_zig_zag_i16,
        write_zig_zag_i16_async,
        read_zig_zag_i16_strict,
        read_zig_zag_i16_non_strict,
        read_zig_zag_i16_strict_async,
        read_zig_zag_i16_non_strict_async
    );
    check_codec!(
        i128::from_le_bytes(bytes),
        write_zig_zag_i128,
        write_zig_zag_i128_async,
        read_zig_zag_i128_strict,
        read_zig_zag_i128_non_strict,
        read_zig_zag_i128_strict_async,
        read_zig_zag_i128_non_strict_async
    );
}

/// Compares length-prefixed UTF-8 framing through both API layers.
fn fuzz_string(payload: &[u8], chunk_size: usize) {
    let value = String::from_utf8_lossy(payload).into_owned();
    let mut expected = Vec::new();
    expected
        .write_utf8_string_uleb_u64(&value)
        .expect("string should encode");

    let mut output = FuzzAsyncOutput::new(chunk_size);
    let write_result = run_to_completion(async {
        output.write_utf8_string_uleb_u64_async(&value).await?;
        output.flush_async().await
    });
    assert_eq!(Ok(()), result_signature(&write_result));
    assert_eq!(expected, output.bytes);

    let mut sync_input = Cursor::new(expected.clone());
    let sync_result = sync_input
        .read_utf8_string_uleb_u64_strict(MAX_STRING_LEN)
        .map(|text| text.into_bytes());
    let sync_position = sync_input.position() as usize;
    let mut async_input = FuzzAsyncInput::new(expected, chunk_size);
    let async_result = run_to_completion(async_input.read_utf8_string_uleb_u64_strict_async(MAX_STRING_LEN))
        .map(|text| text.into_bytes());
    assert_eq!(string_signature(&sync_result), string_signature(&async_result));
    assert_eq!(sync_position, async_input.position);
}

/// Compares error categories and consumed positions for arbitrary input.
fn fuzz_malformed_reads(payload: &[u8], chunk_size: usize) {
    let mut sync_input = Cursor::new(payload);
    let sync_result = sync_input.read_u64_le();
    let sync_position = sync_input.position() as usize;
    let mut async_input = FuzzAsyncInput::new(payload.to_vec(), chunk_size);
    let async_result = run_to_completion(async_input.read_u64_le_async());
    assert_eq!(result_signature(&sync_result), result_signature(&async_result));
    assert_eq!(sync_position, async_input.position);

    let mut sync_input = Cursor::new(payload);
    let sync_result = sync_input
        .read_utf8_string_uleb_u64_strict(MAX_STRING_LEN)
        .map(|text| text.into_bytes());
    let sync_position = sync_input.position() as usize;
    let mut async_input = FuzzAsyncInput::new(payload.to_vec(), chunk_size);
    let async_result = run_to_completion(async_input.read_utf8_string_uleb_u64_strict_async(MAX_STRING_LEN))
        .map(|text| text.into_bytes());
    assert_eq!(string_signature(&sync_result), string_signature(&async_result));
    assert_eq!(sync_position, async_input.position);
}

/// Drives a runtime-neutral future with a deterministic waker.
///
/// Panics if the future remains pending for the full poll budget.
fn run_to_completion<F>(future: F) -> F::Output
where
    F: Future,
{
    let mut future = std::pin::pin!(future);
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    for _ in 0..MAX_ASYNC_POLLS {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => {}
        }
    }
    panic!("async differential future exceeded poll budget of {MAX_ASYNC_POLLS}");
}

/// Converts a result into a value/error-kind signature for exact comparison.
fn result_signature<T>(result: &io::Result<T>) -> Result<T, ErrorKind>
where
    T: Copy,
{
    match result {
        Ok(value) => Ok(*value),
        Err(error) => Err(error.kind()),
    }
}

/// Converts string results into owned byte signatures for exact comparison.
fn string_signature(result: &io::Result<Vec<u8>>) -> Result<Vec<u8>, ErrorKind> {
    match result {
        Ok(value) => Ok(value.clone()),
        Err(error) => Err(error.kind()),
    }
}

/// Runtime-neutral input that produces bounded chunks and one pending poll.
struct FuzzAsyncInput {
    bytes: Vec<u8>,
    position: usize,
    chunk_size: usize,
    pending: bool,
}

impl FuzzAsyncInput {
    /// Creates an input that injects one pending poll before making progress.
    fn new(bytes: Vec<u8>, chunk_size: usize) -> Self {
        Self {
            bytes,
            position: 0,
            chunk_size,
            pending: true,
        }
    }
}

impl AsyncInput for FuzzAsyncInput {
    type Item = u8;

    /// Reads at most one configured chunk and wakes the caller after pending.
    unsafe fn poll_read_unchecked(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        output: &mut [u8],
        index: usize,
        count: usize,
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        if count == 0 {
            return Poll::Ready(Ok(0));
        }
        if this.pending {
            this.pending = false;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        if this.position == this.bytes.len() {
            return Poll::Ready(Ok(0));
        }
        let count = count.min(this.chunk_size).min(this.bytes.len() - this.position);
        output[index..index + count].copy_from_slice(&this.bytes[this.position..this.position + count]);
        this.position += count;
        Poll::Ready(Ok(count))
    }
}

/// Runtime-neutral output that accepts bounded chunks and one pending poll.
struct FuzzAsyncOutput {
    bytes: Vec<u8>,
    chunk_size: usize,
    pending: bool,
}

impl FuzzAsyncOutput {
    /// Creates an output that injects one pending poll before making progress.
    fn new(chunk_size: usize) -> Self {
        Self {
            bytes: Vec::new(),
            chunk_size,
            pending: true,
        }
    }
}

impl AsyncOutput for FuzzAsyncOutput {
    type Item = u8;

    /// Accepts at most one configured chunk and wakes the caller after pending.
    unsafe fn poll_write_unchecked(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        input: &[u8],
        index: usize,
        count: usize,
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        if count == 0 {
            return Poll::Ready(Ok(0));
        }
        if this.pending {
            this.pending = false;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        let count = count.min(this.chunk_size);
        this.bytes.extend_from_slice(&input[index..index + count]);
        Poll::Ready(Ok(count))
    }

    /// Completes flushing after the same pending behavior as writes.
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if this.pending {
            this.pending = false;
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(Ok(()))
        }
    }
}
