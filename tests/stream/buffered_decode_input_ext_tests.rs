use std::io::{
    Cursor,
    ErrorKind,
    Read,
    Seek,
    SeekFrom,
};

use qubit_codec_binary::NonStrict;
use qubit_io_binary::{
    BufferedBinaryReader,
    BufferedLeb128Reader,
    BufferedLeb128Writer,
    ByteOrder,
    LittleEndian,
};

#[test]
fn test_buffered_decode_input_ext_delegates_read() {
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
fn test_buffered_decode_input_ext_maps_incomplete_decode_error() {
    let cursor = Cursor::new(vec![0b1000_0000]);
    let mut reader =
        BufferedLeb128Reader::<_, NonStrict>::with_capacity(cursor, 1);

    assert_eq!(
        ErrorKind::UnexpectedEof,
        reader
            .read_i8()
            .expect_err("truncated leb128 should fail")
            .kind()
    );
}

#[test]
fn test_buffered_decode_input_ext_handles_utf8_length() {
    let value = "hello";
    let mut writer = BufferedLeb128Writer::new(Vec::new());
    writer.write_utf8_string(value).expect("encode payload");
    let bytes = writer.into_inner().expect("extract encoded bytes");
    let cursor = Cursor::new(bytes);

    let mut reader =
        BufferedLeb128Reader::<_, NonStrict>::with_capacity(cursor, 1);
    let got = reader.read_utf8_string(10).expect("read payload");
    assert_eq!(value, got);
}
