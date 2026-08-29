// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::ErrorKind;
use std::marker::PhantomData;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Context;
use std::task::Poll;

use qubit_codec::ByteOrder;
use qubit_io::AsyncInput;
use qubit_io_binary::AsyncBinaryReadExt;
use qubit_io_binary::BinaryWriteExt;

use super::internal::async_io_test_support_tests::ChunkedAsyncInput;
use super::internal::async_io_test_support_tests::complete;
use super::internal::async_io_test_support_tests::poll_once;

struct NonSendAsyncInput {
    inner: ChunkedAsyncInput,
    _not_send: PhantomData<Rc<()>>,
}

impl NonSendAsyncInput {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            inner: ChunkedAsyncInput::starts_ready(bytes),
            _not_send: PhantomData,
        }
    }
}

impl AsyncInput for NonSendAsyncInput {
    type Item = u8;

    unsafe fn poll_read_unchecked(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        output: &mut [u8],
        index: usize,
        count: usize,
    ) -> std::task::Poll<std::io::Result<usize>> {
        let this = self.get_mut();
        // SAFETY: The caller upholds the indexed output range contract.
        unsafe { Pin::new(&mut this.inner).poll_read_unchecked(cx, output, index, count) }
    }
}

fn scalar_fixture() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.write_u8(0xA5).expect("u8 should encode");
    bytes.write_i8(-0x25).expect("i8 should encode");
    bytes.write_u16_be(0x1234).expect("u16 should encode");
    bytes.write_u16_le(0x5678).expect("u16 should encode");
    bytes
        .write_u16(0x9ABC, ByteOrder::NativeEndian)
        .expect("u16 should encode");
    bytes.write_u32_be(0x1234_5678).expect("u32 should encode");
    bytes.write_u32_le(0x9ABC_DEF0).expect("u32 should encode");
    bytes
        .write_u32(0x1357_9BDF, ByteOrder::BigEndian)
        .expect("u32 should encode");
    bytes.write_u64_be(0x0123_4567_89AB_CDEF).expect("u64 should encode");
    bytes.write_u64_le(0xFEDC_BA98_7654_3210).expect("u64 should encode");
    bytes
        .write_u128_be(0x0123_4567_89AB_CDEF_0011_2233_4455_6677)
        .expect("u128 should encode");
    bytes
        .write_u128_le(0x8899_AABB_CCDD_EEFF_FEDC_BA98_7654_3210)
        .expect("u128 should encode");
    bytes.write_i16_be(-1234).expect("i16 should encode");
    bytes.write_i16_le(2345).expect("i16 should encode");
    bytes.write_i32_be(-123_456).expect("i32 should encode");
    bytes.write_i32_le(234_567).expect("i32 should encode");
    bytes.write_i64_be(-1_234_567_890).expect("i64 should encode");
    bytes.write_i64_le(2_345_678_901).expect("i64 should encode");
    bytes
        .write_i128_be(-12_345_678_901_234_567_890)
        .expect("i128 should encode");
    bytes
        .write_i128_le(23_456_789_012_345_678_901)
        .expect("i128 should encode");
    bytes.write_f32_be(1.25).expect("f32 should encode");
    bytes.write_f32_le(-2.5).expect("f32 should encode");
    bytes.write_f64_be(10.25).expect("f64 should encode");
    bytes.write_f64_le(-20.5).expect("f64 should encode");
    bytes
}

#[test]
fn async_binary_read_accepts_non_send_input() {
    let mut input = NonSendAsyncInput::new(vec![0x12, 0x34]);
    assert_eq!(0x1234, complete(input.read_u16_be_async()).unwrap());
}

#[test]
fn async_binary_read_covers_scalars_and_byte_orders() {
    let mut input = ChunkedAsyncInput::new(scalar_fixture());

    assert_eq!(0xA5, complete(input.read_u8_async()).unwrap());
    assert_eq!(-0x25, complete(input.read_i8_async()).unwrap());
    assert_eq!(0x1234, complete(input.read_u16_be_async()).unwrap());
    assert_eq!(0x5678, complete(input.read_u16_le_async()).unwrap());
    assert_eq!(0x9ABC, complete(input.read_u16_async(ByteOrder::NativeEndian)).unwrap(),);
    assert_eq!(0x1234_5678, complete(input.read_u32_be_async()).unwrap(),);
    assert_eq!(0x9ABC_DEF0, complete(input.read_u32_le_async()).unwrap(),);
    assert_eq!(
        0x1357_9BDF,
        complete(input.read_u32_async(ByteOrder::BigEndian)).unwrap(),
    );
    assert_eq!(0x0123_4567_89AB_CDEF, complete(input.read_u64_be_async()).unwrap(),);
    assert_eq!(0xFEDC_BA98_7654_3210, complete(input.read_u64_le_async()).unwrap(),);
    assert_eq!(
        0x0123_4567_89AB_CDEF_0011_2233_4455_6677,
        complete(input.read_u128_be_async()).unwrap(),
    );
    assert_eq!(
        0x8899_AABB_CCDD_EEFF_FEDC_BA98_7654_3210,
        complete(input.read_u128_le_async()).unwrap(),
    );
    assert_eq!(-1234, complete(input.read_i16_be_async()).unwrap());
    assert_eq!(2345, complete(input.read_i16_le_async()).unwrap());
    assert_eq!(-123_456, complete(input.read_i32_be_async()).unwrap(),);
    assert_eq!(234_567, complete(input.read_i32_le_async()).unwrap());
    assert_eq!(-1_234_567_890, complete(input.read_i64_be_async()).unwrap(),);
    assert_eq!(2_345_678_901, complete(input.read_i64_le_async()).unwrap(),);
    assert_eq!(
        -12_345_678_901_234_567_890,
        complete(input.read_i128_be_async()).unwrap(),
    );
    assert_eq!(
        23_456_789_012_345_678_901,
        complete(input.read_i128_le_async()).unwrap(),
    );
    assert_eq!(1.25, complete(input.read_f32_be_async()).unwrap());
    assert_eq!(-2.5, complete(input.read_f32_le_async()).unwrap());
    assert_eq!(10.25, complete(input.read_f64_be_async()).unwrap());
    assert_eq!(-20.5, complete(input.read_f64_le_async()).unwrap());
}

#[test]
fn dropping_binary_read_future_retains_consumed_input() {
    let mut input = ChunkedAsyncInput::starts_ready(vec![0x12, 0x34, 0x56, 0x78]);

    assert!(matches!(poll_once(input.read_u32_be_async()), Poll::Pending,));

    assert_eq!(2, input.position());
}

#[test]
fn async_binary_read_reports_truncated_values() {
    let mut input = ChunkedAsyncInput::new(vec![0x12]);

    let error = complete(input.read_u32_be_async()).expect_err("truncated scalar should fail");

    assert_eq!(ErrorKind::UnexpectedEof, error.kind());
}
