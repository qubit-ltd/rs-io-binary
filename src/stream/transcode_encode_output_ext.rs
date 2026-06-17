// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use std::error::Error as StdError;
use std::io::{self, Error, ErrorKind};

use qubit_codec::{Codec, TranscodeEncodeOutput};
use qubit_io::Output;

/// Codec-oriented helpers for [`TranscodeEncodeOutput`].
pub trait TranscodeEncodeOutputExt<O> {
    /// Encodes one value through the underlying buffered output.
    fn write_encoded<C>(&mut self, value: C::Value) -> io::Result<()>
    where
        O: Output,
        C: Codec<Unit = O::Item> + Default,
        C::EncodeError: StdError + Send + Sync + 'static;
}

impl<O> TranscodeEncodeOutputExt<O> for TranscodeEncodeOutput<O>
where
    O: Output,
    O::Item: Copy + Default,
{
    #[inline(always)]
    fn write_encoded<C>(&mut self, value: C::Value) -> io::Result<()>
    where
        O: Output,
        O::Item: Copy + Default,
        C: Codec<Unit = O::Item> + Default,
        C::EncodeError: StdError + Send + Sync + 'static,
    {
        let mut codec = C::default();
        let max_units = codec
            .max_encode_value_units()
            .map_err(|_| Error::new(ErrorKind::InvalidInput, "codec output bound overflow"))?;
        if let Err(error) = self.ensure_spare_capacity(max_units) {
            if error.kind() != ErrorKind::InvalidInput {
                return Err(error);
            }
            // flush the possible data in the buffer
            self.flush()?;
            // encode the value into the scratch buffer
            let mut scratch = vec![O::Item::default(); max_units];
            let written = codec
                .encode_value_with_reset(&value, &mut scratch, 0)
                .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
            // After flushing pending units, delegate the oversized encoded
            // payload through the wrapped output's unit write path.
            unsafe {
                self.inner_mut().write_all(&scratch, 0, written)?;
            }
            return Ok(());
        }
        // encode the value into the spare buffer
        let (units, output_index, available) = self.spare_raw_parts_mut();
        debug_assert!(
            available >= max_units,
            "reserved spare buffer is smaller than codec upper bound",
        );
        let written = codec
            .encode_value_with_reset(&value, units, output_index)
            .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        // advance the buffer by the number of written units
        unsafe { self.advance(written) };
        Ok(())
    }
}
