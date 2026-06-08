// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use qubit_codec::{
    BufferedDecodeEngine,
    BufferedTranscoder,
    CapacityError,
    Codec,
    CodecDecodeError,
    FinishError,
    TranscodeProgress,
};

use super::stream_codec_buffered_decode_hooks::StreamCodecBufferedDecodeHooks;
use super::stream_codec_decode_error::StreamCodecDecodeError;

/// Decoder wrapper using stream-oriented hooks for codec refill behavior.
pub(crate) struct StreamCodecBufferedDecoder<C>
where
    C: Codec,
{
    engine: BufferedDecodeEngine<C, StreamCodecBufferedDecodeHooks>,
}

impl<C> StreamCodecBufferedDecoder<C>
where
    C: Codec,
    C::DecodeError: StreamCodecDecodeError,
{
    /// Creates a stream-oriented buffered codec decoder.
    #[inline(always)]
    pub(crate) const fn new(codec: C) -> Self {
        Self {
            engine: BufferedDecodeEngine::new(
                codec,
                StreamCodecBufferedDecodeHooks,
            ),
        }
    }
}

impl<C> BufferedTranscoder<C::Unit, C::Value> for StreamCodecBufferedDecoder<C>
where
    C: Codec,
    C::DecodeError: StreamCodecDecodeError,
{
    type Error = CodecDecodeError<C::DecodeError>;

    #[inline(always)]
    fn max_output_len(
        &self,
        input_len: usize,
    ) -> core::result::Result<usize, CapacityError> {
        self.engine.max_output_len(input_len)
    }

    #[inline(always)]
    fn max_finish_output_len(
        &self,
    ) -> core::result::Result<usize, CapacityError> {
        Ok(self.engine.max_finish_output_len())
    }

    #[inline(always)]
    fn reset(&mut self) {
        self.engine.reset();
    }

    #[inline(always)]
    fn transcode(
        &mut self,
        input: &[C::Unit],
        input_index: usize,
        output: &mut [C::Value],
        output_index: usize,
    ) -> core::result::Result<TranscodeProgress, Self::Error> {
        self.engine
            .transcode(input, input_index, output, output_index)
    }

    #[inline(always)]
    fn finish(
        &mut self,
        output: &mut [C::Value],
        output_index: usize,
    ) -> core::result::Result<usize, FinishError<Self::Error>> {
        self.engine.finish(output, output_index)
    }
}
