// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use std::io::Error;
use std::io::Result;
use std::io::Write;

/// Accepts a fixed byte budget, then fails every subsequent nonempty write.
pub(crate) struct PrefixErrorOutput {
    bytes: Vec<u8>,
    limit: usize,
}

impl PrefixErrorOutput {
    pub(crate) fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
        }
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl Write for PrefixErrorOutput {
    fn write(&mut self, bytes: &[u8]) -> Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        let remaining = self.limit.saturating_sub(self.bytes.len());
        if remaining == 0 {
            return Err(Error::other("scripted payload write failure"));
        }
        let accepted = remaining.min(bytes.len());
        self.bytes.extend_from_slice(&bytes[..accepted]);
        Ok(accepted)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
