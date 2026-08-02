// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::{
    Cursor,
    ErrorKind,
};

use qubit_codec_binary::NonStrict;
use qubit_io_binary::BufferedLeb128Reader;

#[test]
fn test_stream_codec_decode_error_maps_incomplete_to_unexpected_eof() {
    let mut reader =
        BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![0x80]));

    let error = reader
        .read_u64_non_strict()
        .expect_err("truncated LEB128 input should fail");

    assert_eq!(ErrorKind::UnexpectedEof, error.kind());
}

#[test]
fn test_stream_codec_decode_error_maps_invalid_to_invalid_data() {
    let mut reader =
        BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(vec![
            0x80, 0x80, 0x80,
        ]));

    let error = reader
        .read_u16_non_strict()
        .expect_err("unterminated maximum-width input should fail");

    assert_eq!(ErrorKind::InvalidData, error.kind());
}
