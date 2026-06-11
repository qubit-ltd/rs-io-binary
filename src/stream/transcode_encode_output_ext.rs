// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use std::error::Error as StdError;
use std::io::{self, Error, ErrorKind};

use qubit_codec::{TranscodeEncodeOutput, Codec};
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
        let max_encode_reset_units = codec.max_encode_reset_units();
        let max_units_per_value = codec.max_units_per_value().get();
        let max_units = max_encode_reset_units
            .checked_add(max_units_per_value)
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "codec output bound overflow"))?;
        if let Err(error) = self.ensure_spare_capacity(max_units) {
            if error.kind() != ErrorKind::InvalidInput {
                return Err(error);
            }
            // flush the possible data in the buffer
            self.flush()?;
            // encode the value into the scratch buffer
            let mut scratch = vec![O::Item::default(); max_units];
            let written = unsafe { encode_with_reset(&mut codec, &value, &mut scratch, 0) }
                .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
            // After flushing pending units, delegate the oversized encoded
            // payload through the wrapped output's unit write path.
            let mut total_written = 0;
            while total_written < written {
                let remaining = written - total_written;
                match unsafe {
                    self.inner_mut()
                        .write_unchecked(&scratch, total_written, remaining)
                } {
                    Ok(0) => {
                        return Err(Error::new(
                            ErrorKind::WriteZero,
                            "failed to write all encoded units",
                        ));
                    }
                    Ok(progress) => {
                        total_written += progress;
                    }
                    Err(error) if error.kind() == ErrorKind::Interrupted => {}
                    Err(error) => return Err(error),
                }
            }
            return Ok(());
        }
        // encode the value into the spare buffer
        let (units, output_index, available) = self.spare_raw_parts_mut();
        debug_assert!(
            available >= max_units,
            "reserved spare buffer is smaller than codec upper bound",
        );
        let written = unsafe { encode_with_reset(&mut codec, &value, units, output_index) }
            .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        // advance the buffer by the number of written units
        unsafe { self.advance_unchecked(written) };
        Ok(())
    }
}

unsafe fn encode_with_reset<C>(
    codec: &mut C,
    value: &C::Value,
    output: &mut [C::Unit],
    index: usize,
) -> Result<usize, C::EncodeError>
where
    C: Codec,
{
    let reset_written = unsafe { codec.encode_reset(output, index) }?;
    debug_assert!(reset_written <= codec.max_encode_reset_units());
    let value_written = unsafe { codec.encode(value, output, index + reset_written) }?;
    debug_assert!(value_written <= codec.max_units_per_value().get());
    Ok(reset_written + value_written)
}
