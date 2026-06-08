// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use qubit_codec::{
    BufferedDecodeHooks,
    Codec,
    CodecDecodeError,
    DecodeAction,
    DecodeContext,
};

use super::stream_codec_decode_error::StreamCodecDecodeError;

/// Stream hook mapping codec-level decode errors to stream actions.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) struct StreamCodecBufferedDecodeHooks;

impl<C> BufferedDecodeHooks<C> for StreamCodecBufferedDecodeHooks
where
    C: Codec,
    C::DecodeError: StreamCodecDecodeError,
{
    type Error = CodecDecodeError<C::DecodeError>;

    #[inline(always)]
    fn handle_decode_error(
        &mut self,
        _codec: &C,
        error: C::DecodeError,
        context: DecodeContext,
    ) -> core::result::Result<DecodeAction<C::Value>, Self::Error> {
        if let Some(required_total) = error.required_total() {
            Ok(DecodeAction::NeedInput { required_total })
        } else {
            Err(CodecDecodeError::decode(error, context.input_index))
        }
    }

    #[inline(always)]
    fn invalid_input_index(
        &mut self,
        _codec: &C,
        index: usize,
        input_len: usize,
    ) -> Self::Error {
        CodecDecodeError::invalid_input_index(index, input_len)
    }

    #[inline(always)]
    fn invalid_output_index(
        &mut self,
        _codec: &C,
        index: usize,
        output_len: usize,
    ) -> Self::Error {
        CodecDecodeError::invalid_output_index(index, output_len)
    }
}
