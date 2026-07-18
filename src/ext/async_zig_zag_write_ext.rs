// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Asynchronous ZigZag writes.

use std::io::Result;

use qubit_codec_binary::{
    NonStrict,
    ZigZagCodec,
};
use qubit_io::AsyncOutput;

use crate::util::{
    encode_infallible_unchecked,
    write_all_async,
};

macro_rules! zig_zag_write_method {
    ($doc:literal, $name:ident, $ty:ty) => {
        #[doc = $doc]
        async fn $name(&mut self, value: $ty) -> Result<()>
        where
            Self: Unpin,
        {
            let mut bytes =
                [0_u8; ZigZagCodec::<$ty, NonStrict>::MAX_UNITS_PER_VALUE];
            type Codec = ZigZagCodec<$ty, NonStrict>;
            // SAFETY: The local buffer has the codec's maximum payload size.
            let len = unsafe {
                encode_infallible_unchecked::<Codec>(value, &mut bytes, 0)
            };
            write_all_async(self, &bytes[..len]).await
        }
    };
}

/// Future-based ZigZag plus unsigned-LEB128 writes.
#[allow(async_fn_in_trait)]
pub trait AsyncZigZagWriteExt: AsyncOutput<Item = u8> {
    zig_zag_write_method!(
        "Asynchronously writes ZigZag `i8`.",
        write_zig_zag_i8_async,
        i8
    );
    zig_zag_write_method!(
        "Asynchronously writes ZigZag `i16`.",
        write_zig_zag_i16_async,
        i16
    );
    zig_zag_write_method!(
        "Asynchronously writes ZigZag `i32`.",
        write_zig_zag_i32_async,
        i32
    );
    zig_zag_write_method!(
        "Asynchronously writes ZigZag `i64`.",
        write_zig_zag_i64_async,
        i64
    );
    zig_zag_write_method!(
        "Asynchronously writes ZigZag `i128`.",
        write_zig_zag_i128_async,
        i128
    );
    zig_zag_write_method!(
        "Asynchronously writes ZigZag `isize`.",
        write_zig_zag_isize_async,
        isize
    );
}

impl<W> AsyncZigZagWriteExt for W where W: AsyncOutput<Item = u8> + ?Sized {}
