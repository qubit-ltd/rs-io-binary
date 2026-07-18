// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow source-test-pair
//! Asynchronous LEB128 reads.

use std::io::Result;

use qubit_codec_binary::{
    Leb128Codec,
    NonStrict,
    Strict,
};
use qubit_io::AsyncInput;

use crate::util::read_leb128_payload_async;

macro_rules! read_leb128_value_async {
    ($reader:expr, $ty:ty, $policy:ty) => {
        read_leb128_payload_async::<
            { Leb128Codec::<$ty, $policy>::MAX_UNITS_PER_VALUE },
            Leb128Codec<$ty, $policy>,
            _,
        >($reader)
        .await
    };
}

macro_rules! leb128_read_method {
    ($doc:literal, $name:ident, $ty:ty, $policy:ty) => {
        #[doc = $doc]
        async fn $name(&mut self) -> Result<$ty>
        where
            Self: Unpin,
        {
            read_leb128_value_async!(self, $ty, $policy)
        }
    };
}

/// Future-based LEB128 integer reads from runtime-neutral async inputs.
///
/// Target-width `usize` and `isize` formats remain platform dependent. Use
/// fixed-width methods for persistent files and cross-platform protocols.
#[allow(async_fn_in_trait)]
pub trait AsyncLeb128ReadExt: AsyncInput<Item = u8> {
    leb128_read_method!(
        "Asynchronously reads a non-strict LEB128 `u8`.",
        read_uleb_u8_async,
        u8,
        NonStrict
    );
    leb128_read_method!(
        "Asynchronously reads a strict LEB128 `u8`.",
        read_uleb_u8_strict_async,
        u8,
        Strict
    );
    leb128_read_method!(
        "Asynchronously reads a non-strict LEB128 `u16`.",
        read_uleb_u16_async,
        u16,
        NonStrict
    );
    leb128_read_method!(
        "Asynchronously reads a strict LEB128 `u16`.",
        read_uleb_u16_strict_async,
        u16,
        Strict
    );
    leb128_read_method!(
        "Asynchronously reads a non-strict LEB128 `u32`.",
        read_uleb_u32_async,
        u32,
        NonStrict
    );
    leb128_read_method!(
        "Asynchronously reads a strict LEB128 `u32`.",
        read_uleb_u32_strict_async,
        u32,
        Strict
    );
    leb128_read_method!(
        "Asynchronously reads a non-strict LEB128 `u64`.",
        read_uleb_u64_async,
        u64,
        NonStrict
    );
    leb128_read_method!(
        "Asynchronously reads a strict LEB128 `u64`.",
        read_uleb_u64_strict_async,
        u64,
        Strict
    );
    leb128_read_method!(
        "Asynchronously reads a non-strict LEB128 `u128`.",
        read_uleb_u128_async,
        u128,
        NonStrict
    );
    leb128_read_method!(
        "Asynchronously reads a strict LEB128 `u128`.",
        read_uleb_u128_strict_async,
        u128,
        Strict
    );
    leb128_read_method!(
        "Asynchronously reads a non-strict LEB128 `usize`.",
        read_uleb_usize_async,
        usize,
        NonStrict
    );
    leb128_read_method!(
        "Asynchronously reads a strict LEB128 `usize`.",
        read_uleb_usize_strict_async,
        usize,
        Strict
    );
    leb128_read_method!(
        "Asynchronously reads a non-strict LEB128 `i8`.",
        read_sleb_i8_async,
        i8,
        NonStrict
    );
    leb128_read_method!(
        "Asynchronously reads a strict LEB128 `i8`.",
        read_sleb_i8_strict_async,
        i8,
        Strict
    );
    leb128_read_method!(
        "Asynchronously reads a non-strict LEB128 `i16`.",
        read_sleb_i16_async,
        i16,
        NonStrict
    );
    leb128_read_method!(
        "Asynchronously reads a strict LEB128 `i16`.",
        read_sleb_i16_strict_async,
        i16,
        Strict
    );
    leb128_read_method!(
        "Asynchronously reads a non-strict LEB128 `i32`.",
        read_sleb_i32_async,
        i32,
        NonStrict
    );
    leb128_read_method!(
        "Asynchronously reads a strict LEB128 `i32`.",
        read_sleb_i32_strict_async,
        i32,
        Strict
    );
    leb128_read_method!(
        "Asynchronously reads a non-strict LEB128 `i64`.",
        read_sleb_i64_async,
        i64,
        NonStrict
    );
    leb128_read_method!(
        "Asynchronously reads a strict LEB128 `i64`.",
        read_sleb_i64_strict_async,
        i64,
        Strict
    );
    leb128_read_method!(
        "Asynchronously reads a non-strict LEB128 `i128`.",
        read_sleb_i128_async,
        i128,
        NonStrict
    );
    leb128_read_method!(
        "Asynchronously reads a strict LEB128 `i128`.",
        read_sleb_i128_strict_async,
        i128,
        Strict
    );
    leb128_read_method!(
        "Asynchronously reads a non-strict LEB128 `isize`.",
        read_sleb_isize_async,
        isize,
        NonStrict
    );
    leb128_read_method!(
        "Asynchronously reads a strict LEB128 `isize`.",
        read_sleb_isize_strict_async,
        isize,
        Strict
    );
}

impl<R> AsyncLeb128ReadExt for R where R: AsyncInput<Item = u8> + ?Sized {}
