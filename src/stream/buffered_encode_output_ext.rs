// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use std::error::Error as StdError;
use std::io::{self, Error, ErrorKind};

use qubit_codec::{BufferedEncodeOutput, Codec};

/// Codec-oriented helpers for [`BufferedEncodeOutput`].
pub trait BufferedEncodeOutputExt<W> {
    /// Encodes one value through the underlying buffered output.
    fn write_encoded<C>(&mut self, value: C::Value) -> io::Result<()>
    where
        W: io::Write,
        C: Codec<Unit = u8> + Default,
        C::EncodeError: StdError + Send + Sync + 'static;
}

impl<W> BufferedEncodeOutputExt<W> for BufferedEncodeOutput<W>
where
    W: io::Write,
{
    #[inline(always)]
    fn write_encoded<C>(&mut self, value: C::Value) -> io::Result<()>
    where
        W: io::Write,
        C: Codec<Unit = u8> + Default,
        C::EncodeError: StdError + Send + Sync + 'static,
    {
        let codec = C::default();
        let max_units_per_value = codec.max_units_per_value().get();
        if let Err(error) = self.ensure_spare_capacity(max_units_per_value) {
            if error.kind() != ErrorKind::InvalidInput {
                return Err(error);
            }
            // flush the possible data in the buffer
            self.flush()?;
            // encode the value into the scratch buffer
            let mut scratch = vec![0_u8; max_units_per_value];
            let written = unsafe { codec.encode_unchecked(&value, &mut scratch, 0) }
                .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
            // write the encoded value directly to the inner output
            return self
                .inner_mut()
                .write_all(&scratch[..written]);
        }
        // encode the value into the spare buffer
        let units = self.spare_slice_mut();
        let written = unsafe { codec.encode_unchecked(&value, units, 0) }
            .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
        // advance the buffer by the number of written units
        unsafe { self.advance_unchecked(written) };
        Ok(())
    }
}
