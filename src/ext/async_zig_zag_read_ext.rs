// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Asynchronous ZigZag reads.

use core::future::Future;
use std::io::Result;

use qubit_codec_binary::NonStrict;
use qubit_codec_binary::Strict;
use qubit_codec_binary::ZigZagCodec;
use qubit_io::AsyncInput;

use crate::util::read_leb128_payload_async;

macro_rules! zig_zag_read_method {
    ($doc:literal, $name:ident, $ty:ty, $policy:ty) => {
        #[doc = $doc]
        #[doc = ""]
        #[doc = "# Returns"]
        #[doc = ""]
        #[doc = concat!("Returns the decoded `", stringify!($ty), "`.")]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns an input error or an invalid-data error when the \
                 payload is malformed or violates the selected policy."]
        #[doc = ""]
        #[doc = "# Cancellation safety"]
        #[doc = ""]
        #[doc = "This operation is not cancellation safe. Dropping the future \
                 retains any bytes already consumed from the input."]
        #[inline(always)]
        fn $name(&mut self) -> impl Future<Output = Result<$ty>> + '_
        where
            Self: Unpin,
        {
            async move {
                read_leb128_payload_async::<
                    { ZigZagCodec::<$ty, $policy>::MAX_DECODE_UNITS_PER_VALUE },
                    ZigZagCodec<$ty, $policy>,
                    _,
                >(self)
                .await
            }
        }
    };
}

/// Future-based ZigZag plus unsigned-LEB128 reads.
///
/// # Cancellation safety
///
/// These reads are not cancellation safe. Dropping a pending future leaves
/// bytes already consumed from the input consumed.
pub trait AsyncZigZagReadExt: AsyncInput<Item = u8> {
    zig_zag_read_method!(
        "Asynchronously reads non-strict ZigZag `i8`.",
        read_zig_zag_i8_non_strict_async,
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
        read_zig_zag_i16_non_strict_async,
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
        read_zig_zag_i32_non_strict_async,
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
        read_zig_zag_i64_non_strict_async,
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
        read_zig_zag_i128_non_strict_async,
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
        read_zig_zag_isize_non_strict_async,
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
