// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Asynchronous fixed-width binary writes.

use core::future::Future;
use std::io::Result;

use qubit_codec::BigEndian;
use qubit_codec::ByteOrder;
use qubit_codec::LittleEndian;
use qubit_codec_binary::BinaryCodec;
use qubit_io::AsyncOutput;

use crate::util::encode_infallible_unchecked;
use crate::util::write_all_async;

macro_rules! write_binary_value_async {
    ($writer:expr, $value:expr, $ty:ty, $order:ty) => {
        write_binary_async::<
            { BinaryCodec::<$ty, $order>::MAX_ENCODE_UNITS_PER_VALUE },
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
        #[doc = ""]
        #[doc = "# Parameters"]
        #[doc = ""]
        #[doc = "- `value`: Value to encode and write."]
        #[doc = ""]
        #[doc = "# Returns"]
        #[doc = ""]
        #[doc = "Returns a future that completes after all encoded bytes have \
                 been written."]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns an output error, including a write-zero error when \
                 the output stops making progress."]
        #[doc = ""]
        #[doc = "# Cancellation safety"]
        #[doc = ""]
        #[doc = "This operation is not cancellation safe. Dropping the future \
                 retains any bytes already written to the output."]
        #[inline(always)]
        fn $name(&mut self, value: $ty) -> impl Future<Output = Result<()>> + '_
        where
            Self: Unpin,
        {
            async move { write_binary_value_async!(self, value, $ty, $order) }
        }
    };
}

macro_rules! runtime_order_write_method {
    ($doc:literal, $name:ident, $ty:ty, $big:ident, $little:ident) => {
        #[doc = $doc]
        #[doc = ""]
        #[doc = "# Parameters"]
        #[doc = ""]
        #[doc = "- `value`: Value to encode and write."]
        #[doc = "- `byte_order`: Byte order used to encode the value."]
        #[doc = ""]
        #[doc = "# Returns"]
        #[doc = ""]
        #[doc = "Returns a future that completes after all encoded bytes have \
                 been written."]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns an output error, including a write-zero error when \
                 the output stops making progress."]
        #[doc = ""]
        #[doc = "# Cancellation safety"]
        #[doc = ""]
        #[doc = "This operation is not cancellation safe. Dropping the future \
                 retains any bytes already written to the output."]
        #[inline]
        fn $name(
            &mut self,
            value: $ty,
            byte_order: ByteOrder,
        ) -> impl Future<Output = Result<()>> + '_
        where
            Self: Unpin,
        {
            async move {
                if use_big_endian(byte_order) {
                    self.$big(value).await
                } else {
                    self.$little(value).await
                }
            }
        }
    };
}

/// Resolves a runtime byte-order choice to a big-endian branch.
///
/// # Parameters
///
/// - `byte_order`: Runtime byte-order selection.
///
/// # Returns
///
/// Returns `true` for big endian, `false` for little endian, and the target's
/// native ordering for [`ByteOrder::NativeEndian`].
#[must_use]
#[inline]
const fn use_big_endian(byte_order: ByteOrder) -> bool {
    match byte_order {
        ByteOrder::BigEndian => true,
        ByteOrder::LittleEndian => false,
        ByteOrder::NativeEndian => cfg!(target_endian = "big"),
    }
}

/// Future-based fixed-width binary writes to runtime-neutral async outputs.
///
/// # Cancellation safety
///
/// These writes are not cancellation safe. Dropping a pending future leaves
/// any already-written prefix in the output; retrying can duplicate that
/// prefix.
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

    runtime_order_write_method!(
        "Asynchronously writes a `u16` using a runtime byte order.",
        write_u16_async,
        u16,
        write_u16_be_async,
        write_u16_le_async
    );

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

    runtime_order_write_method!(
        "Asynchronously writes a `u32` using a runtime byte order.",
        write_u32_async,
        u32,
        write_u32_be_async,
        write_u32_le_async
    );

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

    runtime_order_write_method!(
        "Asynchronously writes a `u64` using a runtime byte order.",
        write_u64_async,
        u64,
        write_u64_be_async,
        write_u64_le_async
    );

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

    runtime_order_write_method!(
        "Asynchronously writes a `u128` using a runtime byte order.",
        write_u128_async,
        u128,
        write_u128_be_async,
        write_u128_le_async
    );

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

    runtime_order_write_method!(
        "Asynchronously writes an `i16` using a runtime byte order.",
        write_i16_async,
        i16,
        write_i16_be_async,
        write_i16_le_async
    );

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

    runtime_order_write_method!(
        "Asynchronously writes an `i32` using a runtime byte order.",
        write_i32_async,
        i32,
        write_i32_be_async,
        write_i32_le_async
    );

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

    runtime_order_write_method!(
        "Asynchronously writes an `i64` using a runtime byte order.",
        write_i64_async,
        i64,
        write_i64_be_async,
        write_i64_le_async
    );

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

    runtime_order_write_method!(
        "Asynchronously writes an `i128` using a runtime byte order.",
        write_i128_async,
        i128,
        write_i128_be_async,
        write_i128_le_async
    );

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

    runtime_order_write_method!(
        "Asynchronously writes an `f32` using a runtime byte order.",
        write_f32_async,
        f32,
        write_f32_be_async,
        write_f32_le_async
    );

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

    runtime_order_write_method!(
        "Asynchronously writes an `f64` using a runtime byte order.",
        write_f64_async,
        f64,
        write_f64_be_async,
        write_f64_le_async
    );

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

/// Encodes and writes one fixed-width value asynchronously.
///
/// # Type Parameters
///
/// - `N`: Encoded scalar width in bytes.
/// - `T`: Value type accepted by the encoder.
/// - `W`: Destination asynchronous byte output.
/// - `F`: Infallible encoding callback.
///
/// # Parameters
///
/// - `writer`: Destination for the fixed-width payload.
/// - `value`: Value passed to `encode`.
/// - `encode`: Callback that fills the local payload buffer.
///
/// # Returns
///
/// Returns after the complete payload has been written.
///
/// # Errors
///
/// Returns an output error, including a write-zero error when the output stops
/// making progress.
///
/// # Cancellation safety
///
/// This operation is not cancellation safe. Dropping it leaves an
/// already-written prefix in `writer`.
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
