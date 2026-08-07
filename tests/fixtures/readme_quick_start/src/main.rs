// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io;
use std::io::Cursor;

use qubit_io_binary::BinaryReadExt;
use qubit_io_binary::BinaryWriteExt;
use qubit_io_binary::Leb128ReadExt;
use qubit_io_binary::Leb128WriteExt;

/// Compiles the README quick-start example with only its documented dependency.
fn main() -> io::Result<()> {
    let mut bytes = Vec::new();
    bytes.write_u32_le(0x0102_0304)?;
    bytes.write_uleb_u64(300)?;

    let mut input = Cursor::new(bytes);
    assert_eq!(0x0102_0304, input.read_u32_le()?);
    assert_eq!(300, input.read_uleb_u64_non_strict()?);
    Ok(())
}
