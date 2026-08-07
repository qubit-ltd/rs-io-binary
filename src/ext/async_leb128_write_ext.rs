// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Asynchronous LEB128 writes.

use core::future::Future;
use std::io::Result;

use qubit_codec_binary::Leb128Codec;
use qubit_codec_binary::NonStrict;
use qubit_io::AsyncOutput;

use crate::util::encode_infallible_unchecked;
use crate::util::write_all_async;

macro_rules! leb128_write_method {
    ($doc:literal, $name:ident, $ty:ty) => {
        #[doc = $doc]
        #[doc = ""]
        #[doc = "# Parameters"]
        #[doc = ""]
        #[doc = "- `value`: Integer to encode and write."]
        #[doc = ""]
        #[doc = "# Returns"]
        #[doc = ""]
        #[doc = "Returns a future that completes after the canonical payload \
                 has been written."]
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
            async move {
                let mut bytes = [0_u8;
                    Leb128Codec::<$ty, NonStrict>::MAX_ENCODE_UNITS_PER_VALUE];
                type Codec = Leb128Codec<$ty, NonStrict>;
                // SAFETY: The local buffer has the codec's maximum payload
                // size.
                let len = unsafe {
                    encode_infallible_unchecked::<Codec>(value, &mut bytes, 0)
                };
                write_all_async(self, &bytes[..len]).await
            }
        }
    };
}

/// Future-based canonical LEB128 writes to runtime-neutral async outputs.
///
/// # Cancellation safety
///
/// These writes are not cancellation safe. Dropping a pending future leaves
/// an already-written prefix in the output.
pub trait AsyncLeb128WriteExt: AsyncOutput<Item = u8> {
    leb128_write_method!(
        "Asynchronously writes LEB128 `u8`.",
        write_uleb_u8_async,
        u8
    );
    leb128_write_method!(
        "Asynchronously writes LEB128 `u16`.",
        write_uleb_u16_async,
        u16
    );
    leb128_write_method!(
        "Asynchronously writes LEB128 `u32`.",
        write_uleb_u32_async,
        u32
    );
    leb128_write_method!(
        "Asynchronously writes LEB128 `u64`.",
        write_uleb_u64_async,
        u64
    );
    leb128_write_method!(
        "Asynchronously writes LEB128 `u128`.",
        write_uleb_u128_async,
        u128
    );
    leb128_write_method!(
        "Asynchronously writes LEB128 `usize`.",
        write_uleb_usize_async,
        usize
    );
    leb128_write_method!(
        "Asynchronously writes LEB128 `i8`.",
        write_sleb_i8_async,
        i8
    );
    leb128_write_method!(
        "Asynchronously writes LEB128 `i16`.",
        write_sleb_i16_async,
        i16
    );
    leb128_write_method!(
        "Asynchronously writes LEB128 `i32`.",
        write_sleb_i32_async,
        i32
    );
    leb128_write_method!(
        "Asynchronously writes LEB128 `i64`.",
        write_sleb_i64_async,
        i64
    );
    leb128_write_method!(
        "Asynchronously writes LEB128 `i128`.",
        write_sleb_i128_async,
        i128
    );
    leb128_write_method!(
        "Asynchronously writes LEB128 `isize`.",
        write_sleb_isize_async,
        isize
    );
}

impl<W> AsyncLeb128WriteExt for W where W: AsyncOutput<Item = u8> + ?Sized {}
