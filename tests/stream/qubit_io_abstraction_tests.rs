// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::Result;

use qubit_codec::BigEndian;
use qubit_codec_binary::NonStrict;
use qubit_io::{
    Input,
    Output,
};
use qubit_io_binary::{
    BinaryReader,
    BinaryWriter,
    BufferedBinaryReader,
    BufferedBinaryWriter,
    BufferedLeb128Reader,
    BufferedLeb128Writer,
    BufferedZigZagReader,
    BufferedZigZagWriter,
    Leb128Reader,
    Leb128Writer,
    ZigZagReader,
    ZigZagWriter,
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
        let read = count.min(self.bytes.len().saturating_sub(self.position));
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
fn binary_reader_accepts_input_without_std_read() {
    let mut reader =
        BinaryReader::<_, BigEndian>::new(QubitInput::new(vec![0x12, 0x34]));

    assert!(!Input::is_buffered(&reader));
    assert_eq!(0x1234, reader.read_u16().unwrap());

    let mut reader = BinaryReader::<_, BigEndian>::new(QubitInput::new(vec![
        0, 1, b'a', 0, 0, 0, 1, b'b',
    ]));
    assert_eq!("a", reader.read_string_with_u16_len(1).unwrap());
    assert_eq!("b", reader.read_string_with_u32_len(1).unwrap());
}

#[test]
fn binary_writer_accepts_output_without_std_write() {
    let mut writer = BinaryWriter::<_, BigEndian>::new(QubitOutput::default());

    assert!(!Output::is_buffered(&writer));
    writer.write_u16(0x1234).unwrap();
    writer.write_string_with_u16_len("a").unwrap();
    writer.write_string_with_u32_len("b").unwrap();

    assert_eq!(
        vec![0x12, 0x34, 0, 1, b'a', 0, 0, 0, 1, b'b'],
        writer.into_inner().bytes,
    );
}

#[test]
fn buffered_binary_reader_accepts_input_without_std_read() {
    let mut reader =
        BufferedBinaryReader::<_, BigEndian>::new(QubitInput::new(vec![
            0x12, 0x34,
        ]));

    assert!(Input::is_buffered(&reader));
    assert_eq!(0x1234, reader.read_u16().unwrap());
}

#[test]
fn buffered_binary_writer_accepts_output_without_std_write() {
    let mut writer =
        BufferedBinaryWriter::<_, BigEndian>::new(QubitOutput::default());

    assert!(Output::is_buffered(&writer));
    writer.write_u16(0x1234).unwrap();
    Output::flush(&mut writer).unwrap();

    assert_eq!(vec![0x12, 0x34], writer.inner().bytes);
}

#[test]
fn buffered_wrappers_support_fallible_capacity_allocation() {
    BufferedBinaryReader::<_, BigEndian>::try_with_capacity(
        QubitInput::new(vec![]),
        1,
    )
    .expect("binary reader buffer should allocate");
    BufferedBinaryWriter::<_, BigEndian>::try_with_capacity(
        QubitOutput::default(),
        1,
    )
    .expect("binary writer buffer should allocate");
    BufferedLeb128Reader::<_, NonStrict>::try_with_capacity(
        QubitInput::new(vec![]),
        1,
    )
    .expect("LEB128 reader buffer should allocate");
    BufferedLeb128Writer::try_with_capacity(QubitOutput::default(), 1)
        .expect("LEB128 writer buffer should allocate");
    BufferedZigZagReader::<_, NonStrict>::try_with_capacity(
        QubitInput::new(vec![]),
        1,
    )
    .expect("ZigZag reader buffer should allocate");
    BufferedZigZagWriter::try_with_capacity(QubitOutput::default(), 1)
        .expect("ZigZag writer buffer should allocate");
}

#[test]
fn leb128_readers_accept_input_without_std_read() {
    let mut reader =
        Leb128Reader::<_, NonStrict>::new(QubitInput::new(vec![0xac, 0x02]));
    assert!(!Input::is_buffered(&reader));
    assert_eq!(300, reader.read_u16().unwrap());

    let mut reader =
        BufferedLeb128Reader::<_, NonStrict>::new(QubitInput::new(vec![
            0xac, 0x02,
        ]));
    assert!(Input::is_buffered(&reader));
    assert_eq!(300, reader.read_u16().unwrap());
}

#[test]
fn leb128_writers_accept_output_without_std_write() {
    let mut writer = Leb128Writer::new(QubitOutput::default());
    assert!(!Output::is_buffered(&writer));
    writer.write_u16(300).unwrap();
    writer.write_utf8_string_usize("a").unwrap();
    writer.write_utf8_string_u64("b").unwrap();
    assert_eq!(
        vec![0xac, 0x02, 1, b'a', 1, b'b'],
        writer.into_inner().bytes
    );

    let mut writer = BufferedLeb128Writer::new(QubitOutput::default());
    assert!(Output::is_buffered(&writer));
    writer.write_u16(300).unwrap();
    Output::flush(&mut writer).unwrap();
    assert_eq!(vec![0xac, 0x02], writer.inner().bytes);
}

#[test]
fn zig_zag_readers_accept_input_without_std_read() {
    let mut reader =
        ZigZagReader::<_, NonStrict>::new(QubitInput::new(vec![0x01]));
    assert!(!Input::is_buffered(&reader));
    assert_eq!(-1, reader.read_i16().unwrap());

    let mut reader =
        BufferedZigZagReader::<_, NonStrict>::new(QubitInput::new(vec![0x01]));
    assert!(Input::is_buffered(&reader));
    assert_eq!(-1, reader.read_i16().unwrap());
}

#[test]
fn zig_zag_writers_accept_output_without_std_write() {
    let mut writer = ZigZagWriter::new(QubitOutput::default());
    assert!(!Output::is_buffered(&writer));
    writer.write_i16(-1).unwrap();
    assert_eq!(vec![0x01], writer.into_inner().bytes);

    let mut writer = BufferedZigZagWriter::new(QubitOutput::default());
    assert!(Output::is_buffered(&writer));
    writer.write_i16(-1).unwrap();
    Output::flush(&mut writer).unwrap();
    assert_eq!(vec![0x01], writer.inner().bytes);
}
