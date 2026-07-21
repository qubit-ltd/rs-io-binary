// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::future::Future;
use std::io::Result;
use std::io::{
    Error,
    ErrorKind,
};
use std::pin::Pin;
use std::sync::{
    Arc,
    Mutex,
};
use std::task::{
    Context,
    Poll,
    Waker,
};

use qubit_io::{
    AsyncInput,
    AsyncOutput,
};
use qubit_io_binary::{
    AsyncBinaryReadExt,
    AsyncBinaryWriteExt,
    AsyncLeb128ReadExt,
    AsyncLeb128WriteExt,
    AsyncStringReadExt,
    AsyncStringWriteExt,
    AsyncZigZagReadExt,
    AsyncZigZagWriteExt,
    ByteOrder,
};

struct ChunkedAsyncInput {
    bytes: Vec<u8>,
    position: usize,
    pending: bool,
}

impl ChunkedAsyncInput {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            position: 0,
            pending: true,
        }
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

struct ChunkedAsyncOutput {
    bytes: Arc<Mutex<Vec<u8>>>,
    pending: bool,
    error: Option<ErrorKind>,
}

impl ChunkedAsyncOutput {
    fn new(bytes: Arc<Mutex<Vec<u8>>>) -> Self {
        Self {
            bytes,
            pending: true,
            error: None,
        }
    }

    fn failing(error: ErrorKind) -> Self {
        Self {
            bytes: Arc::new(Mutex::new(Vec::new())),
            pending: false,
            error: Some(error),
        }
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
fn async_binary_extensions_round_trip_across_pending_and_partial_io() {
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let mut output = ChunkedAsyncOutput::new(bytes.clone());

    complete(output.write_u16_async(0x1234, ByteOrder::BigEndian))
        .expect("u16 should write");
    complete(output.write_i32_le_async(-123_456)).expect("i32 should write");
    complete(output.write_f64_async(12.5, ByteOrder::LittleEndian))
        .expect("f64 should write");
    complete(output.flush_async()).expect("output should flush");

    let encoded = bytes.lock().expect("lock should succeed").clone();
    let mut input = ChunkedAsyncInput::new(encoded);
    assert_eq!(
        0x1234,
        complete(input.read_u16_async(ByteOrder::BigEndian))
            .expect("u16 should read"),
    );
    assert_eq!(
        -123_456,
        complete(input.read_i32_le_async()).expect("i32 should read"),
    );
    assert_eq!(
        12.5,
        complete(input.read_f64_async(ByteOrder::LittleEndian))
            .expect("f64 should read"),
    );
}

#[test]
fn async_fixed_width_extensions_cover_every_scalar_and_order() {
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let mut output = ChunkedAsyncOutput::new(bytes.clone());

    complete(async {
        output.write_u8_async(0xA5).await?;
        output.write_i8_async(-0x25).await?;

        output.write_u16_be_async(0x1234).await?;
        output.write_u16_le_async(0x5678).await?;
        output.write_u16_async(0x9ABC, ByteOrder::BigEndian).await?;
        output
            .write_u16_async(0xDEF0, ByteOrder::LittleEndian)
            .await?;
        output
            .write_u16_async(0x1357, ByteOrder::NativeEndian)
            .await?;

        output.write_u32_be_async(0x1234_5678).await?;
        output.write_u32_le_async(0x9ABC_DEF0).await?;
        output
            .write_u32_async(0x1357_9BDF, ByteOrder::BigEndian)
            .await?;
        output
            .write_u32_async(0x2468_ACE0, ByteOrder::LittleEndian)
            .await?;

        output.write_u64_be_async(0x0123_4567_89AB_CDEF).await?;
        output.write_u64_le_async(0xFEDC_BA98_7654_3210).await?;
        output
            .write_u64_async(0x1111_2222_3333_4444, ByteOrder::BigEndian)
            .await?;
        output
            .write_u64_async(0xAAAA_BBBB_CCCC_DDDD, ByteOrder::LittleEndian)
            .await?;

        output
            .write_u128_be_async(0x0123_4567_89AB_CDEF_0011_2233_4455_6677)
            .await?;
        output
            .write_u128_le_async(0x8899_AABB_CCDD_EEFF_FEDC_BA98_7654_3210)
            .await?;
        output
            .write_u128_async(
                0x1020_3040_5060_7080_90A0_B0C0_D0E0_F000,
                ByteOrder::BigEndian,
            )
            .await?;
        output
            .write_u128_async(
                0x0F0E_0D0C_0B0A_0908_0706_0504_0302_0100,
                ByteOrder::LittleEndian,
            )
            .await?;

        output.write_i16_be_async(-1234).await?;
        output.write_i16_le_async(2345).await?;
        output.write_i16_async(-3456, ByteOrder::BigEndian).await?;
        output
            .write_i16_async(4567, ByteOrder::LittleEndian)
            .await?;

        output.write_i32_be_async(-123_456).await?;
        output.write_i32_le_async(234_567).await?;
        output
            .write_i32_async(-345_678, ByteOrder::BigEndian)
            .await?;
        output
            .write_i32_async(456_789, ByteOrder::LittleEndian)
            .await?;

        output.write_i64_be_async(-1_234_567_890).await?;
        output.write_i64_le_async(2_345_678_901).await?;
        output
            .write_i64_async(-3_456_789_012, ByteOrder::BigEndian)
            .await?;
        output
            .write_i64_async(4_567_890_123, ByteOrder::LittleEndian)
            .await?;

        output
            .write_i128_be_async(-12_345_678_901_234_567_890)
            .await?;
        output
            .write_i128_le_async(23_456_789_012_345_678_901)
            .await?;
        output
            .write_i128_async(-34_567_890_123_456_789_012, ByteOrder::BigEndian)
            .await?;
        output
            .write_i128_async(
                45_678_901_234_567_890_123,
                ByteOrder::LittleEndian,
            )
            .await?;

        output.write_f32_be_async(1.25).await?;
        output.write_f32_le_async(-2.5).await?;
        output.write_f32_async(3.75, ByteOrder::BigEndian).await?;
        output
            .write_f32_async(-4.5, ByteOrder::LittleEndian)
            .await?;

        output.write_f64_be_async(10.25).await?;
        output.write_f64_le_async(-20.5).await?;
        output.write_f64_async(30.75, ByteOrder::BigEndian).await?;
        output
            .write_f64_async(-40.125, ByteOrder::LittleEndian)
            .await?;

        Result::<()>::Ok(())
    })
    .expect("all scalar values should write");

    let encoded = bytes.lock().expect("lock should succeed").clone();
    let mut input = ChunkedAsyncInput::new(encoded);
    complete(async {
        assert_eq!(0xA5, input.read_u8_async().await?);
        assert_eq!(-0x25, input.read_i8_async().await?);

        assert_eq!(0x1234, input.read_u16_be_async().await?);
        assert_eq!(0x5678, input.read_u16_le_async().await?);
        assert_eq!(
            0x9ABC,
            input.read_u16_async(ByteOrder::BigEndian).await?,
        );
        assert_eq!(
            0xDEF0,
            input.read_u16_async(ByteOrder::LittleEndian).await?,
        );
        assert_eq!(
            0x1357,
            input.read_u16_async(ByteOrder::NativeEndian).await?,
        );

        assert_eq!(0x1234_5678, input.read_u32_be_async().await?);
        assert_eq!(0x9ABC_DEF0, input.read_u32_le_async().await?);
        assert_eq!(
            0x1357_9BDF,
            input.read_u32_async(ByteOrder::BigEndian).await?,
        );
        assert_eq!(
            0x2468_ACE0,
            input.read_u32_async(ByteOrder::LittleEndian).await?,
        );

        assert_eq!(
            0x0123_4567_89AB_CDEF,
            input.read_u64_be_async().await?,
        );
        assert_eq!(
            0xFEDC_BA98_7654_3210,
            input.read_u64_le_async().await?,
        );
        assert_eq!(
            0x1111_2222_3333_4444,
            input.read_u64_async(ByteOrder::BigEndian).await?,
        );
        assert_eq!(
            0xAAAA_BBBB_CCCC_DDDD,
            input.read_u64_async(ByteOrder::LittleEndian).await?,
        );

        assert_eq!(
            0x0123_4567_89AB_CDEF_0011_2233_4455_6677,
            input.read_u128_be_async().await?,
        );
        assert_eq!(
            0x8899_AABB_CCDD_EEFF_FEDC_BA98_7654_3210,
            input.read_u128_le_async().await?,
        );
        assert_eq!(
            0x1020_3040_5060_7080_90A0_B0C0_D0E0_F000,
            input.read_u128_async(ByteOrder::BigEndian).await?,
        );
        assert_eq!(
            0x0F0E_0D0C_0B0A_0908_0706_0504_0302_0100,
            input.read_u128_async(ByteOrder::LittleEndian).await?,
        );

        assert_eq!(-1234, input.read_i16_be_async().await?);
        assert_eq!(2345, input.read_i16_le_async().await?);
        assert_eq!(
            -3456,
            input.read_i16_async(ByteOrder::BigEndian).await?,
        );
        assert_eq!(
            4567,
            input.read_i16_async(ByteOrder::LittleEndian).await?,
        );

        assert_eq!(-123_456, input.read_i32_be_async().await?);
        assert_eq!(234_567, input.read_i32_le_async().await?);
        assert_eq!(
            -345_678,
            input.read_i32_async(ByteOrder::BigEndian).await?,
        );
        assert_eq!(
            456_789,
            input.read_i32_async(ByteOrder::LittleEndian).await?,
        );

        assert_eq!(-1_234_567_890, input.read_i64_be_async().await?);
        assert_eq!(2_345_678_901, input.read_i64_le_async().await?);
        assert_eq!(
            -3_456_789_012,
            input.read_i64_async(ByteOrder::BigEndian).await?,
        );
        assert_eq!(
            4_567_890_123,
            input.read_i64_async(ByteOrder::LittleEndian).await?,
        );

        assert_eq!(
            -12_345_678_901_234_567_890,
            input.read_i128_be_async().await?,
        );
        assert_eq!(
            23_456_789_012_345_678_901,
            input.read_i128_le_async().await?,
        );
        assert_eq!(
            -34_567_890_123_456_789_012,
            input.read_i128_async(ByteOrder::BigEndian).await?,
        );
        assert_eq!(
            45_678_901_234_567_890_123,
            input.read_i128_async(ByteOrder::LittleEndian).await?,
        );

        assert_eq!(1.25, input.read_f32_be_async().await?);
        assert_eq!(-2.5, input.read_f32_le_async().await?);
        assert_eq!(3.75, input.read_f32_async(ByteOrder::BigEndian).await?);
        assert_eq!(
            -4.5,
            input.read_f32_async(ByteOrder::LittleEndian).await?,
        );

        assert_eq!(10.25, input.read_f64_be_async().await?);
        assert_eq!(-20.5, input.read_f64_le_async().await?);
        assert_eq!(30.75, input.read_f64_async(ByteOrder::BigEndian).await?);
        assert_eq!(
            -40.125,
            input.read_f64_async(ByteOrder::LittleEndian).await?,
        );

        Result::<()>::Ok(())
    })
    .expect("all scalar values should read");
}

#[test]
fn async_string_extensions_cover_every_length_prefix() {
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let mut output = ChunkedAsyncOutput::new(bytes.clone());

    complete(async {
        output.write_utf8_payload_async("payload").await?;
        output.write_utf8_string_uleb_async("uleb").await?;
        output.write_utf8_string_uleb_async("uleb-strict").await?;
        output.write_utf8_string_uleb_u64_async("uleb-u64").await?;
        output
            .write_utf8_string_uleb_u64_async("uleb-u64-strict")
            .await?;
        output
            .write_utf8_string_u16_async("u16-be", ByteOrder::BigEndian)
            .await?;
        output
            .write_utf8_string_u16_async("u16-le", ByteOrder::LittleEndian)
            .await?;
        output
            .write_utf8_string_u16_be_async("fixed-u16-be")
            .await?;
        output
            .write_utf8_string_u16_le_async("fixed-u16-le")
            .await?;
        output
            .write_utf8_string_u32_async("u32-be", ByteOrder::BigEndian)
            .await?;
        output
            .write_utf8_string_u32_async("u32-le", ByteOrder::LittleEndian)
            .await?;
        output
            .write_utf8_string_u32_be_async("fixed-u32-be")
            .await?;
        output
            .write_utf8_string_u32_le_async("fixed-u32-le")
            .await?;
        Result::<()>::Ok(())
    })
    .expect("all string forms should write");

    let encoded = bytes.lock().expect("lock should succeed").clone();
    let mut input = ChunkedAsyncInput::new(encoded);
    complete(async {
        assert_eq!(
            "payload",
            input.read_utf8_payload_async("payload".len(), 32).await?,
        );
        assert_eq!("uleb", input.read_utf8_string_uleb_async(32).await?);
        assert_eq!(
            "uleb-strict",
            input.read_utf8_string_uleb_strict_async(32).await?,
        );
        assert_eq!(
            "uleb-u64",
            input.read_utf8_string_uleb_u64_async(32).await?,
        );
        assert_eq!(
            "uleb-u64-strict",
            input.read_utf8_string_uleb_u64_strict_async(32).await?,
        );
        assert_eq!(
            "u16-be",
            input
                .read_utf8_string_u16_async(ByteOrder::BigEndian, 32)
                .await?,
        );
        assert_eq!(
            "u16-le",
            input
                .read_utf8_string_u16_async(ByteOrder::LittleEndian, 32)
                .await?,
        );
        assert_eq!(
            "fixed-u16-be",
            input.read_utf8_string_u16_be_async(32).await?,
        );
        assert_eq!(
            "fixed-u16-le",
            input.read_utf8_string_u16_le_async(32).await?,
        );
        assert_eq!(
            "u32-be",
            input
                .read_utf8_string_u32_async(ByteOrder::BigEndian, 32)
                .await?,
        );
        assert_eq!(
            "u32-le",
            input
                .read_utf8_string_u32_async(ByteOrder::LittleEndian, 32)
                .await?,
        );
        assert_eq!(
            "fixed-u32-be",
            input.read_utf8_string_u32_be_async(32).await?,
        );
        assert_eq!(
            "fixed-u32-le",
            input.read_utf8_string_u32_le_async(32).await?,
        );
        Result::<()>::Ok(())
    })
    .expect("all string forms should read");
}

#[test]
fn async_string_reads_propagate_every_prefix_error() {
    macro_rules! assert_unexpected_eof {
        ($future:expr) => {{
            let error =
                complete($future).expect_err("empty prefix should fail");
            assert_eq!(ErrorKind::UnexpectedEof, error.kind());
        }};
    }

    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_unexpected_eof!(input.read_utf8_string_uleb_async(32));
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_unexpected_eof!(input.read_utf8_string_uleb_strict_async(32));
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_unexpected_eof!(input.read_utf8_string_uleb_u64_async(32));
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_unexpected_eof!(input.read_utf8_string_uleb_u64_strict_async(32));
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_unexpected_eof!(
        input.read_utf8_string_u16_async(ByteOrder::BigEndian, 32)
    );
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_unexpected_eof!(input.read_utf8_string_u16_be_async(32));
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_unexpected_eof!(input.read_utf8_string_u16_le_async(32));
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_unexpected_eof!(
        input.read_utf8_string_u32_async(ByteOrder::BigEndian, 32)
    );
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_unexpected_eof!(input.read_utf8_string_u32_be_async(32));
    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_unexpected_eof!(input.read_utf8_string_u32_le_async(32));

    let mut input = ChunkedAsyncInput::new(Vec::new());
    assert_unexpected_eof!(input.read_utf8_payload_async(1, 1));

    let mut input = ChunkedAsyncInput::new(Vec::new());
    let error = complete(input.read_utf8_payload_async(2, 1))
        .expect_err("oversized payload should fail before reading");
    assert_eq!(ErrorKind::InvalidData, error.kind());

    let mut input = ChunkedAsyncInput::new(vec![0x80; 10]);
    let error = complete(input.read_uleb_u64_async())
        .expect_err("maximum-length unterminated LEB128 should fail");
    assert_eq!(ErrorKind::InvalidData, error.kind());
}

#[test]
fn async_string_writes_propagate_length_and_prefix_errors() {
    macro_rules! assert_write_error {
        ($future:expr) => {{
            let error =
                complete($future).expect_err("prefix write should fail");
            assert_eq!(ErrorKind::BrokenPipe, error.kind());
        }};
    }

    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);
    assert_write_error!(output.write_utf8_string_uleb_async("x"));
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);
    assert_write_error!(output.write_utf8_string_uleb_u64_async("x"));
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);
    assert_write_error!(
        output.write_utf8_string_u16_async("x", ByteOrder::BigEndian)
    );
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);
    assert_write_error!(output.write_utf8_string_u16_be_async("x"));
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);
    assert_write_error!(output.write_utf8_string_u16_le_async("x"));
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);
    assert_write_error!(
        output.write_utf8_string_u32_async("x", ByteOrder::BigEndian)
    );
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);
    assert_write_error!(output.write_utf8_string_u32_be_async("x"));
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);
    assert_write_error!(output.write_utf8_string_u32_le_async("x"));

    let oversized = "x".repeat(usize::from(u16::MAX) + 1);
    let mut output = ChunkedAsyncOutput::new(Arc::new(Mutex::new(Vec::new())));
    let error = complete(
        output.write_utf8_string_u16_async(&oversized, ByteOrder::BigEndian),
    )
    .expect_err("oversized u16 string should fail");
    assert_eq!(ErrorKind::InvalidInput, error.kind());
    let error = complete(output.write_utf8_string_u16_be_async(&oversized))
        .expect_err("oversized big-endian u16 string should fail");
    assert_eq!(ErrorKind::InvalidInput, error.kind());
    let error = complete(output.write_utf8_string_u16_le_async(&oversized))
        .expect_err("oversized little-endian u16 string should fail");
    assert_eq!(ErrorKind::InvalidInput, error.kind());
}

#[test]
fn async_variable_length_and_string_extensions_share_codec_semantics() {
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let mut output = ChunkedAsyncOutput::new(bytes.clone());

    complete(output.write_uleb_u64_async(300)).expect("LEB128 should write");
    complete(output.write_zig_zag_i64_async(-42)).expect("ZigZag should write");
    complete(output.write_utf8_string_u32_async("异步", ByteOrder::BigEndian))
        .expect("string should write");

    let encoded = bytes.lock().expect("lock should succeed").clone();
    let mut input = ChunkedAsyncInput::new(encoded);
    assert_eq!(
        300,
        complete(input.read_uleb_u64_strict_async())
            .expect("strict LEB128 should read"),
    );
    assert_eq!(
        -42,
        complete(input.read_zig_zag_i64_strict_async())
            .expect("strict ZigZag should read"),
    );
    assert_eq!(
        "异步",
        complete(input.read_utf8_string_u32_async(ByteOrder::BigEndian, 32,))
            .expect("string should read"),
    );
}

#[test]
fn async_binary_read_reports_unexpected_eof_for_truncated_values() {
    let mut input = ChunkedAsyncInput::new(vec![0x12]);

    let error = complete(input.read_u32_be_async())
        .expect_err("truncated scalar should fail");

    assert_eq!(std::io::ErrorKind::UnexpectedEof, error.kind());
}
