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
    /// Strictly decodes one value through the underlying buffered input.
    ///
    /// The codec's decode reset and finish phases must not declare output.
    /// Codecs with lifecycle output should use
    /// [`TranscodeDecodeInput::read_decoded_lifecycle_with`] directly.
    ///
    /// # Errors
    ///
    /// Returns I/O errors from the input or codec. Returns
    /// [`std::io::ErrorKind::Unsupported`] before reading input when the codec
    /// declares decode lifecycle output.
    fn read_decoded<C>(&mut self) -> Result<C::Value>
    where
        I: Input,
        C: Codec<Unit = I::Item> + Default,
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
        C::DecodeError: StreamCodecDecodeError,
    {
        let mut codec = C::default();
        self.read_decoded_with(&mut codec, |source| {
            Error::new(source.io_error_kind(), source)
        })
    }
}
