// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::{
    convert::Infallible,
    num::NonZeroUsize,
};
use std::error::Error as StdError;
use std::io::{
    Error,
    ErrorKind,
    Read,
    Result,
    Seek,
    SeekFrom,
};

use qubit_codec::{
    BufferedDecodeEngine,
    BufferedDecodeHooks,
    BufferedTranscoder,
    CapacityError,
    Codec,
    CodecDecodeError,
    DecodeAction,
    DecodeContext,
    FinishError,
    TranscodeProgress,
};
use qubit_codec_binary::Leb128DecodeError;

use super::buffered_input::BufferedInput;

/// Codec-oriented helpers for [`BufferedInput`].
pub(crate) trait BufferedInputCodecExt<R> {
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

    /// Reads raw bytes through the internal buffer.
    fn read_raw(&mut self, output: &mut [u8]) -> Result<usize>
    where
        R: Read;

    /// Seeks the wrapped reader and discards buffered bytes after success.
    fn seek_raw(&mut self, position: SeekFrom) -> Result<u64>
    where
        R: Read + Seek;
}

impl<R> BufferedInputCodecExt<R> for BufferedInput<R>
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
        let mut decoder = StreamCodecBufferedDecoder::new(C::default());
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

    #[inline]
    fn read_raw(&mut self, output: &mut [u8]) -> Result<usize>
    where
        R: Read,
    {
        self.read_units(output)
    }

    #[inline]
    fn seek_raw(&mut self, position: SeekFrom) -> Result<u64>
    where
        R: Read + Seek,
    {
        Seek::seek(self, position)
    }
}

/// Decode-error policy used by stream-oriented buffered codec readers.
pub(crate) trait StreamCodecDecodeError:
    StdError + Send + Sync + 'static
{
    /// Returns total units required to continue an incomplete decode.
    fn required_total(&self) -> Option<usize>;

    /// Returns invalid units that should be consumed before reporting error.
    fn consumed(&self) -> Option<NonZeroUsize>;

    /// Returns the matching I/O error kind for this decode error.
    fn io_error_kind(&self) -> ErrorKind;
}

impl StreamCodecDecodeError for Infallible {
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

/// Buffered decoder with stream-friendly incomplete-input policy.
struct StreamCodecBufferedDecoder<C>
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
    const fn new(codec: C) -> Self {
        Self {
            engine: BufferedDecodeEngine::new(
                codec,
                StreamCodecBufferedDecodeHooks,
            ),
        }
    }
}

impl<C> BufferedTranscoder<C::Unit, C::Value>
    for StreamCodecBufferedDecoder<C>
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

/// Stream decode hook that treats codec-level incomplete errors as refill.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
struct StreamCodecBufferedDecodeHooks;

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

/// Converts codec decode failures into stream I/O errors.
fn map_codec_decode_error<E>(error: CodecDecodeError<E>) -> Error
where
    E: StreamCodecDecodeError,
{
    match error {
        CodecDecodeError::Decode { source, .. } => {
            Error::new(source.io_error_kind(), source)
        }
        CodecDecodeError::Incomplete { .. } => {
            Error::new(ErrorKind::UnexpectedEof, error)
        }
        _ => Error::new(ErrorKind::InvalidData, error),
    }
}

/// Returns invalid units that must be consumed before reporting an error.
fn consumed_from_codec_decode_error<E>(
    error: &CodecDecodeError<E>,
) -> Option<NonZeroUsize>
where
    E: StreamCodecDecodeError,
{
    match error {
        CodecDecodeError::Decode { source, .. } => source.consumed(),
        _ => None,
    }
}
