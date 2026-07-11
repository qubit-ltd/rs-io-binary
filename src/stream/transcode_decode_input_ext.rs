// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
use std::io::{
    Error,
    Result,
};

use qubit_codec::{
    Codec,
    TranscodeDecodeInput,
};
use qubit_io::Input;

use super::stream_codec_decode_error::StreamCodecDecodeError;

/// Codec-oriented helpers for [`TranscodeDecodeInput`].
pub trait TranscodeDecodeInputExt<I> {
    /// Decodes one value through the underlying buffered input.
    fn read_decoded<C>(&mut self) -> Result<C::Value>
    where
        I: Input,
        C: Codec<Unit = I::Item> + Default,
        C::Value: Default,
        C::DecodeError: StreamCodecDecodeError;
}

impl<I> TranscodeDecodeInputExt<I> for TranscodeDecodeInput<I>
where
    I: Input,
    I::Item: Copy + Default,
{
    fn read_decoded<C>(&mut self) -> Result<C::Value>
    where
        C: Codec<Unit = I::Item> + Default,
        C::Value: Default,
        C::DecodeError: StreamCodecDecodeError,
    {
        let mut codec = C::default();
        self.read_decoded_with(&mut codec, |source| {
            Error::new(source.io_error_kind(), source)
        })
    }
}

/// Decodes one value with caller-provided lifecycle scratch storage.
///
/// # Parameters
///
/// - `input`: Buffered unit input used for decoding.
/// - `lifecycle_scratch`: Storage for codec reset and finish values.
///
/// # Returns
///
/// Returns the decoded codec value.
///
/// # Errors
///
/// Returns I/O errors from `input` or codec decode errors mapped through
/// [`StreamCodecDecodeError::io_error_kind`].
pub(super) fn read_decoded_with_scratch<I, C>(
    input: &mut TranscodeDecodeInput<I>,
    lifecycle_scratch: &mut [C::Value],
) -> Result<C::Value>
where
    I: Input,
    I::Item: Copy + Default,
    C: Codec<Unit = I::Item> + Default,
    C::DecodeError: StreamCodecDecodeError,
{
    let mut codec = C::default();
    input.read_decoded_with_scratch(&mut codec, lifecycle_scratch, |source| {
        Error::new(source.io_error_kind(), source)
    })
}
