// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error as StdError;
use std::io::{
    Error,
    ErrorKind,
    Result,
    Seek,
    SeekFrom,
    Write,
};

use qubit_codec::{
    Codec,
    CodecBufferedEncoder,
    CodecEncodeError,
};

use super::buffered_output::BufferedOutput;

/// Codec-oriented helpers for [`BufferedOutput`].
pub(crate) trait BufferedOutputCodecExt<W> {
    /// Consumes this buffered output after flushing pending bytes.
    fn into_inner(self) -> Result<W>
    where
        W: Write;

    /// Encodes one value through the shared buffered encode driver.
    fn write_encoded<C>(&mut self, value: C::Value) -> Result<()>
    where
        W: Write,
        C: Codec<Unit = u8> + Default,
        C::Value: Copy,
        C::EncodeError: StdError + Send + Sync + 'static;

    /// Writes all raw bytes through the internal buffer.
    fn write_all_buffered(&mut self, input: &[u8]) -> Result<()>
    where
        W: Write;

    /// Flushes pending bytes before seeking the wrapped writer.
    fn seek_raw(&mut self, position: SeekFrom) -> Result<u64>
    where
        W: Write + Seek;
}

impl<W> BufferedOutputCodecExt<W> for BufferedOutput<W>
where
    W: Write,
{
    #[inline]
    fn into_inner(self) -> Result<W>
    where
        W: Write,
    {
        let mut output = self;
        Write::flush(&mut output)?;
        let (inner, pending) = output.into_parts();
        debug_assert!(pending.is_empty(), "buffer still has pending bytes");
        Ok(inner)
    }

    #[inline(always)]
    fn write_encoded<C>(&mut self, value: C::Value) -> Result<()>
    where
        W: Write,
        C: Codec<Unit = u8> + Default,
        C::Value: Copy,
        C::EncodeError: StdError + Send + Sync + 'static,
    {
        let mut encoder = CodecBufferedEncoder::new(C::default());
        let mut map_error = map_codec_encode_error::<C::EncodeError>;
        let input = [value];
        let written = unsafe {
            // SAFETY: The one-value input range is valid.
            self.encode_from_unchecked(
                &mut encoder,
                &mut map_error,
                &input,
                0,
                1,
            )?
        };
        if written == 1 {
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::WriteZero,
                "failed to encode complete value",
            ))
        }
    }

    #[inline]
    fn write_all_buffered(&mut self, input: &[u8]) -> Result<()>
    where
        W: Write,
    {
        self.write_units(input)
    }

    #[inline]
    fn seek_raw(&mut self, position: SeekFrom) -> Result<u64>
    where
        W: Write + Seek,
    {
        Seek::seek(self, position)
    }
}

/// Converts codec encode failures into stream I/O errors.
fn map_codec_encode_error<E>(error: CodecEncodeError<E>) -> Error
where
    E: StdError + Send + Sync + 'static,
{
    Error::new(ErrorKind::InvalidData, error)
}
