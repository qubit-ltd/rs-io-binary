// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::ErrorKind;
use std::io::Result;
use std::marker::PhantomData;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Context;
use std::task::Poll;

use qubit_codec::ByteOrder;
use qubit_io::AsyncOutput;
use qubit_io_binary::AsyncBinaryWriteExt;
use qubit_io_binary::BinaryWriteExt;

use super::internal::async_io_test_support_tests::ChunkedAsyncOutput;
use super::internal::async_io_test_support_tests::complete;
use super::internal::async_io_test_support_tests::poll_once;

macro_rules! assert_async_write_float_bits {
    ($runtime:ident, $be:ident, $le:ident, $float:ty, $bits:expr) => {{
        let bits: <$float as FloatBits>::Bits = $bits;
        let value = <$float>::from_bits(bits);
        let mut output = ChunkedAsyncOutput::new();
        complete(output.$runtime(value, ByteOrder::BigEndian)).expect("runtime BE float should write");
        assert_eq!(bits.to_be_bytes().as_slice(), output.bytes().as_slice());
        let mut output = ChunkedAsyncOutput::new();
        complete(output.$runtime(value, ByteOrder::LittleEndian)).expect("runtime LE float should write");
        assert_eq!(bits.to_le_bytes().as_slice(), output.bytes().as_slice());
        let mut output = ChunkedAsyncOutput::new();
        complete(output.$runtime(value, ByteOrder::NativeEndian)).expect("runtime native float should write");
        assert_eq!(bits.to_ne_bytes().as_slice(), output.bytes().as_slice());
        let mut output = ChunkedAsyncOutput::new();
        complete(output.$be(value)).expect("fixed BE float should write");
        assert_eq!(bits.to_be_bytes().as_slice(), output.bytes().as_slice());
        let mut output = ChunkedAsyncOutput::new();
        complete(output.$le(value)).expect("fixed LE float should write");
        assert_eq!(bits.to_le_bytes().as_slice(), output.bytes().as_slice());
    }};
}

trait FloatBits {
    type Bits: Copy + std::fmt::Debug + PartialEq;
}

impl FloatBits for f32 {
    type Bits = u32;
}
impl FloatBits for f64 {
    type Bits = u64;
}

struct NonSendAsyncOutput {
    inner: ChunkedAsyncOutput,
    _not_send: PhantomData<Rc<()>>,
}

impl NonSendAsyncOutput {
    fn new() -> Self {
        Self {
            inner: ChunkedAsyncOutput::starts_ready(),
            _not_send: PhantomData,
        }
    }
}

impl AsyncOutput for NonSendAsyncOutput {
    type Item = u8;

    unsafe fn poll_write_unchecked(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        input: &[u8],
        index: usize,
        count: usize,
    ) -> std::task::Poll<std::io::Result<usize>> {
        let this = self.get_mut();
        // SAFETY: The caller upholds the indexed input range contract.
        unsafe { Pin::new(&mut this.inner).poll_write_unchecked(cx, input, index, count) }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> std::task::Poll<std::io::Result<()>> {
        let this = self.get_mut();
        Pin::new(&mut this.inner).poll_flush(cx)
    }
}

#[test]
fn test_async_binary_write_accepts_non_send_output() {
    let mut output = NonSendAsyncOutput::new();
    complete(output.write_u16_be_async(0x1234)).expect("value should write");
    assert_eq!(vec![0x12, 0x34], output.inner.bytes());
}

#[test]
fn test_async_binary_write_covers_scalars_and_byte_orders() {
    let mut output = ChunkedAsyncOutput::new();
    complete(async {
        output.write_u8_async(0xA5).await?;
        output.write_i8_async(-0x25).await?;
        output.write_u16_be_async(0x1234).await?;
        output.write_u16_le_async(0x5678).await?;
        output.write_u16_async(0x9ABC, ByteOrder::NativeEndian).await?;
        output.write_u32_be_async(0x1234_5678).await?;
        output.write_u32_le_async(0x9ABC_DEF0).await?;
        output.write_u32_async(0x1357_9BDF, ByteOrder::BigEndian).await?;
        output.write_u64_be_async(0x0123_4567_89AB_CDEF).await?;
        output.write_u64_le_async(0xFEDC_BA98_7654_3210).await?;
        output
            .write_u128_be_async(0x0123_4567_89AB_CDEF_0011_2233_4455_6677)
            .await?;
        output
            .write_u128_le_async(0x8899_AABB_CCDD_EEFF_FEDC_BA98_7654_3210)
            .await?;
        output.write_i16_be_async(-1234).await?;
        output.write_i16_le_async(2345).await?;
        output.write_i32_be_async(-123_456).await?;
        output.write_i32_le_async(234_567).await?;
        output.write_i64_be_async(-1_234_567_890).await?;
        output.write_i64_le_async(2_345_678_901).await?;
        output.write_i128_be_async(-12_345_678_901_234_567_890).await?;
        output.write_i128_le_async(23_456_789_012_345_678_901).await?;
        output.write_f32_be_async(1.25).await?;
        output.write_f32_le_async(-2.5).await?;
        output.write_f64_be_async(10.25).await?;
        output.write_f64_le_async(-20.5).await?;
        Result::<()>::Ok(())
    })
    .expect("all scalar values should write");

    let mut expected = Vec::new();
    expected.write_u8(0xA5).expect("u8 should encode");
    expected.write_i8(-0x25).expect("i8 should encode");
    expected.write_u16_be(0x1234).expect("u16 should encode");
    expected.write_u16_le(0x5678).expect("u16 should encode");
    expected
        .write_u16(0x9ABC, ByteOrder::NativeEndian)
        .expect("u16 should encode");
    expected.write_u32_be(0x1234_5678).expect("u32 should encode");
    expected.write_u32_le(0x9ABC_DEF0).expect("u32 should encode");
    expected
        .write_u32(0x1357_9BDF, ByteOrder::BigEndian)
        .expect("u32 should encode");
    expected.write_u64_be(0x0123_4567_89AB_CDEF).expect("u64 should encode");
    expected.write_u64_le(0xFEDC_BA98_7654_3210).expect("u64 should encode");
    expected
        .write_u128_be(0x0123_4567_89AB_CDEF_0011_2233_4455_6677)
        .expect("u128 should encode");
    expected
        .write_u128_le(0x8899_AABB_CCDD_EEFF_FEDC_BA98_7654_3210)
        .expect("u128 should encode");
    expected.write_i16_be(-1234).expect("i16 should encode");
    expected.write_i16_le(2345).expect("i16 should encode");
    expected.write_i32_be(-123_456).expect("i32 should encode");
    expected.write_i32_le(234_567).expect("i32 should encode");
    expected.write_i64_be(-1_234_567_890).expect("i64 should encode");
    expected.write_i64_le(2_345_678_901).expect("i64 should encode");
    expected
        .write_i128_be(-12_345_678_901_234_567_890)
        .expect("i128 should encode");
    expected
        .write_i128_le(23_456_789_012_345_678_901)
        .expect("i128 should encode");
    expected.write_f32_be(1.25).expect("f32 should encode");
    expected.write_f32_le(-2.5).expect("f32 should encode");
    expected.write_f64_be(10.25).expect("f64 should encode");
    expected.write_f64_le(-20.5).expect("f64 should encode");

    assert_eq!(expected, output.bytes());
}

#[test]
fn test_async_binary_write_float_bits() {
    for bits in [0x8000_0000_u32, 0x7f80_0000, 0xff80_0000, 0x7fc1_2345, 0xffc5_4321] {
        assert_async_write_float_bits!(write_f32_async, write_f32_be_async, write_f32_le_async, f32, bits);
    }
    for bits in [
        0x8000_0000_0000_0000_u64,
        0x7ff0_0000_0000_0000,
        0xfff0_0000_0000_0000,
        0x7ff8_1234_5678_9abc,
        0xfff8_abcd_ef01_2345,
    ] {
        assert_async_write_float_bits!(write_f64_async, write_f64_be_async, write_f64_le_async, f64, bits);
    }
}

#[test]
fn test_dropping_binary_write_future_retains_partial_output() {
    let mut output = ChunkedAsyncOutput::starts_ready();

    assert!(matches!(
        poll_once(output.write_u32_be_async(0x1234_5678)),
        Poll::Pending,
    ));

    assert_eq!([0x12, 0x34], output.bytes().as_slice());
}

#[test]
fn test_async_binary_write_propagates_output_errors() {
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);

    let error = complete(output.write_u32_be_async(0x1234_5678)).expect_err("scripted output should fail");

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}
