// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::cell::RefCell;
use std::io::{
    Cursor,
    Result,
    SeekFrom,
};
use std::rc::Rc;

use qubit_codec::BigEndian;
use qubit_codec_binary::NonStrict;
use qubit_io::Output;
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

#[derive(Clone)]
struct RecordingOutput {
    bytes: Rc<RefCell<Vec<u8>>>,
}

impl RecordingOutput {
    fn new(bytes: Rc<RefCell<Vec<u8>>>) -> Self {
        Self { bytes }
    }
}

impl Output for RecordingOutput {
    type Item = u8;

    unsafe fn write_unchecked(
        &mut self,
        input: &[u8],
        index: usize,
        count: usize,
    ) -> Result<usize> {
        self.bytes
            .borrow_mut()
            .extend_from_slice(&input[index..index + count]);
        Ok(count)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

macro_rules! exercise_input_methods {
    ($constructor:expr, $buffered:expr) => {{
        let mut reader = $constructor(vec![0x10, 0x20, 0x30]);
        assert_eq!($buffered, reader.is_buffered());

        let mut output = [0; 5];
        assert_eq!(
            2,
            unsafe { reader.read_unchecked(&mut output, 1, 2) }
                .expect("unchecked read should succeed")
        );
        assert_eq!([0, 0x10, 0x20, 0, 0], output);

        let mut reader = $constructor(vec![0x10, 0x20, 0x30]);
        let mut output = [0; 2];
        assert_eq!(2, reader.read(&mut output).expect("read should succeed"));
        assert_eq!([0x10, 0x20], output);

        let mut reader = $constructor(vec![0x10, 0x20, 0x30]);
        let mut output = [0; 5];
        assert_eq!(
            2,
            unsafe { reader.read_fully_unchecked(&mut output, 1, 2) }
                .expect("unchecked full read should succeed")
        );
        assert_eq!([0, 0x10, 0x20, 0, 0], output);

        let mut reader = $constructor(vec![0x10, 0x20, 0x30]);
        let mut output = [0; 3];
        assert_eq!(
            3,
            reader
                .read_fully(&mut output)
                .expect("full read should succeed")
        );
        assert_eq!([0x10, 0x20, 0x30], output);

        let mut reader = $constructor(vec![0x10, 0x20]);
        let mut output = [0; 2];
        reader
            .read_exactly(&mut output)
            .expect("exact read should succeed");
        assert_eq!([0x10, 0x20], output);
    }};
}

macro_rules! exercise_output_methods {
    ($constructor:expr, $buffered:expr) => {{
        let bytes = Rc::new(RefCell::new(Vec::new()));
        let mut writer = $constructor(RecordingOutput::new(bytes.clone()));
        assert_eq!($buffered, writer.is_buffered());

        let input = [0x10, 0x20, 0x30];
        assert_eq!(
            2,
            unsafe { writer.write_unchecked(&input, 1, 2) }
                .expect("unchecked write should succeed")
        );
        assert_eq!(
            2,
            writer.write(&[0x40, 0x50]).expect("write should succeed")
        );

        let input = [0x60, 0x70, 0x80];
        unsafe {
            writer
                .write_fully_unchecked(&input, 1, 2)
                .expect("unchecked full write should succeed");
        }
        writer
            .write_fully(&[0x90, 0xa0])
            .expect("full write should succeed");
        writer.flush().expect("flush should succeed");

        assert_eq!(
            [0x20, 0x30, 0x40, 0x50, 0x70, 0x80, 0x90, 0xa0],
            bytes.borrow().as_slice()
        );
    }};
}

macro_rules! exercise_seek_method {
    ($constructor:expr) => {{
        let mut stream = $constructor;
        assert_eq!(0, stream.seek_to(SeekFrom::Start(0)).unwrap());
        assert_eq!(1, stream.seek_to(SeekFrom::Current(1)).unwrap());
        assert_eq!(0, stream.seek_to(SeekFrom::Start(0)).unwrap());
    }};
}

#[test]
fn reader_forwarding_methods_delegate_for_every_reader_wrapper() {
    exercise_input_methods!(
        |bytes| BinaryReader::<_, BigEndian>::new(Cursor::new(bytes)),
        false
    );
    exercise_input_methods!(
        |bytes| BufferedBinaryReader::<_, BigEndian>::new(Cursor::new(bytes)),
        true
    );
    exercise_input_methods!(
        |bytes| Leb128Reader::<_, NonStrict>::new(Cursor::new(bytes)),
        false
    );
    exercise_input_methods!(
        |bytes| BufferedLeb128Reader::<_, NonStrict>::new(Cursor::new(bytes)),
        true
    );
    exercise_input_methods!(
        |bytes| ZigZagReader::<_, NonStrict>::new(Cursor::new(bytes)),
        false
    );
    exercise_input_methods!(
        |bytes| BufferedZigZagReader::<_, NonStrict>::new(Cursor::new(bytes)),
        true
    );
}

#[test]
fn writer_forwarding_methods_delegate_for_every_writer_wrapper() {
    exercise_output_methods!(BinaryWriter::<_, BigEndian>::new, false);
    exercise_output_methods!(BufferedBinaryWriter::<_, BigEndian>::new, true);
    exercise_output_methods!(Leb128Writer::new, false);
    exercise_output_methods!(BufferedLeb128Writer::new, true);
    exercise_output_methods!(ZigZagWriter::new, false);
    exercise_output_methods!(BufferedZigZagWriter::new, true);
}

#[test]
fn seek_forwarding_methods_delegate_for_every_seekable_wrapper() {
    exercise_seek_method!(BinaryReader::<_, BigEndian>::new(Cursor::new(
        vec![0, 1, 2,]
    )));
    exercise_seek_method!(BufferedBinaryReader::<_, BigEndian>::new(
        Cursor::new(vec![0, 1, 2])
    ));
    exercise_seek_method!(Leb128Reader::<_, NonStrict>::new(Cursor::new(
        vec![0, 1, 2,]
    )));
    exercise_seek_method!(BufferedLeb128Reader::<_, NonStrict>::new(
        Cursor::new(vec![0, 1, 2])
    ));
    exercise_seek_method!(ZigZagReader::<_, NonStrict>::new(Cursor::new(
        vec![0, 1, 2,]
    )));
    exercise_seek_method!(BufferedZigZagReader::<_, NonStrict>::new(
        Cursor::new(vec![0, 1, 2])
    ));
    exercise_seek_method!(BinaryWriter::<_, BigEndian>::new(Cursor::new(
        Vec::new(),
    )));
    exercise_seek_method!(BufferedBinaryWriter::<_, BigEndian>::new(
        Cursor::new(Vec::new())
    ));
    exercise_seek_method!(Leb128Writer::new(Cursor::new(Vec::new())));
    exercise_seek_method!(BufferedLeb128Writer::new(Cursor::new(Vec::new())));
    exercise_seek_method!(ZigZagWriter::new(Cursor::new(Vec::new())));
    exercise_seek_method!(BufferedZigZagWriter::new(Cursor::new(Vec::new())));
}
