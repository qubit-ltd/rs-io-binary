// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use std::error::Error as StdError;
use std::io::{
    Error,
    ErrorKind,
    Result,
    Write,
};

use qubit_codec::BufferedEncodeOutput;
use qubit_codec::{
    Codec,
    CodecBufferedEncoder,
};

/// Codec-oriented helpers for [`BufferedEncodeOutput`].
pub(crate) trait BufferedEncodeOutputExt<W> {
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
}

impl<W> BufferedEncodeOutputExt<W> for BufferedEncodeOutput<W>
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
        let mut map_error = |error| Error::new(ErrorKind::InvalidData, error);
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
}
