// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow source-test-pair
//! Asynchronous ZigZag reads.

use std::io::Result;

use qubit_codec_binary::{
    NonStrict,
    Strict,
    ZigZagCodec,
};
use qubit_io::AsyncInput;

use crate::util::read_leb128_payload_async;

macro_rules! zig_zag_read_method {
    ($doc:literal, $name:ident, $ty:ty, $policy:ty) => {
        #[doc = $doc]
        async fn $name(&mut self) -> Result<$ty>
        where
            Self: Unpin,
        {
            read_leb128_payload_async::<
                { ZigZagCodec::<$ty, $policy>::MAX_UNITS_PER_VALUE },
                ZigZagCodec<$ty, $policy>,
                _,
            >(self)
            .await
        }
    };
}

/// Future-based ZigZag plus unsigned-LEB128 reads.
#[allow(async_fn_in_trait)]
pub trait AsyncZigZagReadExt: AsyncInput<Item = u8> {
    zig_zag_read_method!(
        "Asynchronously reads non-strict ZigZag `i8`.",
        read_zig_zag_i8_async,
        i8,
        NonStrict
    );
    zig_zag_read_method!(
        "Asynchronously reads strict ZigZag `i8`.",
        read_zig_zag_i8_strict_async,
        i8,
        Strict
    );
    zig_zag_read_method!(
        "Asynchronously reads non-strict ZigZag `i16`.",
        read_zig_zag_i16_async,
        i16,
        NonStrict
    );
    zig_zag_read_method!(
        "Asynchronously reads strict ZigZag `i16`.",
        read_zig_zag_i16_strict_async,
        i16,
        Strict
    );
    zig_zag_read_method!(
        "Asynchronously reads non-strict ZigZag `i32`.",
        read_zig_zag_i32_async,
        i32,
        NonStrict
    );
    zig_zag_read_method!(
        "Asynchronously reads strict ZigZag `i32`.",
        read_zig_zag_i32_strict_async,
        i32,
        Strict
    );
    zig_zag_read_method!(
        "Asynchronously reads non-strict ZigZag `i64`.",
        read_zig_zag_i64_async,
        i64,
        NonStrict
    );
    zig_zag_read_method!(
        "Asynchronously reads strict ZigZag `i64`.",
        read_zig_zag_i64_strict_async,
        i64,
        Strict
    );
    zig_zag_read_method!(
        "Asynchronously reads non-strict ZigZag `i128`.",
        read_zig_zag_i128_async,
        i128,
        NonStrict
    );
    zig_zag_read_method!(
        "Asynchronously reads strict ZigZag `i128`.",
        read_zig_zag_i128_strict_async,
        i128,
        Strict
    );
    zig_zag_read_method!(
        "Asynchronously reads non-strict ZigZag `isize`.",
        read_zig_zag_isize_async,
        isize,
        NonStrict
    );
    zig_zag_read_method!(
        "Asynchronously reads strict ZigZag `isize`.",
        read_zig_zag_isize_strict_async,
        isize,
        Strict
    );
}

impl<R> AsyncZigZagReadExt for R where R: AsyncInput<Item = u8> + ?Sized {}
