// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::Cursor;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Write;

use qubit_io_binary::Leb128ReadExt;
use qubit_io_binary::Leb128WriteExt;

struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buffer: &[u8]) -> std::io::Result<usize> {
        Err(Error::other("write failed"))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_leb128_write_ext_writes_all_methods() {
    let mut output = Vec::new();

    output.write_uleb_u8(u8::MAX).expect("u8 should be written");
    output.write_uleb_u16(300).expect("u16 should be written");
    output.write_uleb_u32(0x1f600).expect("u32 should be written");
    output
        .write_uleb_u64(0x0102_0304_0506_0708)
        .expect("u64 should be written");
    output
        .write_uleb_u128(0x0102_0304_0506_0708_1112_1314_1516_1718)
        .expect("u128 should be written");
    output.write_uleb_usize(usize::MAX).expect("usize should be written");
    output.write_sleb_i8(i8::MIN).expect("i8 should be written");
    output.write_sleb_i16(-300).expect("i16 should be written");
    output.write_sleb_i32(-0x1f600).expect("i32 should be written");
    output
        .write_sleb_i64(-0x0102_0304_0506_0708)
        .expect("i64 should be written");
    output
        .write_sleb_i128(-0x0102_0304_0506_0708_1112_1314_1516_1718)
        .expect("i128 should be written");
    output.write_sleb_isize(isize::MIN).expect("isize should be written");

    let mut input = Cursor::new(output);
    assert_eq!(u8::MAX, input.read_uleb_u8_strict().expect("u8 should be readable"));
    assert_eq!(300, input.read_uleb_u16_strict().expect("u16 should be readable"));
    assert_eq!(0x1f600, input.read_uleb_u32_strict().expect("u32 should be readable"));
    assert_eq!(
        0x0102_0304_0506_0708,
        input.read_uleb_u64_strict().expect("u64 should be readable")
    );
    assert_eq!(
        0x0102_0304_0506_0708_1112_1314_1516_1718,
        input.read_uleb_u128_strict().expect("u128 should be readable")
    );
    assert_eq!(
        usize::MAX,
        input.read_uleb_usize_strict().expect("usize should be readable")
    );
    assert_eq!(i8::MIN, input.read_sleb_i8_strict().expect("i8 should be readable"));
    assert_eq!(-300, input.read_sleb_i16_strict().expect("i16 should be readable"));
    assert_eq!(-0x1f600, input.read_sleb_i32_strict().expect("i32 should be readable"));
    assert_eq!(
        -0x0102_0304_0506_0708,
        input.read_sleb_i64_strict().expect("i64 should be readable")
    );
    assert_eq!(
        -0x0102_0304_0506_0708_1112_1314_1516_1718,
        input.read_sleb_i128_strict().expect("i128 should be readable")
    );
    assert_eq!(
        isize::MIN,
        input.read_sleb_isize_strict().expect("isize should be readable")
    );
    assert_eq!(input.position(), input.get_ref().len() as u64);
}

#[test]
fn test_leb128_write_ext_returns_writer_error() {
    let mut writer = FailingWriter;

    let error = writer.write_uleb_u16(300).expect_err("writer error should be returned");

    assert_eq!(ErrorKind::Other, error.kind());
}
