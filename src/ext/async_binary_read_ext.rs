// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow source-test-pair
//! Asynchronous fixed-width binary reads.

use std::io::Result;

use qubit_codec::{
    BigEndian,
    ByteOrder,
    LittleEndian,
};
use qubit_codec_binary::BinaryCodec;
use qubit_io::AsyncInput;

use crate::util::{
    decode_infallible_unchecked,
    read_exactly_async,
};

macro_rules! read_binary_value_async {
    ($reader:expr, $ty:ty, $order:ty) => {
        read_binary_async::<
            { BinaryCodec::<$ty, $order>::MIN_UNITS_PER_VALUE },
            _,
            _,
            _,
        >($reader, |bytes| {
            type Codec = BinaryCodec<$ty, $order>;
            // SAFETY: The local buffer has exactly the codec's fixed width.
            unsafe { decode_infallible_unchecked::<Codec>(bytes, 0) }
        })
        .await
    };
}

macro_rules! fixed_read_method {
    ($doc:literal, $name:ident, $ty:ty, $order:ty) => {
        #[doc = $doc]
        async fn $name(&mut self) -> Result<$ty>
        where
            Self: Unpin,
        {
            read_binary_value_async!(self, $ty, $order)
        }
    };
}

#[inline]
const fn use_big_endian(byte_order: ByteOrder) -> bool {
    match byte_order {
        ByteOrder::BigEndian => true,
        ByteOrder::LittleEndian => false,
        ByteOrder::NativeEndian => cfg!(target_endian = "big"),
    }
}

/// Future-based fixed-width binary reads from runtime-neutral async inputs.
#[allow(async_fn_in_trait)]
pub trait AsyncBinaryReadExt: AsyncInput<Item = u8> {
    fixed_read_method!(
        "Asynchronously reads an unsigned 8-bit integer.",
        read_u8_async,
        u8,
        BigEndian
    );
    fixed_read_method!(
        "Asynchronously reads a signed 8-bit integer.",
        read_i8_async,
        i8,
        BigEndian
    );

    /// Asynchronously reads a `u16` using `byte_order`.
    async fn read_u16_async(&mut self, byte_order: ByteOrder) -> Result<u16>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.read_u16_be_async().await
        } else {
            self.read_u16_le_async().await
        }
    }

    fixed_read_method!(
        "Asynchronously reads a big-endian `u16`.",
        read_u16_be_async,
        u16,
        BigEndian
    );
    fixed_read_method!(
        "Asynchronously reads a little-endian `u16`.",
        read_u16_le_async,
        u16,
        LittleEndian
    );

    /// Asynchronously reads a `u32` using `byte_order`.
    async fn read_u32_async(&mut self, byte_order: ByteOrder) -> Result<u32>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.read_u32_be_async().await
        } else {
            self.read_u32_le_async().await
        }
    }

    fixed_read_method!(
        "Asynchronously reads a big-endian `u32`.",
        read_u32_be_async,
        u32,
        BigEndian
    );
    fixed_read_method!(
        "Asynchronously reads a little-endian `u32`.",
        read_u32_le_async,
        u32,
        LittleEndian
    );

    /// Asynchronously reads a `u64` using `byte_order`.
    async fn read_u64_async(&mut self, byte_order: ByteOrder) -> Result<u64>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.read_u64_be_async().await
        } else {
            self.read_u64_le_async().await
        }
    }

    fixed_read_method!(
        "Asynchronously reads a big-endian `u64`.",
        read_u64_be_async,
        u64,
        BigEndian
    );
    fixed_read_method!(
        "Asynchronously reads a little-endian `u64`.",
        read_u64_le_async,
        u64,
        LittleEndian
    );

    /// Asynchronously reads a `u128` using `byte_order`.
    async fn read_u128_async(&mut self, byte_order: ByteOrder) -> Result<u128>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.read_u128_be_async().await
        } else {
            self.read_u128_le_async().await
        }
    }

    fixed_read_method!(
        "Asynchronously reads a big-endian `u128`.",
        read_u128_be_async,
        u128,
        BigEndian
    );
    fixed_read_method!(
        "Asynchronously reads a little-endian `u128`.",
        read_u128_le_async,
        u128,
        LittleEndian
    );

    /// Asynchronously reads an `i16` using `byte_order`.
    async fn read_i16_async(&mut self, byte_order: ByteOrder) -> Result<i16>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.read_i16_be_async().await
        } else {
            self.read_i16_le_async().await
        }
    }

    fixed_read_method!(
        "Asynchronously reads a big-endian `i16`.",
        read_i16_be_async,
        i16,
        BigEndian
    );
    fixed_read_method!(
        "Asynchronously reads a little-endian `i16`.",
        read_i16_le_async,
        i16,
        LittleEndian
    );

    /// Asynchronously reads an `i32` using `byte_order`.
    async fn read_i32_async(&mut self, byte_order: ByteOrder) -> Result<i32>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.read_i32_be_async().await
        } else {
            self.read_i32_le_async().await
        }
    }

    fixed_read_method!(
        "Asynchronously reads a big-endian `i32`.",
        read_i32_be_async,
        i32,
        BigEndian
    );
    fixed_read_method!(
        "Asynchronously reads a little-endian `i32`.",
        read_i32_le_async,
        i32,
        LittleEndian
    );

    /// Asynchronously reads an `i64` using `byte_order`.
    async fn read_i64_async(&mut self, byte_order: ByteOrder) -> Result<i64>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.read_i64_be_async().await
        } else {
            self.read_i64_le_async().await
        }
    }

    fixed_read_method!(
        "Asynchronously reads a big-endian `i64`.",
        read_i64_be_async,
        i64,
        BigEndian
    );
    fixed_read_method!(
        "Asynchronously reads a little-endian `i64`.",
        read_i64_le_async,
        i64,
        LittleEndian
    );

    /// Asynchronously reads an `i128` using `byte_order`.
    async fn read_i128_async(&mut self, byte_order: ByteOrder) -> Result<i128>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.read_i128_be_async().await
        } else {
            self.read_i128_le_async().await
        }
    }

    fixed_read_method!(
        "Asynchronously reads a big-endian `i128`.",
        read_i128_be_async,
        i128,
        BigEndian
    );
    fixed_read_method!(
        "Asynchronously reads a little-endian `i128`.",
        read_i128_le_async,
        i128,
        LittleEndian
    );

    /// Asynchronously reads an `f32` using `byte_order`.
    async fn read_f32_async(&mut self, byte_order: ByteOrder) -> Result<f32>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.read_f32_be_async().await
        } else {
            self.read_f32_le_async().await
        }
    }

    fixed_read_method!(
        "Asynchronously reads a big-endian `f32`.",
        read_f32_be_async,
        f32,
        BigEndian
    );
    fixed_read_method!(
        "Asynchronously reads a little-endian `f32`.",
        read_f32_le_async,
        f32,
        LittleEndian
    );

    /// Asynchronously reads an `f64` using `byte_order`.
    async fn read_f64_async(&mut self, byte_order: ByteOrder) -> Result<f64>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.read_f64_be_async().await
        } else {
            self.read_f64_le_async().await
        }
    }

    fixed_read_method!(
        "Asynchronously reads a big-endian `f64`.",
        read_f64_be_async,
        f64,
        BigEndian
    );
    fixed_read_method!(
        "Asynchronously reads a little-endian `f64`.",
        read_f64_le_async,
        f64,
        LittleEndian
    );
}

impl<R> AsyncBinaryReadExt for R where R: AsyncInput<Item = u8> + ?Sized {}

async fn read_binary_async<const N: usize, T, R, F>(
    reader: &mut R,
    decode: F,
) -> Result<T>
where
    R: AsyncInput<Item = u8> + Unpin + ?Sized,
    F: FnOnce(&[u8]) -> T,
{
    let mut bytes = [0_u8; N];
    read_exactly_async(reader, &mut bytes).await?;
    Ok(decode(&bytes))
}
