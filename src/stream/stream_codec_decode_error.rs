// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use qubit_codec::CodecDecodeErrorSignal;
use qubit_codec_binary::Leb128DecodeError;
use std::error::Error as StdError;
use std::io::ErrorKind;

/// Decode error behavior shared by codec stream decoders.
pub trait StreamCodecDecodeError:
    CodecDecodeErrorSignal + StdError + Send + Sync + 'static
{
    /// Returns the mapped I/O error kind for this codec decode error.
    fn io_error_kind(&self) -> ErrorKind;
}

impl StreamCodecDecodeError for core::convert::Infallible {
    #[inline(always)]
    fn io_error_kind(&self) -> ErrorKind {
        match *self {}
    }
}

impl StreamCodecDecodeError for Leb128DecodeError {
    #[inline(always)]
    fn io_error_kind(&self) -> ErrorKind {
        if Leb128DecodeError::is_incomplete(*self) {
            ErrorKind::UnexpectedEof
        } else {
            ErrorKind::InvalidData
        }
    }
}
