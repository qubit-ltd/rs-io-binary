use std::io::{
    Cursor,
    Seek,
    SeekFrom,
    Write,
};

use qubit_io_binary::{
    BufferedBinaryWriter,
    BufferedLeb128Reader,
    BufferedLeb128Writer,
    LittleEndian,
};

#[test]
fn test_buffered_encode_output_ext_writes_scalar_and_raw_bytes() {
    let mut writer =
        BufferedBinaryWriter::<_, LittleEndian>::with_capacity(Vec::new(), 4);
    writer.write_u16(0x1234).expect("encoded u16");
    assert_eq!(
        vec![0x34, 0x12],
        writer.into_inner().expect("extract written bytes")
    );
}

#[test]
fn test_buffered_encode_output_ext_writes_raw_bytes_via_io_trait() {
    let mut writer =
        BufferedBinaryWriter::<_, LittleEndian>::with_capacity(Vec::new(), 4);
    writer
        .write_all(b"ab")
        .expect("write_all should be delegated");
    let output = writer.into_inner().expect("extract raw bytes");
    assert_eq!(output, b"ab");
}

#[test]
fn test_buffered_encode_output_ext_seek_calls_flush() {
    let mut writer = BufferedLeb128Writer::new(Cursor::new(Vec::new()));
    writer.write_u8(1).expect("write_u8");
    let _ = writer
        .seek(SeekFrom::Start(0))
        .expect("seek should flush and succeed");
    let output = writer.into_inner().expect("extract output").into_inner();
    assert_eq!(output, vec![1]);
}

#[test]
fn test_buffered_encode_output_ext_writes_utf8_string() {
    let mut writer = BufferedLeb128Writer::new(Vec::new());
    writer
        .write_utf8_string("hello")
        .expect("write utf8 string");
    let bytes = writer.into_inner().expect("extract encoded bytes");

    let mut reader =
        BufferedLeb128Reader::<_, qubit_codec_binary::NonStrict>::new(
            Cursor::new(bytes),
        );
    assert_eq!("hello", reader.read_utf8_string(10).expect("read payload"));
}
