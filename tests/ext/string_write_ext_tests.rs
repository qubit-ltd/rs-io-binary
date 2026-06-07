use std::io::{
    Error,
    ErrorKind,
    Write,
};
#[cfg(all(unix, target_pointer_width = "64"))]
use std::{
    ffi::c_void,
    ptr::null_mut,
};

use qubit_io_binary::{
    ByteOrder,
    StringWriteExt,
};

#[cfg(all(unix, target_pointer_width = "64", target_os = "macos"))]
const MAP_ANONYMOUS_FLAG: i32 = 0x1000;

#[cfg(all(unix, target_pointer_width = "64", any(target_os = "android", target_os = "linux")))]
const MAP_ANONYMOUS_FLAG: i32 = 0x20;

#[cfg(all(unix, target_pointer_width = "64"))]
const MAP_PRIVATE_FLAG: i32 = 0x02;

#[cfg(all(unix, target_pointer_width = "64"))]
const PROT_READ_FLAG: i32 = 0x01;

#[cfg(all(unix, target_pointer_width = "64"))]
const MAP_FAILED: *mut c_void = -1_isize as *mut c_void;

#[cfg(all(unix, target_pointer_width = "64"))]
unsafe extern "C" {
    fn mmap(addr: *mut c_void, len: usize, prot: i32, flags: i32, fd: i32, offset: i64) -> *mut c_void;
    fn munmap(addr: *mut c_void, len: usize) -> i32;
}

#[cfg(all(unix, target_pointer_width = "64"))]
struct MappedBytes {
    ptr: *mut c_void,
    len: usize,
}

#[cfg(all(unix, target_pointer_width = "64"))]
impl MappedBytes {
    fn new_zeroed(len: usize) -> Self {
        // SAFETY: The mapping is anonymous, private, read-only, and requests no fixed address.
        let ptr = unsafe {
            mmap(
                null_mut(),
                len,
                PROT_READ_FLAG,
                MAP_PRIVATE_FLAG | MAP_ANONYMOUS_FLAG,
                -1,
                0,
            )
        };
        assert_ne!(MAP_FAILED, ptr, "failed to reserve sparse zeroed test mapping");
        Self { ptr, len }
    }

    fn as_str(&self) -> &str {
        // SAFETY: Anonymous mappings are zero-filled, and NUL bytes are valid UTF-8.
        let bytes = unsafe { std::slice::from_raw_parts(self.ptr.cast::<u8>(), self.len) };
        // SAFETY: The byte slice consists entirely of valid UTF-8 NUL bytes.
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }
}

#[cfg(all(unix, target_pointer_width = "64"))]
impl Drop for MappedBytes {
    fn drop(&mut self) {
        // SAFETY: `ptr` and `len` come from a successful `mmap` call in `new_zeroed`.
        let result = unsafe { munmap(self.ptr, self.len) };
        debug_assert_eq!(0, result);
    }
}

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
fn test_string_write_ext_writes_all_length_prefix_kinds() {
    let mut output = Vec::new();

    output.write_utf8_payload("raw").expect("payload should be written");
    output
        .write_utf8_string_uleb("hi")
        .expect("ULEB string should be written");
    output
        .write_utf8_string_uleb_u64("u6")
        .expect("u64 ULEB string should be written");
    output
        .write_utf8_string_u16("rt", ByteOrder::BigEndian)
        .expect("runtime u16 BE string should be written");
    output
        .write_utf8_string_u16_be("be")
        .expect("u16 BE string should be written");
    output
        .write_utf8_string_u16("lr", ByteOrder::LittleEndian)
        .expect("runtime u16 LE string should be written");
    output
        .write_utf8_string_u16_le("le")
        .expect("u16 LE string should be written");
    output
        .write_utf8_string_u32("up", ByteOrder::BigEndian)
        .expect("runtime u32 BE string should be written");
    output
        .write_utf8_string_u32_be("up")
        .expect("u32 BE string should be written");
    output
        .write_utf8_string_u32("dn", ByteOrder::LittleEndian)
        .expect("runtime u32 LE string should be written");
    output
        .write_utf8_string_u32_le("dn")
        .expect("u32 LE string should be written");

    assert_eq!(
        vec![
            b'r', b'a', b'w', 0x02, b'h', b'i', 0x02, b'u', b'6', 0x00, 0x02, b'r', b't', 0x00, 0x02, b'b', b'e',
            0x02, 0x00, b'l', b'r', 0x02, 0x00, b'l', b'e', 0x00, 0x00, 0x00, 0x02, b'u', b'p', 0x00, 0x00, 0x00,
            0x02, b'u', b'p', 0x02, 0x00, 0x00, 0x00, b'd', b'n', 0x02, 0x00, 0x00, 0x00, b'd', b'n'
        ],
        output
    );
}

#[test]
fn test_string_write_ext_reports_length_and_writer_errors() {
    let mut output = Vec::new();
    let value = "x".repeat(usize::from(u16::MAX) + 1);
    assert_eq!(
        ErrorKind::InvalidInput,
        output
            .write_utf8_string_u16(&value, ByteOrder::BigEndian)
            .expect_err("oversized runtime u16 string should fail")
            .kind()
    );
    assert_eq!(
        ErrorKind::InvalidInput,
        output
            .write_utf8_string_u16_be(&value)
            .expect_err("oversized u16 BE string should fail")
            .kind()
    );
    assert_eq!(
        ErrorKind::InvalidInput,
        output
            .write_utf8_string_u16_le(&value)
            .expect_err("oversized u16 LE string should fail")
            .kind()
    );

    let mut writer = FailingWriter;
    assert_eq!(
        ErrorKind::Other,
        writer
            .write_utf8_payload("hi")
            .expect_err("payload writer error should be returned")
            .kind()
    );

    let mut writer = FailingWriter;
    assert_eq!(
        ErrorKind::Other,
        writer
            .write_utf8_string_uleb("hi")
            .expect_err("writer error should be returned")
            .kind()
    );

    let mut writer = FailingWriter;
    assert_eq!(
        ErrorKind::Other,
        writer
            .write_utf8_string_uleb_u64("hi")
            .expect_err("u64 ULEB writer error should be returned")
            .kind()
    );

    let mut writer = FailingWriter;
    assert_eq!(
        ErrorKind::Other,
        writer
            .write_utf8_string_u16("hi", ByteOrder::BigEndian)
            .expect_err("runtime u16 writer error should be returned")
            .kind()
    );

    let mut writer = FailingWriter;
    assert_eq!(
        ErrorKind::Other,
        writer
            .write_utf8_string_u16_be("hi")
            .expect_err("u16 BE writer error should be returned")
            .kind()
    );

    let mut writer = FailingWriter;
    assert_eq!(
        ErrorKind::Other,
        writer
            .write_utf8_string_u16_le("hi")
            .expect_err("u16 LE writer error should be returned")
            .kind()
    );

    let mut writer = FailingWriter;
    assert_eq!(
        ErrorKind::Other,
        writer
            .write_utf8_string_u32("hi", ByteOrder::BigEndian)
            .expect_err("runtime u32 writer error should be returned")
            .kind()
    );

    let mut writer = FailingWriter;
    assert_eq!(
        ErrorKind::Other,
        writer
            .write_utf8_string_u32_be("hi")
            .expect_err("u32 BE writer error should be returned")
            .kind()
    );

    let mut writer = FailingWriter;
    assert_eq!(
        ErrorKind::Other,
        writer
            .write_utf8_string_u32_le("hi")
            .expect_err("u32 LE writer error should be returned")
            .kind()
    );
}

#[cfg(all(unix, target_pointer_width = "64"))]
#[test]
fn test_string_write_ext_reports_u32_length_overflow() {
    let value = MappedBytes::new_zeroed(u32::MAX as usize + 1);
    let mut output = Vec::new();

    let error = output
        .write_utf8_string_u32(value.as_str(), ByteOrder::BigEndian)
        .expect_err("oversized u32 string should fail");

    assert_eq!(ErrorKind::InvalidInput, error.kind());
}
