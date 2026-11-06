// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Extension methods for writing length-prefixed UTF-8 strings.

use std::io::Result;

use qubit_codec::ByteOrder;
use qubit_io::ByteOutput;

use crate::BinaryWriteExt;
use crate::Leb128WriteExt;
use crate::util::checked_u64_len;
use crate::util::write_all;
use crate::util::write_utf8_payload as write_utf8_payload_impl;
use crate::util::write_utf8_string_with_u16_len;
use crate::util::write_utf8_string_with_u32_len;

/// Extension methods for writing length-prefixed UTF-8 strings.
///
/// A failed write can leave the prefix or payload bytes already written to the
/// output. Length-prefixed methods are not transactional: retrying the whole
/// method can append a second length prefix. Recover using the enclosing
/// protocol's framing or transaction boundary instead.
///
/// # Examples
///
/// ```
/// use qubit_io_binary::StringWriteExt;
///
/// let mut output = Vec::new();
/// output.write_string_with_u16_len_be("Ada")?;
/// assert_eq!(output, [0, 3, b'A', b'd', b'a']);
/// # Ok::<(), std::io::Error>(())
/// ```
pub trait StringWriteExt: ByteOutput {
    /// Writes a UTF-8 payload without a length prefix.
    ///
    /// # Parameters
    ///
    /// - `value`: String slice to write.
    ///
    /// # Returns
    ///
    /// Returns after the payload has been written.
    ///
    /// # Errors
    ///
    /// Returns an I/O error from the underlying writer.
    fn write_utf8_payload(&mut self, value: &str) -> Result<()>;

    /// Writes a UTF-8 string with an unsigned LEB128 byte-length prefix.
    ///
    /// The length prefix is encoded as `usize`, so this format is target-width
    /// dependent. Prefer `u16` or `u32` length-prefix methods for persistent
    /// files and cross-platform protocols.
    ///
    /// # Parameters
    ///
    /// - `value`: String slice to write.
    ///
    /// # Returns
    ///
    /// Returns after the prefix and payload have been written.
    ///
    /// # Errors
    ///
    /// Returns an I/O error from the underlying writer.
    fn write_utf8_string_uleb_usize(&mut self, value: &str) -> Result<()>;

    /// Writes a UTF-8 string with an unsigned LEB128 `u64` byte-length prefix.
    ///
    /// Prefer this method over [`Self::write_utf8_string_uleb_usize`] for
    /// persistent files and cross-platform protocols because the length
    /// field is independent of the current Rust target's pointer width.
    ///
    /// # Parameters
    ///
    /// - `value`: String slice to write.
    ///
    /// # Returns
    ///
    /// Returns after the prefix and payload have been written.
    ///
    /// # Errors
    ///
    /// Returns [`std::io::ErrorKind::InvalidInput`] when the UTF-8 byte length
    /// cannot be represented as `u64`, or an I/O error from the underlying
    /// writer.
    fn write_utf8_string_uleb_u64(&mut self, value: &str) -> Result<()>;

    /// Writes a UTF-8 string with a runtime-order `u16` byte-length prefix.
    ///
    /// # Parameters
    ///
    /// - `value`: String slice to write.
    /// - `byte_order`: Byte order used by the length prefix.
    ///
    /// # Returns
    ///
    /// Returns after the prefix and payload have been written.
    ///
    /// # Errors
    ///
    /// Returns [`std::io::ErrorKind::InvalidInput`] when the UTF-8 byte length
    /// does not fit into `u16`, or an I/O error from the underlying writer.
    fn write_string_with_u16_len(&mut self, value: &str, byte_order: ByteOrder) -> Result<()>;

    /// Writes a UTF-8 string with a big-endian `u16` byte-length prefix.
    ///
    /// # Parameters
    ///
    /// - `value`: String slice to write.
    ///
    /// # Returns
    ///
    /// Returns after the prefix and payload have been written.
    ///
    /// # Errors
    ///
    /// Returns [`std::io::ErrorKind::InvalidInput`] when the UTF-8 byte length
    /// does not fit into `u16`, or an I/O error from the underlying writer.
    fn write_string_with_u16_len_be(&mut self, value: &str) -> Result<()>;

    /// Writes a UTF-8 string with a little-endian `u16` byte-length prefix.
    ///
    /// # Parameters
    ///
    /// - `value`: String slice to write.
    ///
    /// # Returns
    ///
    /// Returns after the prefix and payload have been written.
    ///
    /// # Errors
    ///
    /// Returns [`std::io::ErrorKind::InvalidInput`] when the UTF-8 byte length
    /// does not fit into `u16`, or an I/O error from the underlying writer.
    fn write_string_with_u16_len_le(&mut self, value: &str) -> Result<()>;

    /// Writes a UTF-8 string with a runtime-order `u32` byte-length prefix.
    ///
    /// # Parameters
    ///
    /// - `value`: String slice to write.
    /// - `byte_order`: Byte order used by the length prefix.
    ///
    /// # Returns
    ///
    /// Returns after the prefix and payload have been written.
    ///
    /// # Errors
    ///
    /// Returns [`std::io::ErrorKind::InvalidInput`] when the UTF-8 byte length
    /// does not fit into `u32`, or an I/O error from the underlying writer.
    fn write_string_with_u32_len(&mut self, value: &str, byte_order: ByteOrder) -> Result<()>;

    /// Writes a UTF-8 string with a big-endian `u32` byte-length prefix.
    ///
    /// # Parameters
    ///
    /// - `value`: String slice to write.
    ///
    /// # Returns
    ///
    /// Returns after the prefix and payload have been written.
    ///
    /// # Errors
    ///
    /// Returns [`std::io::ErrorKind::InvalidInput`] when the UTF-8 byte length
    /// does not fit into `u32`, or an I/O error from the underlying writer.
    fn write_string_with_u32_len_be(&mut self, value: &str) -> Result<()>;

    /// Writes a UTF-8 string with a little-endian `u32` byte-length prefix.
    ///
    /// # Parameters
    ///
    /// - `value`: String slice to write.
    ///
    /// # Returns
    ///
    /// Returns after the prefix and payload have been written.
    ///
    /// # Errors
    ///
    /// Returns [`std::io::ErrorKind::InvalidInput`] when the UTF-8 byte length
    /// does not fit into `u32`, or an I/O error from the underlying writer.
    fn write_string_with_u32_len_le(&mut self, value: &str) -> Result<()>;
}

impl<T> StringWriteExt for T
where
    T: ByteOutput + ?Sized,
{
    /// Writes the string's UTF-8 bytes without adding a length prefix.
    ///
    /// A write failure may occur after some payload bytes have reached the
    /// underlying output.
    ///
    /// # Parameters
    ///
    /// - `value`: UTF-8 string whose bytes are written to the output.
    ///
    /// # Returns
    ///
    /// Returns after all UTF-8 payload bytes have been written.
    ///
    /// # Errors
    ///
    /// Returns an I/O error from the underlying output.
    #[inline]
    fn write_utf8_payload(&mut self, value: &str) -> Result<()> {
        write_utf8_payload_impl(self, value)
    }

    /// Writes the UTF-8 byte length as target-width-dependent unsigned LEB128,
    /// followed by the string bytes.
    ///
    /// A failure can leave a partial prefix or payload in the output.
    ///
    /// # Parameters
    ///
    /// - `value`: UTF-8 string to write after its target-width-dependent byte
    ///   length.
    ///
    /// # Returns
    ///
    /// Returns after the length prefix and payload have been written.
    ///
    /// # Errors
    ///
    /// Returns an I/O error from writing the prefix or payload.
    #[inline]
    fn write_utf8_string_uleb_usize(&mut self, value: &str) -> Result<()> {
        let bytes = value.as_bytes();
        self.write_uleb_usize(bytes.len())?;
        write_all(self, bytes)
    }

    /// Writes the UTF-8 byte length as unsigned LEB128 `u64`, then the string
    /// bytes.
    ///
    /// Returns `InvalidInput` if the byte length does not fit in `u64`; a
    /// write failure can leave a partial prefix or payload in the output.
    ///
    /// # Parameters
    ///
    /// - `value`: UTF-8 string to write after its `u64` byte length.
    ///
    /// # Returns
    ///
    /// Returns after the length prefix and payload have been written.
    ///
    /// # Errors
    ///
    /// Returns `InvalidInput` if the byte length does not fit in `u64`, or an
    /// I/O error while writing.
    #[inline]
    fn write_utf8_string_uleb_u64(&mut self, value: &str) -> Result<()> {
        let bytes = value.as_bytes();
        self.write_uleb_u64(checked_u64_len(bytes.len())?)?;
        write_all(self, bytes)
    }

    /// Writes a `u16` UTF-8 byte-length prefix in the requested byte order,
    /// followed by the string bytes.
    ///
    /// Returns `InvalidInput` if the byte length exceeds `u16::MAX`; a write
    /// failure can leave a partial prefix or payload in the output.
    ///
    /// # Parameters
    ///
    /// - `value`: UTF-8 string to write after its `u16` byte length.
    /// - `byte_order`: Byte order used to encode the length prefix.
    ///
    /// # Returns
    ///
    /// Returns after the length prefix and payload have been written.
    ///
    /// # Errors
    ///
    /// Returns `InvalidInput` if the byte length exceeds `u16::MAX`, or an
    /// I/O error while writing.
    #[inline]
    fn write_string_with_u16_len(&mut self, value: &str, byte_order: ByteOrder) -> Result<()> {
        write_utf8_string_with_u16_len(self, value, |writer, len| writer.write_u16(len, byte_order))
    }

    /// Writes a big-endian `u16` UTF-8 byte-length prefix, then the string
    /// bytes.
    ///
    /// Returns `InvalidInput` if the byte length exceeds `u16::MAX`; a write
    /// failure can leave a partial prefix or payload in the output.
    ///
    /// # Parameters
    ///
    /// - `value`: UTF-8 string to write after its big-endian `u16` byte length.
    ///
    /// # Returns
    ///
    /// Returns after the length prefix and payload have been written.
    ///
    /// # Errors
    ///
    /// Returns `InvalidInput` if the byte length exceeds `u16::MAX`, or an
    /// I/O error while writing.
    #[inline]
    fn write_string_with_u16_len_be(&mut self, value: &str) -> Result<()> {
        write_utf8_string_with_u16_len(self, value, |writer, len| writer.write_u16_be(len))
    }

    /// Writes a little-endian `u16` UTF-8 byte-length prefix, then the string
    /// bytes.
    ///
    /// Returns `InvalidInput` if the byte length exceeds `u16::MAX`; a write
    /// failure can leave a partial prefix or payload in the output.
    ///
    /// # Parameters
    ///
    /// - `value`: UTF-8 string to write after its little-endian `u16` byte
    ///   length.
    ///
    /// # Returns
    ///
    /// Returns after the length prefix and payload have been written.
    ///
    /// # Errors
    ///
    /// Returns `InvalidInput` if the byte length exceeds `u16::MAX`, or an
    /// I/O error while writing.
    #[inline]
    fn write_string_with_u16_len_le(&mut self, value: &str) -> Result<()> {
        write_utf8_string_with_u16_len(self, value, |writer, len| writer.write_u16_le(len))
    }

    /// Writes a `u32` UTF-8 byte-length prefix in the requested byte order,
    /// followed by the string bytes.
    ///
    /// Returns `InvalidInput` if the byte length exceeds `u32::MAX`; a write
    /// failure can leave a partial prefix or payload in the output.
    ///
    /// # Parameters
    ///
    /// - `value`: UTF-8 string to write after its `u32` byte length.
    /// - `byte_order`: Byte order used to encode the length prefix.
    ///
    /// # Returns
    ///
    /// Returns after the length prefix and payload have been written.
    ///
    /// # Errors
    ///
    /// Returns `InvalidInput` if the byte length exceeds `u32::MAX`, or an
    /// I/O error while writing.
    #[inline]
    fn write_string_with_u32_len(&mut self, value: &str, byte_order: ByteOrder) -> Result<()> {
        write_utf8_string_with_u32_len(self, value, |writer, len| writer.write_u32(len, byte_order))
    }

    /// Writes a big-endian `u32` UTF-8 byte-length prefix, then the string
    /// bytes.
    ///
    /// Returns `InvalidInput` if the byte length exceeds `u32::MAX`; a write
    /// failure can leave a partial prefix or payload in the output.
    ///
    /// # Parameters
    ///
    /// - `value`: UTF-8 string to write after its big-endian `u32` byte length.
    ///
    /// # Returns
    ///
    /// Returns after the length prefix and payload have been written.
    ///
    /// # Errors
    ///
    /// Returns `InvalidInput` if the byte length exceeds `u32::MAX`, or an
    /// I/O error while writing.
    #[inline]
    fn write_string_with_u32_len_be(&mut self, value: &str) -> Result<()> {
        write_utf8_string_with_u32_len(self, value, |writer, len| writer.write_u32_be(len))
    }

    /// Writes a little-endian `u32` UTF-8 byte-length prefix, then the string
    /// bytes.
    ///
    /// Returns `InvalidInput` if the byte length exceeds `u32::MAX`; a write
    /// failure can leave a partial prefix or payload in the output.
    ///
    /// # Parameters
    ///
    /// - `value`: UTF-8 string to write after its little-endian `u32` byte
    ///   length.
    ///
    /// # Returns
    ///
    /// Returns after the length prefix and payload have been written.
    ///
    /// # Errors
    ///
    /// Returns `InvalidInput` if the byte length exceeds `u32::MAX`, or an
    /// I/O error while writing.
    #[inline]
    fn write_string_with_u32_len_le(&mut self, value: &str) -> Result<()> {
        write_utf8_string_with_u32_len(self, value, |writer, len| writer.write_u32_le(len))
    }
}
