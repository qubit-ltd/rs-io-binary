// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use std::io::{
    Error,
    ErrorKind,
    Read,
    Result,
};

use qubit_codec::{
    BufferedDecodeEngine,
    BufferedDecodeInput,
    Codec,
};

use super::stream_codec_buffered_decode_hooks::StreamCodecBufferedDecodeHooks;
use super::stream_codec_decode_error::{
    StreamCodecDecodeError,
    consumed_from_codec_decode_error,
    map_codec_decode_error,
};

/// Codec-oriented helpers for [`BufferedDecodeInput`].
pub(crate) trait BufferedDecodeInputExt<R> {
    /// Consumes this buffered input and returns the wrapped input object.
    #[must_use]
    fn into_inner(self) -> R;

    /// Decodes one value through the shared buffered decode driver.
    fn read_decoded<C>(&mut self) -> Result<C::Value>
    where
        R: Read,
        C: Codec<Unit = u8> + Default,
        C::Value: Copy + Default,
        C::DecodeError: StreamCodecDecodeError;
}

impl<R> BufferedDecodeInputExt<R> for BufferedDecodeInput<R>
where
    R: Read,
{
    #[inline]
    fn into_inner(self) -> R {
        let (inner, _) = self.into_parts();
        inner
    }

    #[inline(always)]
    fn read_decoded<C>(&mut self) -> Result<C::Value>
    where
        R: Read,
        C: Codec<Unit = u8> + Default,
        C::Value: Copy + Default,
        C::DecodeError: StreamCodecDecodeError,
    {
        let mut decoder = BufferedDecodeEngine::new(
            C::default(),
            StreamCodecBufferedDecodeHooks,
        );
        let mut consumed_on_error = None;
        let mut map_error = |error| {
            consumed_on_error =
                consumed_from_codec_decode_error::<C::DecodeError>(&error);
            map_codec_decode_error(error)
        };
        let mut output = [C::Value::default(); 1];
        let result = unsafe {
            // SAFETY: The one-value output range is valid.
            self.decode_into_unchecked(
                &mut decoder,
                &mut map_error,
                &mut output,
                0,
                1,
            )
        };
        let read = match result {
            Ok(read) => read,
            Err(error) => {
                if let Some(consumed) = consumed_on_error {
                    self.consume_units(consumed.get());
                }
                return Err(error);
            }
        };
        if read == 1 {
            Ok(output[0])
        } else {
            self.consume_available();
            Err(Error::new(
                ErrorKind::UnexpectedEof,
                "failed to decode complete value",
            ))
        }
    }
}
