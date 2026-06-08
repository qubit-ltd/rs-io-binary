// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use core::num::NonZeroUsize;
use qubit_codec_binary::Leb128DecodeError;
use std::error::Error as StdError;
use std::io::ErrorKind;

/// Decode error behavior shared by codec stream decoders.
pub(crate) trait StreamCodecDecodeError:
    StdError + Send + Sync + 'static
{
    /// Returns the total required input units when the decode is incomplete.
    fn required_total(&self) -> Option<usize>;

    /// Returns invalid units that should be consumed before reporting an error.
    fn consumed(&self) -> Option<NonZeroUsize>;

    /// Returns the mapped I/O error kind for this codec decode error.
    fn io_error_kind(&self) -> ErrorKind;
}

impl StreamCodecDecodeError for core::convert::Infallible {
    #[inline(always)]
    fn required_total(&self) -> Option<usize> {
        match *self {}
    }

    #[inline(always)]
    fn consumed(&self) -> Option<NonZeroUsize> {
        match *self {}
    }

    #[inline(always)]
    fn io_error_kind(&self) -> ErrorKind {
        match *self {}
    }
}

impl StreamCodecDecodeError for Leb128DecodeError {
    #[inline(always)]
    fn required_total(&self) -> Option<usize> {
        Leb128DecodeError::required(*self).map(NonZeroUsize::get)
    }

    #[inline(always)]
    fn consumed(&self) -> Option<NonZeroUsize> {
        Leb128DecodeError::consumed(*self)
    }

    #[inline(always)]
    fn io_error_kind(&self) -> ErrorKind {
        if Leb128DecodeError::is_incomplete(*self) {
            ErrorKind::UnexpectedEof
        } else {
            ErrorKind::InvalidData
        }
    }
}
