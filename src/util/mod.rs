/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
mod allocation;
mod streams;

pub(crate) use streams::{
    checked_u16_len,
    checked_u32_len,
    checked_u64_len,
    decode_available_leb128,
    decode_infallible_unchecked,
    encode_infallible_unchecked,
    map_leb128_decode_error,
    read_leb128_from_reader,
    read_leb128_payload,
    read_utf8_payload,
    usize_from_u64_len,
    write_utf8_payload,
    write_utf8_string_with_u16_len,
    write_utf8_string_with_u32_len,
};
