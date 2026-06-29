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
