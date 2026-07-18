// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow source-test-pair
//! Asynchronous fixed-width binary writes.

use std::io::Result;

use qubit_codec::{
    BigEndian,
    ByteOrder,
    LittleEndian,
};
use qubit_codec_binary::BinaryCodec;
use qubit_io::AsyncOutput;

use crate::util::{
    encode_infallible_unchecked,
    write_all_async,
};

macro_rules! write_binary_value_async {
    ($writer:expr, $value:expr, $ty:ty, $order:ty) => {
        write_binary_async::<
            { BinaryCodec::<$ty, $order>::MAX_UNITS_PER_VALUE },
            _,
            _,
            _,
        >($writer, $value, |bytes, value| {
            type Codec = BinaryCodec<$ty, $order>;
            // SAFETY: The local buffer has exactly the codec's fixed width.
            unsafe {
                let _ = encode_infallible_unchecked::<Codec>(value, bytes, 0);
            }
        })
        .await
    };
}

macro_rules! fixed_write_method {
    ($doc:literal, $name:ident, $ty:ty, $order:ty) => {
        #[doc = $doc]
        async fn $name(&mut self, value: $ty) -> Result<()>
        where
            Self: Unpin,
        {
            write_binary_value_async!(self, value, $ty, $order)
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

/// Future-based fixed-width binary writes to runtime-neutral async outputs.
#[allow(async_fn_in_trait)]
pub trait AsyncBinaryWriteExt: AsyncOutput<Item = u8> {
    fixed_write_method!(
        "Asynchronously writes an unsigned 8-bit integer.",
        write_u8_async,
        u8,
        BigEndian
    );
    fixed_write_method!(
        "Asynchronously writes a signed 8-bit integer.",
        write_i8_async,
        i8,
        BigEndian
    );

    /// Asynchronously writes a `u16` using `byte_order`.
    async fn write_u16_async(
        &mut self,
        value: u16,
        byte_order: ByteOrder,
    ) -> Result<()>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.write_u16_be_async(value).await
        } else {
            self.write_u16_le_async(value).await
        }
    }

    fixed_write_method!(
        "Asynchronously writes a big-endian `u16`.",
        write_u16_be_async,
        u16,
        BigEndian
    );
    fixed_write_method!(
        "Asynchronously writes a little-endian `u16`.",
        write_u16_le_async,
        u16,
        LittleEndian
    );

    /// Asynchronously writes a `u32` using `byte_order`.
    async fn write_u32_async(
        &mut self,
        value: u32,
        byte_order: ByteOrder,
    ) -> Result<()>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.write_u32_be_async(value).await
        } else {
            self.write_u32_le_async(value).await
        }
    }

    fixed_write_method!(
        "Asynchronously writes a big-endian `u32`.",
        write_u32_be_async,
        u32,
        BigEndian
    );
    fixed_write_method!(
        "Asynchronously writes a little-endian `u32`.",
        write_u32_le_async,
        u32,
        LittleEndian
    );

    /// Asynchronously writes a `u64` using `byte_order`.
    async fn write_u64_async(
        &mut self,
        value: u64,
        byte_order: ByteOrder,
    ) -> Result<()>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.write_u64_be_async(value).await
        } else {
            self.write_u64_le_async(value).await
        }
    }

    fixed_write_method!(
        "Asynchronously writes a big-endian `u64`.",
        write_u64_be_async,
        u64,
        BigEndian
    );
    fixed_write_method!(
        "Asynchronously writes a little-endian `u64`.",
        write_u64_le_async,
        u64,
        LittleEndian
    );

    /// Asynchronously writes a `u128` using `byte_order`.
    async fn write_u128_async(
        &mut self,
        value: u128,
        byte_order: ByteOrder,
    ) -> Result<()>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.write_u128_be_async(value).await
        } else {
            self.write_u128_le_async(value).await
        }
    }

    fixed_write_method!(
        "Asynchronously writes a big-endian `u128`.",
        write_u128_be_async,
        u128,
        BigEndian
    );
    fixed_write_method!(
        "Asynchronously writes a little-endian `u128`.",
        write_u128_le_async,
        u128,
        LittleEndian
    );

    /// Asynchronously writes an `i16` using `byte_order`.
    async fn write_i16_async(
        &mut self,
        value: i16,
        byte_order: ByteOrder,
    ) -> Result<()>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.write_i16_be_async(value).await
        } else {
            self.write_i16_le_async(value).await
        }
    }

    fixed_write_method!(
        "Asynchronously writes a big-endian `i16`.",
        write_i16_be_async,
        i16,
        BigEndian
    );
    fixed_write_method!(
        "Asynchronously writes a little-endian `i16`.",
        write_i16_le_async,
        i16,
        LittleEndian
    );

    /// Asynchronously writes an `i32` using `byte_order`.
    async fn write_i32_async(
        &mut self,
        value: i32,
        byte_order: ByteOrder,
    ) -> Result<()>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.write_i32_be_async(value).await
        } else {
            self.write_i32_le_async(value).await
        }
    }

    fixed_write_method!(
        "Asynchronously writes a big-endian `i32`.",
        write_i32_be_async,
        i32,
        BigEndian
    );
    fixed_write_method!(
        "Asynchronously writes a little-endian `i32`.",
        write_i32_le_async,
        i32,
        LittleEndian
    );

    /// Asynchronously writes an `i64` using `byte_order`.
    async fn write_i64_async(
        &mut self,
        value: i64,
        byte_order: ByteOrder,
    ) -> Result<()>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.write_i64_be_async(value).await
        } else {
            self.write_i64_le_async(value).await
        }
    }

    fixed_write_method!(
        "Asynchronously writes a big-endian `i64`.",
        write_i64_be_async,
        i64,
        BigEndian
    );
    fixed_write_method!(
        "Asynchronously writes a little-endian `i64`.",
        write_i64_le_async,
        i64,
        LittleEndian
    );

    /// Asynchronously writes an `i128` using `byte_order`.
    async fn write_i128_async(
        &mut self,
        value: i128,
        byte_order: ByteOrder,
    ) -> Result<()>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.write_i128_be_async(value).await
        } else {
            self.write_i128_le_async(value).await
        }
    }

    fixed_write_method!(
        "Asynchronously writes a big-endian `i128`.",
        write_i128_be_async,
        i128,
        BigEndian
    );
    fixed_write_method!(
        "Asynchronously writes a little-endian `i128`.",
        write_i128_le_async,
        i128,
        LittleEndian
    );

    /// Asynchronously writes an `f32` using `byte_order`.
    async fn write_f32_async(
        &mut self,
        value: f32,
        byte_order: ByteOrder,
    ) -> Result<()>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.write_f32_be_async(value).await
        } else {
            self.write_f32_le_async(value).await
        }
    }

    fixed_write_method!(
        "Asynchronously writes a big-endian `f32`.",
        write_f32_be_async,
        f32,
        BigEndian
    );
    fixed_write_method!(
        "Asynchronously writes a little-endian `f32`.",
        write_f32_le_async,
        f32,
        LittleEndian
    );

    /// Asynchronously writes an `f64` using `byte_order`.
    async fn write_f64_async(
        &mut self,
        value: f64,
        byte_order: ByteOrder,
    ) -> Result<()>
    where
        Self: Unpin,
    {
        if use_big_endian(byte_order) {
            self.write_f64_be_async(value).await
        } else {
            self.write_f64_le_async(value).await
        }
    }

    fixed_write_method!(
        "Asynchronously writes a big-endian `f64`.",
        write_f64_be_async,
        f64,
        BigEndian
    );
    fixed_write_method!(
        "Asynchronously writes a little-endian `f64`.",
        write_f64_le_async,
        f64,
        LittleEndian
    );
}

impl<W> AsyncBinaryWriteExt for W where W: AsyncOutput<Item = u8> + ?Sized {}

async fn write_binary_async<const N: usize, T, W, F>(
    writer: &mut W,
    value: T,
    encode: F,
) -> Result<()>
where
    W: AsyncOutput<Item = u8> + Unpin + ?Sized,
    F: FnOnce(&mut [u8], T),
{
    let mut bytes = [0_u8; N];
    encode(&mut bytes, value);
    write_all_async(writer, &bytes).await
}
