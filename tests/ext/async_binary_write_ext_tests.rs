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
    AsyncBinaryWriteExt,
    BinaryWriteExt,
};

use super::internal::async_io_test_support_tests::{
    ChunkedAsyncOutput,
    assert_send,
    complete,
    poll_once,
};

#[allow(dead_code)]
fn assert_binary_write_future_is_send<T>(output: &mut T)
where
    T: AsyncOutput<Item = u8> + Send + Unpin + ?Sized,
{
    assert_send(output.write_u32_be_async(0));
}

#[test]
fn async_binary_write_covers_scalars_and_byte_orders() {
    let mut output = ChunkedAsyncOutput::new();
    complete(async {
        output.write_u8_async(0xA5).await?;
        output.write_i8_async(-0x25).await?;
        output.write_u16_be_async(0x1234).await?;
        output.write_u16_le_async(0x5678).await?;
        output
            .write_u16_async(0x9ABC, ByteOrder::NativeEndian)
            .await?;
        output.write_u32_be_async(0x1234_5678).await?;
        output.write_u32_le_async(0x9ABC_DEF0).await?;
        output
            .write_u32_async(0x1357_9BDF, ByteOrder::BigEndian)
            .await?;
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
        output
            .write_i128_be_async(-12_345_678_901_234_567_890)
            .await?;
        output
            .write_i128_le_async(23_456_789_012_345_678_901)
            .await?;
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
    expected
        .write_u32_be(0x1234_5678)
        .expect("u32 should encode");
    expected
        .write_u32_le(0x9ABC_DEF0)
        .expect("u32 should encode");
    expected
        .write_u32(0x1357_9BDF, ByteOrder::BigEndian)
        .expect("u32 should encode");
    expected
        .write_u64_be(0x0123_4567_89AB_CDEF)
        .expect("u64 should encode");
    expected
        .write_u64_le(0xFEDC_BA98_7654_3210)
        .expect("u64 should encode");
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
    expected
        .write_i64_be(-1_234_567_890)
        .expect("i64 should encode");
    expected
        .write_i64_le(2_345_678_901)
        .expect("i64 should encode");
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
fn dropping_binary_write_future_retains_partial_output() {
    let mut output = ChunkedAsyncOutput::starts_ready();

    assert!(matches!(
        poll_once(output.write_u32_be_async(0x1234_5678)),
        Poll::Pending,
    ));

    assert_eq!([0x12, 0x34], output.bytes().as_slice());
}

#[test]
fn async_binary_write_propagates_output_errors() {
    let mut output = ChunkedAsyncOutput::failing(ErrorKind::BrokenPipe);

    let error = complete(output.write_u32_be_async(0x1234_5678))
        .expect_err("scripted output should fail");

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}
