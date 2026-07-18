// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Asynchronous length-prefixed UTF-8 writes.

use std::io::Result;

use qubit_io::AsyncOutput;

use crate::util::{
    checked_u16_len,
    checked_u32_len,
    checked_u64_len,
    write_all_async,
};
use crate::{
    AsyncBinaryWriteExt,
    AsyncLeb128WriteExt,
    ByteOrder,
};

/// Future-based length-prefixed UTF-8 writes.
#[allow(async_fn_in_trait)]
pub trait AsyncStringWriteExt: AsyncOutput<Item = u8> {
    /// Asynchronously writes a UTF-8 payload without a length prefix.
    async fn write_utf8_payload_async(&mut self, value: &str) -> Result<()>
    where
        Self: Unpin,
    {
        write_all_async(self, value.as_bytes()).await
    }

    /// Asynchronously writes a target-width LEB128-length UTF-8 string.
    async fn write_utf8_string_uleb_async(&mut self, value: &str) -> Result<()>
    where
        Self: Unpin,
    {
        self.write_uleb_usize_async(value.len()).await?;
        write_all_async(self, value.as_bytes()).await
    }

    /// Asynchronously writes a `u64` LEB128-length UTF-8 string.
    async fn write_utf8_string_uleb_u64_async(
        &mut self,
        value: &str,
    ) -> Result<()>
    where
        Self: Unpin,
    {
        let len = checked_u64_len(value.len())?;
        self.write_uleb_u64_async(len).await?;
        write_all_async(self, value.as_bytes()).await
    }

    /// Asynchronously writes a runtime-order `u16`-length UTF-8 string.
    async fn write_utf8_string_u16_async(
        &mut self,
        value: &str,
        byte_order: ByteOrder,
    ) -> Result<()>
    where
        Self: Unpin,
    {
        let len = checked_u16_len(value.len())?;
        self.write_u16_async(len, byte_order).await?;
        write_all_async(self, value.as_bytes()).await
    }

    /// Asynchronously writes a big-endian `u16`-length UTF-8 string.
    async fn write_utf8_string_u16_be_async(
        &mut self,
        value: &str,
    ) -> Result<()>
    where
        Self: Unpin,
    {
        let len = checked_u16_len(value.len())?;
        self.write_u16_be_async(len).await?;
        write_all_async(self, value.as_bytes()).await
    }

    /// Asynchronously writes a little-endian `u16`-length UTF-8 string.
    async fn write_utf8_string_u16_le_async(
        &mut self,
        value: &str,
    ) -> Result<()>
    where
        Self: Unpin,
    {
        let len = checked_u16_len(value.len())?;
        self.write_u16_le_async(len).await?;
        write_all_async(self, value.as_bytes()).await
    }

    /// Asynchronously writes a runtime-order `u32`-length UTF-8 string.
    async fn write_utf8_string_u32_async(
        &mut self,
        value: &str,
        byte_order: ByteOrder,
    ) -> Result<()>
    where
        Self: Unpin,
    {
        let len = checked_u32_len(value.len())?;
        self.write_u32_async(len, byte_order).await?;
        write_all_async(self, value.as_bytes()).await
    }

    /// Asynchronously writes a big-endian `u32`-length UTF-8 string.
    async fn write_utf8_string_u32_be_async(
        &mut self,
        value: &str,
    ) -> Result<()>
    where
        Self: Unpin,
    {
        let len = checked_u32_len(value.len())?;
        self.write_u32_be_async(len).await?;
        write_all_async(self, value.as_bytes()).await
    }

    /// Asynchronously writes a little-endian `u32`-length UTF-8 string.
    async fn write_utf8_string_u32_le_async(
        &mut self,
        value: &str,
    ) -> Result<()>
    where
        Self: Unpin,
    {
        let len = checked_u32_len(value.len())?;
        self.write_u32_le_async(len).await?;
        write_all_async(self, value.as_bytes()).await
    }
}

impl<W> AsyncStringWriteExt for W where W: AsyncOutput<Item = u8> + ?Sized {}
