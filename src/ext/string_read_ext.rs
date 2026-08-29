// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Extension methods for reading length-prefixed UTF-8 strings.

use std::io::Result;

use qubit_codec::ByteOrder;
use qubit_io::Input;

use crate::BinaryReadExt;
use crate::Leb128ReadExt;
use crate::util::read_utf8_payload as read_utf8_payload_impl;
use crate::util::read_utf8_payload_into as read_utf8_payload_into_impl;
#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
use crate::util::usize_from_u32_len;
#[cfg(not(target_pointer_width = "64"))]
use crate::util::usize_from_u64_len;

/// Extension methods for reading length-prefixed UTF-8 strings.
///
/// String-producing methods may return allocation failures as I/O errors in
/// addition to input and validation errors described below.
pub trait StringReadExt: Input<Item = u8> {
    /// Reads and validates a UTF-8 payload into reusable byte storage.
    ///
    /// # Parameters
    ///
    /// - `bytes`: Reusable destination buffer. It is cleared before a permitted
    ///   read and contains the received bytes when the read succeeds or when
    ///   UTF-8 validation fails.
    /// - `len`: UTF-8 payload length in bytes.
    /// - `max_len`: Maximum accepted UTF-8 payload length in bytes.
    ///
    /// # Returns
    ///
    /// Returns after `bytes` contains one valid UTF-8 payload.
    ///
    /// # Errors
    ///
    /// Returns an I/O error for payload reads or allocation failures, or
    /// [`std::io::ErrorKind::InvalidData`] when `len` exceeds `max_len` or
    /// the payload is not valid UTF-8.
    fn read_utf8_payload_into(&mut self, bytes: &mut Vec<u8>, len: usize, max_len: usize) -> Result<()>;

    /// Reads a UTF-8 payload with an already decoded byte length.
    ///
    /// # Parameters
    ///
    /// - `len`: UTF-8 payload length in bytes.
    /// - `max_len`: Maximum accepted UTF-8 payload length in bytes.
    ///
    /// # Returns
    ///
    /// The decoded string.
    ///
    /// # Errors
    ///
    /// Returns an I/O error for payload reads,
    /// [`std::io::ErrorKind::InvalidData`] when `len` exceeds `max_len`, or
    /// [`std::io::ErrorKind::InvalidData`] when the payload is not valid
    /// UTF-8.
    fn read_utf8_payload(&mut self, len: usize, max_len: usize) -> Result<String>;

    /// Reads a UTF-8 string with an unsigned LEB128 byte-length prefix.
    ///
    /// The length prefix is decoded as `usize`, so this format is target-width
    /// dependent. Prefer `u16` or `u32` length-prefix methods for persistent
    /// files and cross-platform protocols.
    ///
    /// # Parameters
    ///
    /// - `max_len`: Maximum accepted UTF-8 payload length in bytes.
    ///
    /// # Returns
    ///
    /// The decoded string.
    ///
    /// # Errors
    ///
    /// Returns an I/O error for length or payload reads,
    /// [`std::io::ErrorKind::InvalidData`] when the encoded length exceeds
    /// `max_len`, or [`std::io::ErrorKind::InvalidData`] when the payload
    /// is not valid UTF-8.
    fn read_utf8_string_uleb_usize(&mut self, max_len: usize) -> Result<String>;

    /// Reads a UTF-8 string with a canonical unsigned LEB128 byte-length
    /// prefix.
    ///
    /// The length prefix is decoded as `usize`, so this format is target-width
    /// dependent. Prefer `u16` or `u32` length-prefix methods for persistent
    /// files and cross-platform protocols.
    ///
    /// # Parameters
    ///
    /// - `max_len`: Maximum accepted UTF-8 payload length in bytes.
    ///
    /// # Returns
    ///
    /// The decoded string.
    ///
    /// # Errors
    ///
    /// Returns an I/O error for length or payload reads,
    /// [`std::io::ErrorKind::InvalidData`] when the length prefix is
    /// malformed or non-canonical, [`std::io::ErrorKind::InvalidData`] when
    /// the encoded length exceeds `max_len`, or
    /// [`std::io::ErrorKind::InvalidData`] when the payload is not valid
    /// UTF-8.
    fn read_utf8_string_uleb_usize_strict(&mut self, max_len: usize) -> Result<String>;

    /// Reads a UTF-8 string with an unsigned LEB128 `u64` byte-length prefix.
    ///
    /// Prefer this method over [`Self::read_utf8_string_uleb_usize`] for
    /// persistent files and cross-platform protocols because the length
    /// field is independent of the current Rust target's pointer width.
    ///
    /// # Parameters
    ///
    /// - `max_len`: Maximum accepted UTF-8 payload length in bytes.
    ///
    /// # Returns
    ///
    /// The decoded string.
    ///
    /// # Errors
    ///
    /// Returns an I/O error for length or payload reads,
    /// [`std::io::ErrorKind::InvalidData`] when the encoded `u64` length
    /// does not fit the local `usize`, when the encoded length exceeds
    /// `max_len`, or when the payload is not valid UTF-8.
    fn read_utf8_string_uleb_u64(&mut self, max_len: usize) -> Result<String>;

    /// Reads a UTF-8 string with a canonical unsigned LEB128 `u64` byte-length
    /// prefix.
    ///
    /// Prefer this method over [`Self::read_utf8_string_uleb_usize_strict`] for
    /// persistent files and cross-platform protocols because the length field
    /// is independent of the current Rust target's pointer width.
    ///
    /// # Parameters
    ///
    /// - `max_len`: Maximum accepted UTF-8 payload length in bytes.
    ///
    /// # Returns
    ///
    /// The decoded string.
    ///
    /// # Errors
    ///
    /// Returns an I/O error for length or payload reads,
    /// [`std::io::ErrorKind::InvalidData`] when the length prefix is
    /// malformed or non-canonical, when the encoded `u64` length does not
    /// fit the local `usize`, when the encoded length exceeds `max_len`, or
    /// when the payload is not valid UTF-8.
    fn read_utf8_string_uleb_u64_strict(&mut self, max_len: usize) -> Result<String>;

    /// Reads a UTF-8 string with a runtime-order `u16` byte-length prefix.
    ///
    /// # Parameters
    ///
    /// - `byte_order`: Byte order used by the length prefix.
    /// - `max_len`: Maximum accepted UTF-8 payload length in bytes.
    ///
    /// # Returns
    ///
    /// The decoded string.
    ///
    /// # Errors
    ///
    /// Returns an I/O error for length or payload reads,
    /// [`std::io::ErrorKind::InvalidData`] when the encoded length exceeds
    /// `max_len`, or [`std::io::ErrorKind::InvalidData`] when the payload
    /// is not valid UTF-8.
    fn read_string_with_u16_len(&mut self, byte_order: ByteOrder, max_len: usize) -> Result<String>;

    /// Reads a UTF-8 string with a big-endian `u16` byte-length prefix.
    ///
    /// # Parameters
    ///
    /// - `max_len`: Maximum accepted UTF-8 payload length in bytes.
    ///
    /// # Returns
    ///
    /// The decoded string.
    ///
    /// # Errors
    ///
    /// Returns an I/O error for length or payload reads,
    /// [`std::io::ErrorKind::InvalidData`] when the encoded length exceeds
    /// `max_len`, or [`std::io::ErrorKind::InvalidData`] when the payload
    /// is not valid UTF-8.
    fn read_string_with_u16_len_be(&mut self, max_len: usize) -> Result<String>;

    /// Reads a UTF-8 string with a little-endian `u16` byte-length prefix.
    ///
    /// # Parameters
    ///
    /// - `max_len`: Maximum accepted UTF-8 payload length in bytes.
    ///
    /// # Returns
    ///
    /// The decoded string.
    ///
    /// # Errors
    ///
    /// Returns an I/O error for length or payload reads,
    /// [`std::io::ErrorKind::InvalidData`] when the encoded length exceeds
    /// `max_len`, or [`std::io::ErrorKind::InvalidData`] when the payload
    /// is not valid UTF-8.
    fn read_string_with_u16_len_le(&mut self, max_len: usize) -> Result<String>;

    /// Reads a UTF-8 string with a runtime-order `u32` byte-length prefix.
    ///
    /// # Parameters
    ///
    /// - `byte_order`: Byte order used by the length prefix.
    /// - `max_len`: Maximum accepted UTF-8 payload length in bytes.
    ///
    /// # Returns
    ///
    /// The decoded string.
    ///
    /// # Errors
    ///
    /// Returns an I/O error for length or payload reads,
    /// [`std::io::ErrorKind::InvalidData`] when the encoded length exceeds
    /// `max_len`, or [`std::io::ErrorKind::InvalidData`] when the payload
    /// is not valid UTF-8.
    fn read_string_with_u32_len(&mut self, byte_order: ByteOrder, max_len: usize) -> Result<String>;

    /// Reads a UTF-8 string with a big-endian `u32` byte-length prefix.
    ///
    /// # Parameters
    ///
    /// - `max_len`: Maximum accepted UTF-8 payload length in bytes.
    ///
    /// # Returns
    ///
    /// The decoded string.
    ///
    /// # Errors
    ///
    /// Returns an I/O error for length or payload reads,
    /// [`std::io::ErrorKind::InvalidData`] when the encoded length exceeds
    /// `max_len`, or [`std::io::ErrorKind::InvalidData`] when the payload
    /// is not valid UTF-8.
    fn read_string_with_u32_len_be(&mut self, max_len: usize) -> Result<String>;

    /// Reads a UTF-8 string with a little-endian `u32` byte-length prefix.
    ///
    /// # Parameters
    ///
    /// - `max_len`: Maximum accepted UTF-8 payload length in bytes.
    ///
    /// # Returns
    ///
    /// The decoded string.
    ///
    /// # Errors
    ///
    /// Returns an I/O error for length or payload reads,
    /// [`std::io::ErrorKind::InvalidData`] when the encoded length exceeds
    /// `max_len`, or [`std::io::ErrorKind::InvalidData`] when the payload
    /// is not valid UTF-8.
    fn read_string_with_u32_len_le(&mut self, max_len: usize) -> Result<String>;
}

impl<T> StringReadExt for T
where
    T: Input<Item = u8> + ?Sized,
{
    #[inline]
    fn read_utf8_payload_into(&mut self, bytes: &mut Vec<u8>, len: usize, max_len: usize) -> Result<()> {
        read_utf8_payload_into_impl(self, bytes, len, max_len)
    }

    #[inline]
    fn read_utf8_payload(&mut self, len: usize, max_len: usize) -> Result<String> {
        read_utf8_payload_impl(self, len, max_len)
    }

    #[inline]
    fn read_utf8_string_uleb_usize(&mut self, max_len: usize) -> Result<String> {
        let len = self.read_uleb_usize_non_strict()?;
        read_utf8_payload_impl(self, len, max_len)
    }

    #[inline]
    fn read_utf8_string_uleb_usize_strict(&mut self, max_len: usize) -> Result<String> {
        let len = self.read_uleb_usize_strict()?;
        read_utf8_payload_impl(self, len, max_len)
    }

    #[inline]
    fn read_utf8_string_uleb_u64(&mut self, max_len: usize) -> Result<String> {
        let len = self.read_uleb_u64_non_strict()?;
        #[cfg(target_pointer_width = "64")]
        let len = len as usize;
        #[cfg(not(target_pointer_width = "64"))]
        let len = usize_from_u64_len(len)?;
        read_utf8_payload_impl(self, len, max_len)
    }

    #[inline]
    fn read_utf8_string_uleb_u64_strict(&mut self, max_len: usize) -> Result<String> {
        let len = self.read_uleb_u64_strict()?;
        #[cfg(target_pointer_width = "64")]
        let len = len as usize;
        #[cfg(not(target_pointer_width = "64"))]
        let len = usize_from_u64_len(len)?;
        read_utf8_payload_impl(self, len, max_len)
    }

    #[inline]
    fn read_string_with_u16_len(&mut self, byte_order: ByteOrder, max_len: usize) -> Result<String> {
        let len = usize::from(self.read_u16(byte_order)?);
        read_utf8_payload_impl(self, len, max_len)
    }

    #[inline]
    fn read_string_with_u16_len_be(&mut self, max_len: usize) -> Result<String> {
        let len = self.read_u16_be()? as usize;
        read_utf8_payload_impl(self, len, max_len)
    }

    #[inline]
    fn read_string_with_u16_len_le(&mut self, max_len: usize) -> Result<String> {
        let len = self.read_u16_le()? as usize;
        read_utf8_payload_impl(self, len, max_len)
    }

    #[inline]
    fn read_string_with_u32_len(&mut self, byte_order: ByteOrder, max_len: usize) -> Result<String> {
        let len = self.read_u32(byte_order)?;
        #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
        let len = len as usize;
        #[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
        let len = usize_from_u32_len(len)?;
        read_utf8_payload_impl(self, len, max_len)
    }

    #[inline]
    fn read_string_with_u32_len_be(&mut self, max_len: usize) -> Result<String> {
        let len = self.read_u32_be()?;
        #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
        let len = len as usize;
        #[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
        let len = usize_from_u32_len(len)?;
        read_utf8_payload_impl(self, len, max_len)
    }

    #[inline]
    fn read_string_with_u32_len_le(&mut self, max_len: usize) -> Result<String> {
        let len = self.read_u32_le()?;
        #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
        let len = len as usize;
        #[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
        let len = usize_from_u32_len(len)?;
        read_utf8_payload_impl(self, len, max_len)
    }
}
