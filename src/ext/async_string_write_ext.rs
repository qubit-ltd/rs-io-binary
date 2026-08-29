// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Asynchronous length-prefixed UTF-8 writes.

use core::future::Future;
use std::io::Result;

use qubit_codec::ByteOrder;
use qubit_io::AsyncOutput;

use crate::AsyncBinaryWriteExt;
use crate::AsyncLeb128WriteExt;
use crate::util::checked_u16_len;
use crate::util::checked_u32_len;
use crate::util::checked_u64_len;
use crate::util::write_all_async;

/// Future-based length-prefixed UTF-8 writes.
///
/// # Cancellation safety
///
/// These writes are not cancellation safe. Dropping a pending future leaves
/// any already-written prefix or payload bytes in the output.
pub trait AsyncStringWriteExt: AsyncOutput<Item = u8> {
    /// Asynchronously writes a UTF-8 payload without a length prefix.
    ///
    /// # Parameters
    ///
    /// - `value`: String payload to write.
    ///
    /// # Returns
    ///
    /// Returns a future that completes after the payload has been written.
    ///
    /// # Errors
    ///
    /// Returns an output error, including a write-zero error when the output
    /// stops making progress.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future leaves
    /// any already-written payload prefix in the output.
    #[inline]
    fn write_utf8_payload_async<'a>(&'a mut self, value: &'a str) -> impl Future<Output = Result<()>> + 'a
    where
        Self: Unpin,
    {
        async move { write_all_async(self, value.as_bytes()).await }
    }

    /// Asynchronously writes a target-width LEB128-length UTF-8 string.
    ///
    /// # Parameters
    ///
    /// - `value`: String to length-prefix and write.
    ///
    /// # Returns
    ///
    /// Returns a future that completes after the prefix and payload are
    /// written.
    ///
    /// # Errors
    ///
    /// Returns an output error, including a write-zero error when the output
    /// stops making progress.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future leaves
    /// any already-written prefix or payload bytes in the output.
    #[inline]
    fn write_utf8_string_uleb_usize_async<'a>(&'a mut self, value: &'a str) -> impl Future<Output = Result<()>> + 'a
    where
        Self: Unpin,
    {
        async move {
            self.write_uleb_usize_async(value.len()).await?;
            write_all_async(self, value.as_bytes()).await
        }
    }

    /// Asynchronously writes a `u64` LEB128-length UTF-8 string.
    ///
    /// # Parameters
    ///
    /// - `value`: String to length-prefix and write.
    ///
    /// # Returns
    ///
    /// Returns a future that completes after the prefix and payload are
    /// written.
    ///
    /// # Errors
    ///
    /// Returns an invalid-input error when the length cannot fit in `u64`, or
    /// an output error while writing.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future leaves
    /// any already-written prefix or payload bytes in the output.
    #[inline]
    fn write_utf8_string_uleb_u64_async<'a>(&'a mut self, value: &'a str) -> impl Future<Output = Result<()>> + 'a
    where
        Self: Unpin,
    {
        async move {
            let len = checked_u64_len(value.len())?;
            self.write_uleb_u64_async(len).await?;
            write_all_async(self, value.as_bytes()).await
        }
    }

    /// Asynchronously writes a runtime-order `u16`-length UTF-8 string.
    ///
    /// # Parameters
    ///
    /// - `value`: String to length-prefix and write.
    /// - `byte_order`: Byte order used to encode the length.
    ///
    /// # Returns
    ///
    /// Returns a future that completes after the prefix and payload are
    /// written.
    ///
    /// # Errors
    ///
    /// Returns an invalid-input error when the length exceeds `u16::MAX`, or
    /// an output error while writing.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future leaves
    /// any already-written prefix or payload bytes in the output.
    #[inline]
    fn write_string_with_u16_len_async<'a>(
        &'a mut self,
        value: &'a str,
        byte_order: ByteOrder,
    ) -> impl Future<Output = Result<()>> + 'a
    where
        Self: Unpin,
    {
        async move {
            let len = checked_u16_len(value.len())?;
            self.write_u16_async(len, byte_order).await?;
            write_all_async(self, value.as_bytes()).await
        }
    }

    /// Asynchronously writes a big-endian `u16`-length UTF-8 string.
    ///
    /// # Parameters
    ///
    /// - `value`: String to length-prefix and write.
    ///
    /// # Returns
    ///
    /// Returns a future that completes after the prefix and payload are
    /// written.
    ///
    /// # Errors
    ///
    /// Returns an invalid-input error when the length exceeds `u16::MAX`, or
    /// an output error while writing.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future leaves
    /// any already-written prefix or payload bytes in the output.
    #[inline]
    fn write_string_with_u16_len_be_async<'a>(&'a mut self, value: &'a str) -> impl Future<Output = Result<()>> + 'a
    where
        Self: Unpin,
    {
        async move {
            let len = checked_u16_len(value.len())?;
            self.write_u16_be_async(len).await?;
            write_all_async(self, value.as_bytes()).await
        }
    }

    /// Asynchronously writes a little-endian `u16`-length UTF-8 string.
    ///
    /// # Parameters
    ///
    /// - `value`: String to length-prefix and write.
    ///
    /// # Returns
    ///
    /// Returns a future that completes after the prefix and payload are
    /// written.
    ///
    /// # Errors
    ///
    /// Returns an invalid-input error when the length exceeds `u16::MAX`, or
    /// an output error while writing.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future leaves
    /// any already-written prefix or payload bytes in the output.
    #[inline]
    fn write_string_with_u16_len_le_async<'a>(&'a mut self, value: &'a str) -> impl Future<Output = Result<()>> + 'a
    where
        Self: Unpin,
    {
        async move {
            let len = checked_u16_len(value.len())?;
            self.write_u16_le_async(len).await?;
            write_all_async(self, value.as_bytes()).await
        }
    }

    /// Asynchronously writes a runtime-order `u32`-length UTF-8 string.
    ///
    /// # Parameters
    ///
    /// - `value`: String to length-prefix and write.
    /// - `byte_order`: Byte order used to encode the length.
    ///
    /// # Returns
    ///
    /// Returns a future that completes after the prefix and payload are
    /// written.
    ///
    /// # Errors
    ///
    /// Returns an invalid-input error when the length exceeds `u32::MAX`, or
    /// an output error while writing.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future leaves
    /// any already-written prefix or payload bytes in the output.
    #[inline]
    fn write_string_with_u32_len_async<'a>(
        &'a mut self,
        value: &'a str,
        byte_order: ByteOrder,
    ) -> impl Future<Output = Result<()>> + 'a
    where
        Self: Unpin,
    {
        async move {
            let len = checked_u32_len(value.len())?;
            self.write_u32_async(len, byte_order).await?;
            write_all_async(self, value.as_bytes()).await
        }
    }

    /// Asynchronously writes a big-endian `u32`-length UTF-8 string.
    ///
    /// # Parameters
    ///
    /// - `value`: String to length-prefix and write.
    ///
    /// # Returns
    ///
    /// Returns a future that completes after the prefix and payload are
    /// written.
    ///
    /// # Errors
    ///
    /// Returns an invalid-input error when the length exceeds `u32::MAX`, or
    /// an output error while writing.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future leaves
    /// any already-written prefix or payload bytes in the output.
    #[inline]
    fn write_string_with_u32_len_be_async<'a>(&'a mut self, value: &'a str) -> impl Future<Output = Result<()>> + 'a
    where
        Self: Unpin,
    {
        async move {
            let len = checked_u32_len(value.len())?;
            self.write_u32_be_async(len).await?;
            write_all_async(self, value.as_bytes()).await
        }
    }

    /// Asynchronously writes a little-endian `u32`-length UTF-8 string.
    ///
    /// # Parameters
    ///
    /// - `value`: String to length-prefix and write.
    ///
    /// # Returns
    ///
    /// Returns a future that completes after the prefix and payload are
    /// written.
    ///
    /// # Errors
    ///
    /// Returns an invalid-input error when the length exceeds `u32::MAX`, or
    /// an output error while writing.
    ///
    /// # Cancellation safety
    ///
    /// This operation is not cancellation safe; dropping the future leaves
    /// any already-written prefix or payload bytes in the output.
    #[inline]
    fn write_string_with_u32_len_le_async<'a>(&'a mut self, value: &'a str) -> impl Future<Output = Result<()>> + 'a
    where
        Self: Unpin,
    {
        async move {
            let len = checked_u32_len(value.len())?;
            self.write_u32_le_async(len).await?;
            write_all_async(self, value.as_bytes()).await
        }
    }
}

impl<W> AsyncStringWriteExt for W where W: AsyncOutput<Item = u8> + ?Sized {}
