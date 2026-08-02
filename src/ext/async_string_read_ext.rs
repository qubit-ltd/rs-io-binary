// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Asynchronous length-prefixed UTF-8 reads.

use core::future::Future;
use std::io::Result;

use qubit_codec::ByteOrder;
use qubit_io::AsyncInput;

use crate::util::read_utf8_payload_async;
#[cfg(not(any(
    target_pointer_width = "32",
    target_pointer_width = "64"
)))]
use crate::util::usize_from_u32_len;
#[cfg(not(target_pointer_width = "64"))]
use crate::util::usize_from_u64_len;
use crate::{
    AsyncBinaryReadExt,
    AsyncLeb128ReadExt,
};

/// Future-based length-prefixed UTF-8 reads.
///
/// Every method returns a [`Send`] future when the input itself is [`Send`].
///
/// # Cancellation safety
///
/// These reads are not cancellation safe. Dropping a pending future leaves
/// bytes already consumed from the prefix or payload consumed.
pub trait AsyncStringReadExt: AsyncInput<Item = u8> {
    /// Asynchronously reads an already length-delimited UTF-8 payload.
    ///
    /// # Parameters
    ///
    /// - `len`: Encoded payload length in bytes.
    /// - `max_len`: Maximum accepted payload length in bytes.
    ///
    /// # Returns
    ///
    /// Returns the decoded string.
    ///
    /// # Errors
    ///
    /// Returns an input or allocation error, or an invalid-data error when
    /// `len` exceeds `max_len` or the payload is not valid UTF-8.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future retains
    /// any bytes already consumed from the input.
    #[inline]
    fn read_utf8_payload_async(
        &mut self,
        len: usize,
        max_len: usize,
    ) -> impl Future<Output = Result<String>> + Send + '_
    where
        Self: Send + Unpin,
    {
        async move { read_utf8_payload_async(self, len, max_len).await }
    }

    /// Asynchronously reads a target-width LEB128-length UTF-8 string.
    ///
    /// # Parameters
    ///
    /// - `max_len`: Maximum accepted payload length in bytes.
    ///
    /// # Returns
    ///
    /// Returns the decoded string.
    ///
    /// # Errors
    ///
    /// Returns an input or allocation error, or an invalid-data error for a
    /// malformed length, an excessive length, or invalid UTF-8.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future retains
    /// any prefix or payload bytes already consumed.
    #[inline]
    fn read_utf8_string_uleb_usize_async(
        &mut self,
        max_len: usize,
    ) -> impl Future<Output = Result<String>> + Send + '_
    where
        Self: Send + Unpin,
    {
        async move {
            let len = self.read_uleb_usize_non_strict_async().await?;
            read_utf8_payload_async(self, len, max_len).await
        }
    }

    /// Asynchronously reads a canonical target-width LEB128-length string.
    ///
    /// # Parameters
    ///
    /// - `max_len`: Maximum accepted payload length in bytes.
    ///
    /// # Returns
    ///
    /// Returns the decoded string.
    ///
    /// # Errors
    ///
    /// Returns an input or allocation error, or an invalid-data error for a
    /// non-canonical length, an excessive length, or invalid UTF-8.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future retains
    /// any prefix or payload bytes already consumed.
    #[inline]
    fn read_utf8_string_uleb_usize_strict_async(
        &mut self,
        max_len: usize,
    ) -> impl Future<Output = Result<String>> + Send + '_
    where
        Self: Send + Unpin,
    {
        async move {
            let len = self.read_uleb_usize_strict_async().await?;
            read_utf8_payload_async(self, len, max_len).await
        }
    }

    /// Asynchronously reads a `u64` LEB128-length UTF-8 string.
    ///
    /// # Parameters
    ///
    /// - `max_len`: Maximum accepted payload length in bytes.
    ///
    /// # Returns
    ///
    /// Returns the decoded string.
    ///
    /// # Errors
    ///
    /// Returns an input, conversion, or allocation error, or an invalid-data
    /// error for a malformed or excessive length or invalid UTF-8.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future retains
    /// any prefix or payload bytes already consumed.
    #[inline]
    fn read_utf8_string_uleb_u64_async(
        &mut self,
        max_len: usize,
    ) -> impl Future<Output = Result<String>> + Send + '_
    where
        Self: Send + Unpin,
    {
        async move {
            let len = self.read_uleb_u64_non_strict_async().await?;
            #[cfg(target_pointer_width = "64")]
            let len = len as usize;
            #[cfg(not(target_pointer_width = "64"))]
            let len = usize_from_u64_len(len)?;
            read_utf8_payload_async(self, len, max_len).await
        }
    }

    /// Asynchronously reads a canonical `u64` LEB128-length UTF-8 string.
    ///
    /// # Parameters
    ///
    /// - `max_len`: Maximum accepted payload length in bytes.
    ///
    /// # Returns
    ///
    /// Returns the decoded string.
    ///
    /// # Errors
    ///
    /// Returns an input, conversion, or allocation error, or an invalid-data
    /// error for a non-canonical or excessive length or invalid UTF-8.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future retains
    /// any prefix or payload bytes already consumed.
    #[inline]
    fn read_utf8_string_uleb_u64_strict_async(
        &mut self,
        max_len: usize,
    ) -> impl Future<Output = Result<String>> + Send + '_
    where
        Self: Send + Unpin,
    {
        async move {
            let len = self.read_uleb_u64_strict_async().await?;
            #[cfg(target_pointer_width = "64")]
            let len = len as usize;
            #[cfg(not(target_pointer_width = "64"))]
            let len = usize_from_u64_len(len)?;
            read_utf8_payload_async(self, len, max_len).await
        }
    }

    /// Asynchronously reads a runtime-order `u16`-length UTF-8 string.
    ///
    /// # Parameters
    ///
    /// - `byte_order`: Byte order used to decode the length.
    /// - `max_len`: Maximum accepted payload length in bytes.
    ///
    /// # Returns
    ///
    /// Returns the decoded string.
    ///
    /// # Errors
    ///
    /// Returns an input or allocation error, or an invalid-data error for an
    /// excessive length or invalid UTF-8.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future retains
    /// any prefix or payload bytes already consumed.
    #[inline]
    fn read_string_with_u16_len_async(
        &mut self,
        byte_order: ByteOrder,
        max_len: usize,
    ) -> impl Future<Output = Result<String>> + Send + '_
    where
        Self: Send + Unpin,
    {
        async move {
            let len = usize::from(self.read_u16_async(byte_order).await?);
            read_utf8_payload_async(self, len, max_len).await
        }
    }

    /// Asynchronously reads a big-endian `u16`-length UTF-8 string.
    ///
    /// # Parameters
    ///
    /// - `max_len`: Maximum accepted payload length in bytes.
    ///
    /// # Returns
    ///
    /// Returns the decoded string.
    ///
    /// # Errors
    ///
    /// Returns an input or allocation error, or an invalid-data error for an
    /// excessive length or invalid UTF-8.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future retains
    /// any prefix or payload bytes already consumed.
    #[inline]
    fn read_string_with_u16_len_be_async(
        &mut self,
        max_len: usize,
    ) -> impl Future<Output = Result<String>> + Send + '_
    where
        Self: Send + Unpin,
    {
        async move {
            let len = usize::from(self.read_u16_be_async().await?);
            read_utf8_payload_async(self, len, max_len).await
        }
    }

    /// Asynchronously reads a little-endian `u16`-length UTF-8 string.
    ///
    /// # Parameters
    ///
    /// - `max_len`: Maximum accepted payload length in bytes.
    ///
    /// # Returns
    ///
    /// Returns the decoded string.
    ///
    /// # Errors
    ///
    /// Returns an input or allocation error, or an invalid-data error for an
    /// excessive length or invalid UTF-8.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future retains
    /// any prefix or payload bytes already consumed.
    #[inline]
    fn read_string_with_u16_len_le_async(
        &mut self,
        max_len: usize,
    ) -> impl Future<Output = Result<String>> + Send + '_
    where
        Self: Send + Unpin,
    {
        async move {
            let len = usize::from(self.read_u16_le_async().await?);
            read_utf8_payload_async(self, len, max_len).await
        }
    }

    /// Asynchronously reads a runtime-order `u32`-length UTF-8 string.
    ///
    /// # Parameters
    ///
    /// - `byte_order`: Byte order used to decode the length.
    /// - `max_len`: Maximum accepted payload length in bytes.
    ///
    /// # Returns
    ///
    /// Returns the decoded string.
    ///
    /// # Errors
    ///
    /// Returns an input, conversion, or allocation error, or an invalid-data
    /// error for an excessive length or invalid UTF-8.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future retains
    /// any prefix or payload bytes already consumed.
    #[inline]
    fn read_string_with_u32_len_async(
        &mut self,
        byte_order: ByteOrder,
        max_len: usize,
    ) -> impl Future<Output = Result<String>> + Send + '_
    where
        Self: Send + Unpin,
    {
        async move {
            let len = self.read_u32_async(byte_order).await?;
            #[cfg(any(
                target_pointer_width = "32",
                target_pointer_width = "64"
            ))]
            let len = len as usize;
            #[cfg(not(any(
                target_pointer_width = "32",
                target_pointer_width = "64"
            )))]
            let len = usize_from_u32_len(len)?;
            read_utf8_payload_async(self, len, max_len).await
        }
    }

    /// Asynchronously reads a big-endian `u32`-length UTF-8 string.
    ///
    /// # Parameters
    ///
    /// - `max_len`: Maximum accepted payload length in bytes.
    ///
    /// # Returns
    ///
    /// Returns the decoded string.
    ///
    /// # Errors
    ///
    /// Returns an input, conversion, or allocation error, or an invalid-data
    /// error for an excessive length or invalid UTF-8.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future retains
    /// any prefix or payload bytes already consumed.
    #[inline]
    fn read_string_with_u32_len_be_async(
        &mut self,
        max_len: usize,
    ) -> impl Future<Output = Result<String>> + Send + '_
    where
        Self: Send + Unpin,
    {
        async move {
            let len = self.read_u32_be_async().await?;
            #[cfg(any(
                target_pointer_width = "32",
                target_pointer_width = "64"
            ))]
            let len = len as usize;
            #[cfg(not(any(
                target_pointer_width = "32",
                target_pointer_width = "64"
            )))]
            let len = usize_from_u32_len(len)?;
            read_utf8_payload_async(self, len, max_len).await
        }
    }

    /// Asynchronously reads a little-endian `u32`-length UTF-8 string.
    ///
    /// # Parameters
    ///
    /// - `max_len`: Maximum accepted payload length in bytes.
    ///
    /// # Returns
    ///
    /// Returns the decoded string.
    ///
    /// # Errors
    ///
    /// Returns an input, conversion, or allocation error, or an invalid-data
    /// error for an excessive length or invalid UTF-8.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future retains
    /// any prefix or payload bytes already consumed.
    #[inline]
    fn read_string_with_u32_len_le_async(
        &mut self,
        max_len: usize,
    ) -> impl Future<Output = Result<String>> + Send + '_
    where
        Self: Send + Unpin,
    {
        async move {
            let len = self.read_u32_le_async().await?;
            #[cfg(any(
                target_pointer_width = "32",
                target_pointer_width = "64"
            ))]
            let len = len as usize;
            #[cfg(not(any(
                target_pointer_width = "32",
                target_pointer_width = "64"
            )))]
            let len = usize_from_u32_len(len)?;
            read_utf8_payload_async(self, len, max_len).await
        }
    }
}

impl<R> AsyncStringReadExt for R where R: AsyncInput<Item = u8> + ?Sized {}
