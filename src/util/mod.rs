// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Private stream and allocation helpers shared by extension traits.

mod async_streams;
mod streams;
pub(crate) use qubit_utils::try_reserve_vec;

pub(crate) use self::async_streams::read_exactly_async;
pub(crate) use self::async_streams::read_leb128_payload_async;
pub(crate) use self::async_streams::read_utf8_payload_async;
pub(crate) use self::async_streams::read_utf8_payload_into_async;
pub(crate) use self::async_streams::write_all_async;
pub(crate) use self::streams::MIN_CODEC_BUFFER_CAPACITY;
pub(crate) use self::streams::checked_u16_len;
pub(crate) use self::streams::checked_u32_len;
pub(crate) use self::streams::checked_u64_len;
pub(crate) use self::streams::decode_infallible_unchecked;
pub(crate) use self::streams::encode_infallible_unchecked;
pub(crate) use self::streams::read_leb128_from_reader;
pub(crate) use self::streams::read_leb128_payload;
pub(crate) use self::streams::read_utf8_payload;
pub(crate) use self::streams::read_utf8_payload_into;
#[cfg(not(any(
    target_pointer_width = "32",
    target_pointer_width = "64"
)))]
pub(crate) use self::streams::usize_from_u32_len;
#[cfg(not(target_pointer_width = "64"))]
pub(crate) use self::streams::usize_from_u64_len;
pub(crate) use self::streams::write_all;
pub(crate) use self::streams::write_utf8_payload;
pub(crate) use self::streams::write_utf8_string_with_u16_len;
pub(crate) use self::streams::write_utf8_string_with_u32_len;
