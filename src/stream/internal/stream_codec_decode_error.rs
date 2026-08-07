// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Conversion helpers for codec decode failures exposed as I/O errors.

use std::error::Error as StdError;
use std::io::ErrorKind;

use qubit_codec_binary::Leb128DecodeError;

/// Decode error behavior shared by codec stream decoders.
pub(crate) trait StreamCodecDecodeError:
    StdError + Send + Sync + 'static
{
    /// Returns the mapped I/O error kind for this codec decode error.
    ///
    /// # Returns
    ///
    /// Returns the I/O category exposed by stream adapters.
    fn io_error_kind(&self) -> ErrorKind;
}

impl StreamCodecDecodeError for core::convert::Infallible {
    #[inline(always)]
    fn io_error_kind(&self) -> ErrorKind {
        match *self {}
    }
}

impl StreamCodecDecodeError for Leb128DecodeError {
    #[inline]
    fn io_error_kind(&self) -> ErrorKind {
        if Leb128DecodeError::is_incomplete(*self) {
            ErrorKind::UnexpectedEof
        } else {
            ErrorKind::InvalidData
        }
    }
}
