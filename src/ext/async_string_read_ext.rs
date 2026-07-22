// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow source-test-pair
//! Asynchronous length-prefixed UTF-8 reads.

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
#[allow(async_fn_in_trait)]
pub trait AsyncStringReadExt: AsyncInput<Item = u8> {
    /// Asynchronously reads an already length-delimited UTF-8 payload.
    async fn read_utf8_payload_async(
        &mut self,
        len: usize,
        max_len: usize,
    ) -> Result<String>
    where
        Self: Unpin,
    {
        read_utf8_payload_async(self, len, max_len).await
    }

    /// Asynchronously reads a target-width LEB128-length UTF-8 string.
    async fn read_utf8_string_uleb_async(
        &mut self,
        max_len: usize,
    ) -> Result<String>
    where
        Self: Unpin,
    {
        let len = self.read_uleb_usize_async().await?;
        read_utf8_payload_async(self, len, max_len).await
    }

    /// Asynchronously reads a canonical target-width LEB128-length string.
    async fn read_utf8_string_uleb_strict_async(
        &mut self,
        max_len: usize,
    ) -> Result<String>
    where
        Self: Unpin,
    {
        let len = self.read_uleb_usize_strict_async().await?;
        read_utf8_payload_async(self, len, max_len).await
    }

    /// Asynchronously reads a `u64` LEB128-length UTF-8 string.
    async fn read_utf8_string_uleb_u64_async(
        &mut self,
        max_len: usize,
    ) -> Result<String>
    where
        Self: Unpin,
    {
        let len = self.read_uleb_u64_async().await?;
        #[cfg(target_pointer_width = "64")]
        let len = len as usize;
        #[cfg(not(target_pointer_width = "64"))]
        let len = usize_from_u64_len(len)?;
        read_utf8_payload_async(self, len, max_len).await
    }

    /// Asynchronously reads a canonical `u64` LEB128-length UTF-8 string.
    async fn read_utf8_string_uleb_u64_strict_async(
        &mut self,
        max_len: usize,
    ) -> Result<String>
    where
        Self: Unpin,
    {
        let len = self.read_uleb_u64_strict_async().await?;
        #[cfg(target_pointer_width = "64")]
        let len = len as usize;
        #[cfg(not(target_pointer_width = "64"))]
        let len = usize_from_u64_len(len)?;
        read_utf8_payload_async(self, len, max_len).await
    }

    /// Asynchronously reads a runtime-order `u16`-length UTF-8 string.
    async fn read_utf8_string_u16_async(
        &mut self,
        byte_order: ByteOrder,
        max_len: usize,
    ) -> Result<String>
    where
        Self: Unpin,
    {
        let len = usize::from(self.read_u16_async(byte_order).await?);
        read_utf8_payload_async(self, len, max_len).await
    }

    /// Asynchronously reads a big-endian `u16`-length UTF-8 string.
    async fn read_utf8_string_u16_be_async(
        &mut self,
        max_len: usize,
    ) -> Result<String>
    where
        Self: Unpin,
    {
        let len = usize::from(self.read_u16_be_async().await?);
        read_utf8_payload_async(self, len, max_len).await
    }

    /// Asynchronously reads a little-endian `u16`-length UTF-8 string.
    async fn read_utf8_string_u16_le_async(
        &mut self,
        max_len: usize,
    ) -> Result<String>
    where
        Self: Unpin,
    {
        let len = usize::from(self.read_u16_le_async().await?);
        read_utf8_payload_async(self, len, max_len).await
    }

    /// Asynchronously reads a runtime-order `u32`-length UTF-8 string.
    async fn read_utf8_string_u32_async(
        &mut self,
        byte_order: ByteOrder,
        max_len: usize,
    ) -> Result<String>
    where
        Self: Unpin,
    {
        let len = self.read_u32_async(byte_order).await?;
        #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
        let len = len as usize;
        #[cfg(not(any(
            target_pointer_width = "32",
            target_pointer_width = "64"
        )))]
        let len = usize_from_u32_len(len)?;
        read_utf8_payload_async(self, len, max_len).await
    }

    /// Asynchronously reads a big-endian `u32`-length UTF-8 string.
    async fn read_utf8_string_u32_be_async(
        &mut self,
        max_len: usize,
    ) -> Result<String>
    where
        Self: Unpin,
    {
        let len = self.read_u32_be_async().await?;
        #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
        let len = len as usize;
        #[cfg(not(any(
            target_pointer_width = "32",
            target_pointer_width = "64"
        )))]
        let len = usize_from_u32_len(len)?;
        read_utf8_payload_async(self, len, max_len).await
    }

    /// Asynchronously reads a little-endian `u32`-length UTF-8 string.
    async fn read_utf8_string_u32_le_async(
        &mut self,
        max_len: usize,
    ) -> Result<String>
    where
        Self: Unpin,
    {
        let len = self.read_u32_le_async().await?;
        #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
        let len = len as usize;
        #[cfg(not(any(
            target_pointer_width = "32",
            target_pointer_width = "64"
        )))]
        let len = usize_from_u32_len(len)?;
        read_utf8_payload_async(self, len, max_len).await
    }
}

impl<R> AsyncStringReadExt for R where R: AsyncInput<Item = u8> + ?Sized {}
