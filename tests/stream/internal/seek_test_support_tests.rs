// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Result;
use std::io::Seek;
use std::io::SeekFrom;

/// Reads a fixed byte buffer while rejecting relative and end-relative seeks.
///
/// This reader supports absolute seeks so tests can verify that callers do not
/// depend on `SeekFrom::Current` or `SeekFrom::End`.
#[must_use = "the reader must be used by the operation under test"]
pub(crate) struct RejectingCurrentSeekReader {
    bytes: Vec<u8>,
    position: usize,
}

impl RejectingCurrentSeekReader {
    /// Creates a reader positioned at the beginning of `bytes`.
    #[inline]
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        Self { bytes, position: 0 }
    }
}

impl Read for RejectingCurrentSeekReader {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let remaining = self.bytes.len() - self.position;
        let count = remaining.min(buffer.len());
        buffer[..count].copy_from_slice(&self.bytes[self.position..self.position + count]);
        self.position += count;
        Ok(count)
    }
}

impl Seek for RejectingCurrentSeekReader {
    fn seek(&mut self, position: SeekFrom) -> Result<u64> {
        match position {
            SeekFrom::Start(position) => {
                self.position = usize::try_from(position)
                    .map_err(|_| Error::new(ErrorKind::InvalidInput, "seek position exceeds usize"))?;
                Ok(position)
            }
            SeekFrom::Current(_) => Err(Error::other("current seek rejected")),
            SeekFrom::End(_) => Err(Error::new(ErrorKind::Unsupported, "unsupported seek")),
        }
    }
}
