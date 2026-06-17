use std::convert::Infallible;
use std::io::{Cursor, Error, ErrorKind, Seek, SeekFrom, Write};
use std::num::NonZeroUsize;

use qubit_codec::{Codec, nz};
use qubit_io_binary::{
    BufferedBinaryWriter, BufferedLeb128Reader, BufferedLeb128Writer, LittleEndian,
    TranscodeEncodeOutput, TranscodeEncodeOutputExt,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct NonCopyValue([u8; 2]);

#[derive(Default)]
struct NonCopyValueCodec;

unsafe impl Codec for NonCopyValueCodec {
    type Value = NonCopyValue;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

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
    ) -> Result<(NonCopyValue, NonZeroUsize), Self::DecodeError> {
        Ok((
            NonCopyValue([input[index], input[index + 1]]),
            // SAFETY: decode always consumes exactly two bytes.
            unsafe { NonZeroUsize::new_unchecked(2) },
        ))
    }

    #[inline(always)]
    unsafe fn encode(
        &mut self,
        value: &NonCopyValue,
        output: &mut [u8],
        index: usize,
    ) -> Result<NonZeroUsize, Self::EncodeError> {
        debug_assert!(index + 2 <= output.len());
        output[index] = value.0[0];
        output[index + 1] = value.0[1];
        Ok(nz!(2))
    }
}

#[derive(Default)]
struct LargeFixedCodec;

#[derive(Default)]
struct ResetThenValueCodec {
    reset_emitted: bool,
}

#[derive(Default)]
struct FlushThenWriteLargeAfterFlushWriter {
    output: Vec<u8>,
    flushed: bool,
}

#[derive(Default)]
struct VecOutput {
    output: Vec<u8>,
    flushed: bool,
}

#[derive(Debug, Default)]
struct U16Output {
    output: Vec<u16>,
    flushed: bool,
}

impl Write for FlushThenWriteLargeAfterFlushWriter {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        if !self.flushed && buffer.len() > 1 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "write occurred before flush",
            ));
        }
        self.output.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flushed = true;
        Ok(())
    }
}

impl qubit_io::Output for VecOutput {
    type Item = u8;

    unsafe fn write(
        &mut self,
        input: &[Self::Item],
        index: usize,
        count: usize,
    ) -> std::io::Result<usize> {
        self.output.extend_from_slice(&input[index..index + count]);
        Ok(count)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flushed = true;
        Ok(())
    }
}

impl qubit_io::Output for U16Output {
    type Item = u16;

    unsafe fn write(
        &mut self,
        input: &[Self::Item],
        index: usize,
        count: usize,
    ) -> std::io::Result<usize> {
        self.output.extend_from_slice(&input[index..index + count]);
        Ok(count)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flushed = true;
        Ok(())
    }
}

#[derive(Default)]
struct U16PairCodec;

unsafe impl Codec for U16PairCodec {
    type Value = (u16, u16);
    type Unit = u16;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

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
        Ok(((input[index], input[index + 1]), unsafe {
            NonZeroUsize::new_unchecked(2)
        }))
    }

    #[inline(always)]
    unsafe fn encode(
        &mut self,
        value: &(u16, u16),
        output: &mut [u16],
        index: usize,
    ) -> Result<NonZeroUsize, Self::EncodeError> {
        output[index] = value.0;
        output[index + 1] = value.1;
        Ok(nz!(2))
    }
}

unsafe impl Codec for LargeFixedCodec {
    type Value = [u8; 4];
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    #[inline(always)]
    fn min_units_per_value(&self) -> NonZeroUsize {
        // SAFETY: 4 is non-zero.
        unsafe { NonZeroUsize::new_unchecked(4) }
    }

    #[inline(always)]
    fn max_units_per_value(&self) -> NonZeroUsize {
        // SAFETY: 4 is non-zero.
        unsafe { NonZeroUsize::new_unchecked(4) }
    }

    #[inline(always)]
    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<([u8; 4], NonZeroUsize), Self::DecodeError> {
        let mut value = [0; 4];
        value.copy_from_slice(&input[index..index + 4]);
        // SAFETY: fixed-width decode always consumes four bytes.
        Ok((value, unsafe { NonZeroUsize::new_unchecked(4) }))
    }

    #[inline(always)]
    unsafe fn encode(
        &mut self,
        value: &[u8; 4],
        output: &mut [u8],
        index: usize,
    ) -> Result<NonZeroUsize, Self::EncodeError> {
        output[index..index + 4].copy_from_slice(value);
        Ok(nz!(4))
    }
}

unsafe impl Codec for ResetThenValueCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    #[inline(always)]
    fn min_units_per_value(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    #[inline(always)]
    fn max_units_per_value(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    #[inline(always)]
    fn max_encode_reset_units(&self) -> usize {
        1
    }

    #[inline(always)]
    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, NonZeroUsize), Self::DecodeError> {
        Ok((input[index], NonZeroUsize::MIN))
    }

    #[inline(always)]
    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<NonZeroUsize, Self::EncodeError> {
        debug_assert!(self.reset_emitted);
        output[index] = *value;
        Ok(NonZeroUsize::MIN)
    }

    #[inline(always)]
    unsafe fn encode_reset(
        &mut self,
        output: &mut [u8],
        index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[index] = 0xFE;
        self.reset_emitted = true;
        Ok(1)
    }
}

#[test]
fn test_transcode_encode_output_ext_accepts_non_copy_codec_values() {
    let mut output = TranscodeEncodeOutput::with_capacity(Vec::new(), 19);
    output
        .write_encoded::<NonCopyValueCodec>(NonCopyValue([0x34, 0x12]))
        .expect("write non-copy value");
    output.flush().expect("flush encoded bytes");
    let (inner, _) = output.into_parts();
    assert_eq!(inner, vec![0x34, 0x12]);
}

#[test]
fn test_transcode_encode_output_ext_writes_encode_reset_before_value() {
    let mut output = TranscodeEncodeOutput::with_capacity(Vec::new(), 2);

    output
        .write_encoded::<ResetThenValueCodec>(0x11)
        .expect("write reset and value");
    output.flush().expect("flush encoded bytes");

    let (inner, _) = output.into_parts();
    assert_eq!(inner, vec![0xFE, 0x11]);
}

#[test]
fn test_transcode_encode_output_ext_encodes_with_tiny_buffer_capacity() {
    let mut output = TranscodeEncodeOutput::with_capacity(Vec::new(), 1);

    output
        .write_encoded::<LargeFixedCodec>([0x11, 0x22, 0x33, 0x44])
        .expect("codec should be encoded despite tiny buffer capacity");
    output.flush().expect("flush encoded bytes");

    let (inner, _) = output.into_parts();
    assert_eq!(inner, vec![0x11, 0x22, 0x33, 0x44]);
}

#[test]
fn test_transcode_encode_output_ext_fallback_preserves_pending_bytes() {
    let mut output = TranscodeEncodeOutput::with_capacity(Vec::new(), 1);

    output
        .write_all(&[0xAA])
        .expect("stage pending byte in inner buffer");
    output
        .write_encoded::<LargeFixedCodec>([0x11, 0x22, 0x33, 0x44])
        .expect("fallback path should still emit buffered prefix");
    output.flush().expect("flush encoded bytes");

    let (inner, _) = output.into_parts();
    assert_eq!(inner, vec![0xAA, 0x11, 0x22, 0x33, 0x44]);
}

#[test]
fn test_transcode_encode_output_ext_fallback_calls_flush_before_write() {
    let mut output =
        TranscodeEncodeOutput::with_capacity(FlushThenWriteLargeAfterFlushWriter::default(), 1);

    output
        .write_all(&[0xAA])
        .expect("stage pending byte in inner buffer");
    output
        .write_encoded::<LargeFixedCodec>([0x11, 0x22, 0x33, 0x44])
        .expect("fallback should succeed after flush");

    let (inner, _) = output.into_parts();
    assert!(inner.flushed);
    assert_eq!(inner.output, vec![0xAA, 0x11, 0x22, 0x33, 0x44]);
}

#[test]
fn test_transcode_encode_output_ext_accepts_output_without_write() {
    let mut output = TranscodeEncodeOutput::with_capacity(VecOutput::default(), 1);

    output
        .write_encoded::<LargeFixedCodec>([0x11, 0x22, 0x33, 0x44])
        .expect("output-only target should still be accepted");
    output.flush().expect("flush encoded bytes");

    let (inner, pending) = output.into_parts();
    assert!(pending.is_empty());
    assert!(inner.flushed);
    assert_eq!(inner.output, vec![0x11, 0x22, 0x33, 0x44]);
}

#[test]
fn test_transcode_encode_output_ext_accepts_non_u8_unit_output() {
    let mut output = TranscodeEncodeOutput::with_capacity(U16Output::default(), 1);

    output
        .write_encoded::<U16PairCodec>((0x1111, 0x2222))
        .expect("non-u8 unit output should be accepted");
    output.flush().expect("flush encoded u16 units");

    let (inner, pending) = output.into_parts();
    assert!(pending.is_empty());
    assert!(inner.flushed);
    assert_eq!(inner.output, vec![0x1111, 0x2222]);
}

#[test]
fn test_transcode_encode_output_ext_writes_scalar_and_raw_bytes() {
    let mut writer = BufferedBinaryWriter::<_, LittleEndian>::with_capacity(Vec::new(), 4);
    writer.write_u16(0x1234).expect("encoded u16");
    assert_eq!(vec![0x34, 0x12], {
        writer.flush().expect("flush should write bytes");
        writer.inner().clone()
    });
}

#[test]
fn test_transcode_encode_output_ext_writes_raw_bytes_via_io_trait() {
    let mut writer = BufferedBinaryWriter::<_, LittleEndian>::with_capacity(Vec::new(), 4);
    writer
        .write_all(b"ab")
        .expect("write_all should be delegated");
    writer.flush().expect("flush should write raw bytes");
    let output = writer.inner().clone();
    assert_eq!(output, b"ab");
}

#[test]
fn test_transcode_encode_output_ext_writes_multiple_values_with_tiny_capacity() {
    let mut writer = BufferedBinaryWriter::<_, LittleEndian>::with_capacity(Vec::new(), 1);
    writer.write_u16(0x1234).expect("write first u16");
    writer.write_u32(0x89ABCDEF).expect("write u32");
    writer.write_u16(0x0102).expect("write second u16");
    writer.write_u8(0xFF).expect("write u8");

    writer.flush().expect("flush should write bytes");
    let output = writer.inner().clone();
    let mut expected = Vec::new();
    expected.extend_from_slice(&0x1234_u16.to_le_bytes());
    expected.extend_from_slice(&0x89ABCDEF_u32.to_le_bytes());
    expected.extend_from_slice(&0x0102_u16.to_le_bytes());
    expected.push(0xFF);
    assert_eq!(expected, output);
}

#[test]
fn test_transcode_encode_output_ext_seek_calls_flush() {
    let mut writer = BufferedLeb128Writer::new(Cursor::new(Vec::new()));
    writer.write_u8(1).expect("write_u8");
    let _ = writer
        .seek(SeekFrom::Start(0))
        .expect("seek should flush and succeed");
    writer.flush().expect("flush should write output");
    let output = writer.inner().clone().into_inner();
    assert_eq!(output, vec![1]);
}

#[test]
fn test_transcode_encode_output_ext_writes_utf8_string() {
    let mut writer = BufferedLeb128Writer::new(Vec::new());
    writer
        .write_utf8_string("hello")
        .expect("write utf8 string");
    writer.flush().expect("flush should write encoded bytes");
    let bytes = writer.inner().clone();

    let mut reader =
        BufferedLeb128Reader::<_, qubit_codec_binary::NonStrict>::new(Cursor::new(bytes));
    assert_eq!("hello", reader.read_utf8_string(10).expect("read payload"));
}
