// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::Cursor;
use std::io::ErrorKind;

use qubit_io_binary::Leb128ReadExt;
use qubit_io_binary::Leb128WriteExt;

#[test]
fn test_read_leb128_payload_round_trips_maximum_u128_width() {
    let mut bytes = Vec::new();
    bytes
        .write_uleb_u128(u128::MAX)
        .expect("maximum u128 should encode");
    assert_eq!(19, bytes.len());

    let decoded = Cursor::new(bytes)
        .read_uleb_u128_non_strict()
        .expect("maximum-width u128 should decode");

    assert_eq!(u128::MAX, decoded);
}

#[test]
fn test_read_leb128_payload_rejects_unterminated_maximum_width() {
    let mut reader = Cursor::new(vec![0x80; 19]);

    let error = reader
        .read_uleb_u128_non_strict()
        .expect_err("unterminated maximum-width payload should fail");

    assert_eq!(ErrorKind::InvalidData, error.kind());
}

#[test]
fn test_read_leb128_payload_maps_incomplete_input_to_unexpected_eof() {
    let mut reader = Cursor::new(vec![0x80]);

    let error = reader
        .read_uleb_u16_non_strict()
        .expect_err("truncated payload should fail");

    assert_eq!(ErrorKind::UnexpectedEof, error.kind());
}
