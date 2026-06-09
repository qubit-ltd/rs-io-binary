// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use std::io::{Error, ErrorKind, Read, Result};

use qubit_codec::{BufferedDecodeInput, Codec};

use super::stream_codec_decode_error::StreamCodecDecodeError;

/// Codec-oriented helpers for [`BufferedDecodeInput`].
pub(crate) trait BufferedDecodeInputExt<R> {
    /// Decodes one value through the underlying buffered input.
    fn read_decoded<C>(&mut self) -> Result<C::Value>
    where
        R: Read,
        C: Codec<Unit = u8> + Default,
        C::DecodeError: StreamCodecDecodeError;
}

impl<R> BufferedDecodeInputExt<R> for BufferedDecodeInput<R>
where
    R: Read,
{
    #[inline(always)]
    fn read_decoded<C>(&mut self) -> Result<C::Value>
    where
        R: Read,
        C: Codec<Unit = u8> + Default,
        C::DecodeError: StreamCodecDecodeError,
    {
        let codec = C::default();
        let min_units_per_value = codec.min_units_per_value().get();

        loop {
            let available = self.available();
            if available < min_units_per_value && !self.fill_until(min_units_per_value)? {
                let available = self.available();
                self.consume(available);
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "failed to decode complete value",
                ));
            }

            let units = self.unread_slice();
            debug_assert!(units.len() >= min_units_per_value);
            let decode_result = unsafe {
                // SAFETY: `min_units_per_value <= units.len()` guarantees
                // `decode_unchecked` preconditions for this slice.
                codec.decode_unchecked(units, 0)
            };
            match decode_result {
                Ok((value, consumed)) => {
                    self.consume(consumed.get());
                    return Ok(value);
                }
                Err(error) => {
                    if let Some(required_total) = error.required_total() {
                        if units.len() >= required_total {
                            if let Some(consumed) = error.consumed() {
                                debug_assert!(
                                    consumed.get() <= units.len(),
                                    "decode error consumed bytes exceed unread window"
                                );
                                self.consume(consumed.get());
                            }
                            return Err(Error::new(error.io_error_kind(), error));
                        }
                        if !self.fill_until(required_total)? {
                            let available = self.available();
                            self.consume(available);
                            return Err(Error::new(error.io_error_kind(), error));
                        }
                    } else {
                        if let Some(consumed) = error.consumed() {
                            debug_assert!(
                                consumed.get() <= units.len(),
                                "decode error consumed bytes exceed unread window"
                            );
                            self.consume(consumed.get());
                        }
                        return Err(Error::new(error.io_error_kind(), error));
                    }
                }
            }
        }
    }
}
