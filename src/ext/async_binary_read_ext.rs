// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Asynchronous fixed-width binary reads.

use core::future::Future;
use std::io::Result;

use qubit_codec::{BigEndian, ByteOrder, LittleEndian};
use qubit_codec_binary::BinaryCodec;
use qubit_io::AsyncInput;

use crate::util::{decode_infallible_unchecked, read_exactly_async};

macro_rules! read_binary_value_async {
    ($reader:expr, $ty:ty, $order:ty) => {
        read_binary_async::<{ BinaryCodec::<$ty, $order>::MIN_UNITS_PER_VALUE }, _, _, _>(
            $reader,
            |bytes| {
                type Codec = BinaryCodec<$ty, $order>;
                // SAFETY: The local buffer has exactly the codec's fixed width.
                unsafe { decode_infallible_unchecked::<Codec>(bytes, 0) }
            },
        )
        .await
    };
}

macro_rules! fixed_read_method {
    ($doc:literal, $name:ident, $ty:ty, $order:ty) => {
        #[doc = $doc]
        #[doc = ""]
        #[doc = "# Returns"]
        #[doc = ""]
        #[doc = concat!("Returns the decoded `", stringify!($ty), "`.")]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns an input error, including an unexpected-end-of-input \
                 error when a complete value is unavailable."]
        #[doc = ""]
        #[doc = "# Cancellation safety"]
        #[doc = ""]
        #[doc = "This operation is not cancellation safe. Dropping the future \
                 retains any bytes already consumed from the input."]
        #[inline(always)]
        fn $name(&mut self) -> impl Future<Output = Result<$ty>> + Send + '_
        where
            Self: Send + Unpin,
        {
            async move { read_binary_value_async!(self, $ty, $order) }
        }
    };
}

macro_rules! runtime_order_read_method {
    ($doc:literal, $name:ident, $ty:ty, $big:ident, $little:ident) => {
        #[doc = $doc]
        #[doc = ""]
        #[doc = "# Parameters"]
        #[doc = ""]
        #[doc = "- `byte_order`: Byte order used to decode the value."]
        #[doc = ""]
        #[doc = "# Returns"]
        #[doc = ""]
        #[doc = concat!("Returns the decoded `", stringify!($ty), "`.")]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns an input error, including an unexpected-end-of-input \
                 error when a complete value is unavailable."]
        #[doc = ""]
        #[doc = "# Cancellation safety"]
        #[doc = ""]
        #[doc = "This operation is not cancellation safe. Dropping the future \
                 retains any bytes already consumed from the input."]
        #[inline]
        fn $name(&mut self, byte_order: ByteOrder) -> impl Future<Output = Result<$ty>> + Send + '_
        where
            Self: Send + Unpin,
        {
            async move {
                if use_big_endian(byte_order) {
                    self.$big().await
                } else {
                    self.$little().await
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

/// Future-based fixed-width binary reads from runtime-neutral async inputs.
///
/// Every method returns a [`Send`] future when the input itself is [`Send`].
///
/// # Cancellation safety
///
/// These reads are not cancellation safe. Dropping a pending future leaves
/// bytes already consumed from the input consumed; retrying starts at the
/// input's new position.
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

    runtime_order_read_method!(
        "Asynchronously reads a `u16` using a runtime byte order.",
        read_u16_async,
        u16,
        read_u16_be_async,
        read_u16_le_async
    );

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

    runtime_order_read_method!(
        "Asynchronously reads a `u32` using a runtime byte order.",
        read_u32_async,
        u32,
        read_u32_be_async,
        read_u32_le_async
    );

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

    runtime_order_read_method!(
        "Asynchronously reads a `u64` using a runtime byte order.",
        read_u64_async,
        u64,
        read_u64_be_async,
        read_u64_le_async
    );

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

    runtime_order_read_method!(
        "Asynchronously reads a `u128` using a runtime byte order.",
        read_u128_async,
        u128,
        read_u128_be_async,
        read_u128_le_async
    );

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

    runtime_order_read_method!(
        "Asynchronously reads an `i16` using a runtime byte order.",
        read_i16_async,
        i16,
        read_i16_be_async,
        read_i16_le_async
    );

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

    runtime_order_read_method!(
        "Asynchronously reads an `i32` using a runtime byte order.",
        read_i32_async,
        i32,
        read_i32_be_async,
        read_i32_le_async
    );

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

    runtime_order_read_method!(
        "Asynchronously reads an `i64` using a runtime byte order.",
        read_i64_async,
        i64,
        read_i64_be_async,
        read_i64_le_async
    );

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

    runtime_order_read_method!(
        "Asynchronously reads an `i128` using a runtime byte order.",
        read_i128_async,
        i128,
        read_i128_be_async,
        read_i128_le_async
    );

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

    runtime_order_read_method!(
        "Asynchronously reads an `f32` using a runtime byte order.",
        read_f32_async,
        f32,
        read_f32_be_async,
        read_f32_le_async
    );

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

    runtime_order_read_method!(
        "Asynchronously reads an `f64` using a runtime byte order.",
        read_f64_async,
        f64,
        read_f64_be_async,
        read_f64_le_async
    );

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

/// Reads and decodes one fixed-width value asynchronously.
///
/// # Type Parameters
///
/// - `N`: Encoded scalar width in bytes.
/// - `T`: Decoded value type.
/// - `R`: Source asynchronous byte input.
/// - `F`: Infallible decoding callback.
///
/// # Parameters
///
/// - `reader`: Source from which the fixed-width payload is read.
/// - `decode`: Callback that decodes the initialized local buffer.
///
/// # Returns
///
/// Returns the decoded value.
///
/// # Errors
///
/// Returns an input error, including an unexpected-end-of-input error when the
/// payload is truncated.
///
/// # Cancellation safety
///
/// This operation is not cancellation safe. Dropping it retains bytes already
/// consumed from `reader`.
async fn read_binary_async<const N: usize, T, R, F>(reader: &mut R, decode: F) -> Result<T>
where
    R: AsyncInput<Item = u8> + Unpin + ?Sized,
    F: FnOnce(&[u8]) -> T,
{
    let mut bytes = [0_u8; N];
    read_exactly_async(reader, &mut bytes).await?;
    Ok(decode(&bytes))
}
