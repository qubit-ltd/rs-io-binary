use std::io::{Cursor, ErrorKind, Read, Seek, SeekFrom, Write};
use std::num::NonZeroUsize;

#[path = "../../src/stream/transcode_decode_input_ext.rs"]
mod transcode_decode_input_ext;
#[path = "../../src/stream/stream_codec_decode_error.rs"]
mod stream_codec_decode_error;

use transcode_decode_input_ext::TranscodeDecodeInputExt;
use qubit_codec::{TranscodeDecodeInput, Codec};
use qubit_codec_binary::NonStrict;
use qubit_io_binary::{
    BufferedBinaryReader, BufferedLeb128Reader, BufferedLeb128Writer, ByteOrder, LittleEndian,
};

#[derive(Default)]
struct FixedU16LeCodec;

#[derive(Default)]
struct SliceInput {
    data: Vec<u8>,
    position: usize,
}

impl SliceInput {
    fn new(data: impl Into<Vec<u8>>) -> Self {
        Self {
            data: data.into(),
            position: 0,
        }
    }
}

impl qubit_io::Input for SliceInput {
    type Item = u8;

    unsafe fn read_unchecked(
        &mut self,
        output: &mut [Self::Item],
        index: usize,
        count: usize,
    ) -> std::io::Result<usize> {
        let available = self.data.len().saturating_sub(self.position);
        let read = available.min(count);
        let end = self.position + read;
        output[index..index + read].copy_from_slice(&self.data[self.position..end]);
        self.position = end;
        Ok(read)
    }
}

#[derive(Default)]
struct U16Input {
    data: Vec<u16>,
    position: usize,
}

impl U16Input {
    fn new(data: impl Into<Vec<u16>>) -> Self {
        Self {
            data: data.into(),
            position: 0,
        }
    }
}

impl qubit_io::Input for U16Input {
    type Item = u16;

    unsafe fn read_unchecked(
        &mut self,
        output: &mut [Self::Item],
        index: usize,
        count: usize,
    ) -> std::io::Result<usize> {
        let available = self.data.len().saturating_sub(self.position);
        let read = available.min(count);
        let end = self.position + read;
        output[index..index + read].copy_from_slice(&self.data[self.position..end]);
        self.position = end;
        Ok(read)
    }
}

#[derive(Default)]
struct U16PairValueCodec;

unsafe impl Codec for U16PairValueCodec {
    type Value = u32;
    type Unit = u16;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;
    type DecodeState = ();
    type EncodeState = ();

    #[inline(always)]
    fn min_units_per_value(&self) -> NonZeroUsize {
        // SAFETY: 2 is non-zero.
        unsafe { NonZeroUsize::new_unchecked(2) }
    }

    #[inline(always)]
    fn max_units_per_value(&self) -> NonZeroUsize {
        // SAFETY: 2 is non-zero.
        unsafe { NonZeroUsize::new_unchecked(2) }
    }

    #[inline(always)]
    unsafe fn decode(
        &mut self,
        input: &[u16],
        index: usize,
    ) -> Result<(Self::Value, NonZeroUsize), Self::DecodeError> {
        let value = ((input[index] as u32) << 16) | (input[index + 1] as u32);
        Ok((value, unsafe { NonZeroUsize::new_unchecked(2) }))
    }

    #[inline(always)]
    unsafe fn encode(
        &mut self,
        value: &Self::Value,
        output: &mut [u16],
        index: usize,
    ) -> Result<usize, Self::EncodeError> {
        let bytes = value.to_be_bytes();
        output[index] = (bytes[0] as u16) << 8 | bytes[1] as u16;
        output[index + 1] = (bytes[2] as u16) << 8 | bytes[3] as u16;
        Ok(2)
    }
}

unsafe impl Codec for FixedU16LeCodec {
    type Value = u16;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;
    type DecodeState = ();
    type EncodeState = ();

    #[inline(always)]
    fn min_units_per_value(&self) -> NonZeroUsize {
        // SAFETY: 2 is non-zero.
        unsafe { NonZeroUsize::new_unchecked(2) }
    }

    #[inline(always)]
    fn max_units_per_value(&self) -> NonZeroUsize {
        // SAFETY: 2 is non-zero.
        unsafe { NonZeroUsize::new_unchecked(2) }
    }

    #[inline(always)]
    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(Self::Value, NonZeroUsize), Self::DecodeError> {
        let value = u16::from_le_bytes([input[index], input[index + 1]]);
        // SAFETY: fixed-width decode always consumes two bytes.
        Ok((value, unsafe { NonZeroUsize::new_unchecked(2) }))
    }

    #[inline(always)]
    unsafe fn encode(
        &mut self,
        value: &Self::Value,
        output: &mut [u8],
        index: usize,
    ) -> Result<usize, Self::EncodeError> {
        let bytes = value.to_le_bytes();
        output[index..index + 2].copy_from_slice(&bytes);
        Ok(2)
    }
}

#[test]
fn test_transcode_decode_input_ext_delegates_read() {
    let mut reader = BufferedBinaryReader::<_, LittleEndian>::with_capacity(
        Cursor::new(vec![0x34, 0x12, 0x56, 0x78]),
        1,
    );

    assert_eq!(ByteOrder::LittleEndian, reader.byte_order());
    let mut buffer = [0u8; 1];
    assert_eq!(
        1,
        reader
            .read(&mut buffer)
            .expect("raw read should return one byte")
    );
    assert_eq!(0x34, buffer[0]);
    assert_eq!(
        1,
        reader.seek(SeekFrom::Start(1)).expect("seek should work")
    );
    assert_eq!(
        0x5612,
        reader.read_u16().expect("read_decoded should still work")
    );
}

#[test]
fn test_transcode_decode_input_ext_maps_incomplete_decode_error() {
    let cursor = Cursor::new(vec![0b1000_0000]);
    let mut reader = BufferedLeb128Reader::<_, NonStrict>::with_capacity(cursor, 1);

    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader
            .read_i8()
            .expect_err("truncated leb128 should fail")
            .kind()
    );
}

#[test]
fn test_transcode_decode_input_ext_handles_utf8_length() {
    let value = "hello";
    let mut writer = BufferedLeb128Writer::new(Vec::new());
    writer.write_utf8_string(value).expect("encode payload");
    writer.flush().expect("flush should write encoded bytes");
    let bytes = writer.inner().clone();
    let cursor = Cursor::new(bytes);

    let mut reader = BufferedLeb128Reader::<_, NonStrict>::with_capacity(cursor, 1);
    let got = reader.read_utf8_string(10).expect("read payload");
    assert_eq!(value, got);
}

#[test]
fn test_transcode_decode_input_ext_accepts_input_without_read() {
    let mut input = TranscodeDecodeInput::with_capacity(SliceInput::new([0x34, 0x12]), 2);

    let value = input
        .read_decoded::<FixedU16LeCodec>()
        .expect("input-only source should still decode");

    assert_eq!(0x1234, value);
}

#[test]
fn test_transcode_decode_input_ext_accepts_non_u8_unit_input() {
    let mut input = TranscodeDecodeInput::with_capacity(U16Input::new(vec![0x11, 0x22]), 1);

    let value = input
        .read_decoded::<U16PairValueCodec>()
        .expect("input-only u16 unit source should still decode");

    assert_eq!(0x0011_0022, value);
}
