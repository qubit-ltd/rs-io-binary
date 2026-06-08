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
use qubit_codec::Codec;

/// Codec-oriented helpers for [`BufferedEncodeOutput`].
pub(crate) trait BufferedEncodeOutputExt<W> {
    /// Consumes this buffered output after flushing pending bytes.
    fn into_inner(self) -> Result<W>
    where
        W: Write;

    /// Encodes one value through the underlying buffered output.
    fn write_encoded<C>(&mut self, value: C::Value) -> Result<()>
    where
        W: Write,
        C: Codec<Unit = u8> + Default,
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
        C::EncodeError: StdError + Send + Sync + 'static,
    {
        let codec = C::default();
        let max_units_per_value = codec.max_units_per_value().get();
        self.ensure_spare_capacity(max_units_per_value)?;

        let (units, unit_index, _) = self.spare_raw_parts_mut();
        let written = unsafe {
            // SAFETY: `ensure_spare_capacity` guarantees enough spare units.
            codec.encode_unchecked(&value, units, unit_index)
        }
        .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        self.advance_unchecked(written);
        Ok(())
    }
}
