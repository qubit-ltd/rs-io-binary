// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::Cursor;
use std::io::ErrorKind;

use qubit_io_binary::ZigZagReadExt;
use qubit_io_binary::ZigZagWriteExt;

#[test]
fn test_zig_zag_read_ext_reads_all_methods() {
    let mut bytes = Vec::new();
    bytes.write_zig_zag_i8(i8::MIN).expect("i8 should be encoded");
    bytes.write_zig_zag_i16(-300).expect("i16 should be encoded");
    bytes.write_zig_zag_i32(-0x1f600).expect("i32 should be encoded");
    bytes.write_zig_zag_i64(i64::MIN).expect("i64 should be encoded");
    bytes.write_zig_zag_i128(i128::MIN).expect("i128 should be encoded");
    bytes.write_zig_zag_isize(isize::MIN).expect("isize should be encoded");

    let mut input = Cursor::new(bytes.clone());
    assert_eq!(i8::MIN, input.read_zig_zag_i8_non_strict().expect("i8 should be read"));
    assert_eq!(-300, input.read_zig_zag_i16_non_strict().expect("i16 should be read"));
    assert_eq!(
        -0x1f600,
        input.read_zig_zag_i32_non_strict().expect("i32 should be read")
    );
    assert_eq!(
        i64::MIN,
        input.read_zig_zag_i64_non_strict().expect("i64 should be read")
    );
    assert_eq!(
        i128::MIN,
        input.read_zig_zag_i128_non_strict().expect("i128 should be read")
    );
    assert_eq!(
        isize::MIN,
        input.read_zig_zag_isize_non_strict().expect("isize should be read")
    );

    let mut input = Cursor::new(bytes);
    assert_eq!(
        i8::MIN,
        input.read_zig_zag_i8_strict().expect("strict i8 should be read")
    );
    assert_eq!(
        -300,
        input.read_zig_zag_i16_strict().expect("strict i16 should be read")
    );
    assert_eq!(
        -0x1f600,
        input.read_zig_zag_i32_strict().expect("strict i32 should be read")
    );
    assert_eq!(
        i64::MIN,
        input.read_zig_zag_i64_strict().expect("strict i64 should be read")
    );
    assert_eq!(
        i128::MIN,
        input.read_zig_zag_i128_strict().expect("strict i128 should be read")
    );
    assert_eq!(
        isize::MIN,
        input.read_zig_zag_isize_strict().expect("strict isize should be read")
    );
}

#[test]
fn test_zig_zag_read_ext_reports_invalid_data_and_eof() {
    let mut input = Cursor::new([0x80, 0x00]);
    assert_eq!(
        ErrorKind::InvalidData,
        input
            .read_zig_zag_i16_strict()
            .expect_err("non-canonical value should fail")
            .kind()
    );

    let mut input = Cursor::new([0x80, 0x80]);
    assert_eq!(
        ErrorKind::InvalidData,
        input
            .read_zig_zag_i8_non_strict()
            .expect_err("unterminated max-width i8 value should fail")
            .kind()
    );

    let mut input = Cursor::new([0x80]);
    assert_eq!(
        ErrorKind::UnexpectedEof,
        input
            .read_zig_zag_i64_non_strict()
            .expect_err("truncated value should report EOF")
            .kind()
    );

    let mut input = Cursor::new([0x80, 0x80, 0x80]);
    assert_eq!(
        ErrorKind::InvalidData,
        input
            .read_zig_zag_i16_non_strict()
            .expect_err("unterminated max-width value should fail")
            .kind()
    );
}

#[test]
fn test_zig_zag_read_golden_vector() {
    macro_rules! check_read {
        ($strict:ident, $loose:ident, $value:expr, $bytes:expr) => {{
            let bytes: &[u8] = &$bytes;
            let mut strict = Cursor::new(bytes);
            assert_eq!($value, strict.$strict().expect("canonical vector must decode"));
            assert_eq!(bytes.len() as u64, strict.position());
            let mut loose = Cursor::new(bytes);
            assert_eq!($value, loose.$loose().expect("canonical vector must decode"));
            assert_eq!(bytes.len() as u64, loose.position());
        }};
    }
    check_read!(read_zig_zag_i8_strict, read_zig_zag_i8_non_strict, -1, [1]);
    check_read!(read_zig_zag_i16_strict, read_zig_zag_i16_non_strict, -1, [1]);
    check_read!(read_zig_zag_i32_strict, read_zig_zag_i32_non_strict, -1, [1]);
    check_read!(read_zig_zag_i64_strict, read_zig_zag_i64_non_strict, -1, [1]);
    check_read!(read_zig_zag_i128_strict, read_zig_zag_i128_non_strict, -1, [1]);
    check_read!(read_zig_zag_isize_strict, read_zig_zag_isize_non_strict, -1, [1]);
    check_read!(read_zig_zag_i16_strict, read_zig_zag_i16_non_strict, -300, [0xd7, 4]);
    check_read!(read_zig_zag_i32_strict, read_zig_zag_i32_non_strict, -300, [0xd7, 4]);
    check_read!(read_zig_zag_i64_strict, read_zig_zag_i64_non_strict, -300, [0xd7, 4]);
    check_read!(read_zig_zag_i128_strict, read_zig_zag_i128_non_strict, -300, [0xd7, 4]);
    check_read!(
        read_zig_zag_isize_strict,
        read_zig_zag_isize_non_strict,
        -300,
        [0xd7, 4]
    );
}

#[test]
fn test_zig_zag_varint_boundary_i128_min() {
    let mut bytes = vec![0xff; 18];
    bytes.push(0x03);
    let mut strict = Cursor::new(bytes.as_slice());
    assert_eq!(
        i128::MIN,
        strict
            .read_zig_zag_i128_strict()
            .expect("strict reader should decode the i128 minimum")
    );
    assert_eq!(19, strict.position());
    let mut loose = Cursor::new(bytes.as_slice());
    assert_eq!(
        i128::MIN,
        loose
            .read_zig_zag_i128_non_strict()
            .expect("non-strict reader should decode the i128 minimum")
    );
    assert_eq!(19, loose.position());
}
