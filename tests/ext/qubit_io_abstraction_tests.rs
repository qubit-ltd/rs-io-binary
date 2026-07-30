// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::Result;

use qubit_io::{Input, Output};
use qubit_io_binary::{
    BinaryReadExt, BinaryWriteExt, Leb128ReadExt, Leb128WriteExt, StringReadExt, StringWriteExt,
    ZigZagReadExt, ZigZagWriteExt,
};

struct QubitInput {
    bytes: Vec<u8>,
    position: usize,
}

impl QubitInput {
    fn new(bytes: Vec<u8>) -> Self {
        Self { bytes, position: 0 }
    }
}

impl Input for QubitInput {
    type Item = u8;

    unsafe fn read_unchecked(
        &mut self,
        output: &mut [u8],
        index: usize,
        count: usize,
    ) -> Result<usize> {
        let available = self.bytes.len().saturating_sub(self.position);
        let read = available.min(count);
        output[index..index + read]
            .copy_from_slice(&self.bytes[self.position..self.position + read]);
        self.position += read;
        Ok(read)
    }
}

#[derive(Default)]
struct QubitOutput {
    bytes: Vec<u8>,
}

impl Output for QubitOutput {
    type Item = u8;

    unsafe fn write_unchecked(
        &mut self,
        input: &[u8],
        index: usize,
        count: usize,
    ) -> Result<usize> {
        self.bytes.extend_from_slice(&input[index..index + count]);
        Ok(count)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

#[test]
fn binary_read_ext_accepts_input_without_std_read() {
    let mut input = QubitInput::new(vec![0x12, 0x34, 0x78, 0x56]);

    assert_eq!(0x1234, input.read_u16_be().unwrap());
    assert_eq!(0x5678, input.read_u16_le().unwrap());
}

#[test]
fn binary_write_ext_accepts_output_without_std_write() {
    let mut output = QubitOutput::default();

    output.write_u16_be(0x1234).unwrap();
    output.write_u16_le(0x5678).unwrap();

    assert_eq!(vec![0x12, 0x34, 0x78, 0x56], output.bytes);
}

#[test]
fn variable_length_read_extensions_accept_input_without_std_read() {
    let mut input = QubitInput::new(vec![0xac, 0x02, 0x01, 0x02, b'h', b'i']);

    assert_eq!(300, input.read_uleb_u16().unwrap());
    assert_eq!(-1, input.read_zig_zag_i16().unwrap());
    assert_eq!("hi", input.read_utf8_string_uleb(2).unwrap());
}

#[test]
fn variable_length_write_extensions_accept_output_without_std_write() {
    let mut output = QubitOutput::default();

    output.write_uleb_u16(300).unwrap();
    output.write_zig_zag_i16(-1).unwrap();
    output.write_utf8_string_uleb("hi").unwrap();

    assert_eq!(vec![0xac, 0x02, 0x01, 0x02, b'h', b'i'], output.bytes);
}
