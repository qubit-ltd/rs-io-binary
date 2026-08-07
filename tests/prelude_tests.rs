// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::Cursor;

use qubit_codec::BigEndian;
use qubit_codec::ByteOrder;
use qubit_codec::ByteOrderSpec;
use qubit_codec_binary::Leb128DecodePolicy;
use qubit_codec_binary::NonStrict;
use qubit_io_binary::prelude::Leb128ReadExt;
use qubit_io_binary::prelude::Leb128WriteExt;
use qubit_io_binary::prelude::ZigZagReadExt;
use qubit_io_binary::prelude::ZigZagWriteExt;

fn leb128_policy_is_strict<P: Leb128DecodePolicy>() -> bool {
    P::STRICT
}

#[test]
fn test_prelude_imports_binary_extension_traits() {
    assert_eq!(ByteOrder::BigEndian, BigEndian::ORDER);
    assert!(!leb128_policy_is_strict::<NonStrict>());

    let mut buffer = Vec::new();
    buffer
        .write_uleb_u16(300)
        .expect("Leb128WriteExt should be in prelude");
    buffer
        .write_zig_zag_i16(-42)
        .expect("ZigZagWriteExt should be in prelude");

    let mut input = Cursor::new(buffer);
    assert_eq!(
        300,
        input
            .read_uleb_u16_non_strict()
            .expect("Leb128ReadExt should be in prelude")
    );
    assert_eq!(
        -42,
        input
            .read_zig_zag_i16_non_strict()
            .expect("ZigZagReadExt should be in prelude")
    );
}
