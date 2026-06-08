// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use std::io::{
    Error,
    ErrorKind,
    Read,
    Result,
};

use qubit_codec::{
    BufferedDecodeInput,
    Codec,
};

use super::stream_codec_decode_error::StreamCodecDecodeError;

/// Codec-oriented helpers for [`BufferedDecodeInput`].
pub(crate) trait BufferedDecodeInputExt<R> {
    /// Consumes this buffered input and returns the wrapped input object.
    #[must_use]
    fn into_inner(self) -> R;

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
        C::DecodeError: StreamCodecDecodeError,
    {
        let codec = C::default();
        let min_units_per_value = codec.min_units_per_value().get();

        loop {
            let (_, _, available) = self.unread_raw_parts();
            if available < min_units_per_value
                && !self.fill_until(min_units_per_value)?
            {
                self.consume_available();
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "failed to decode complete value",
                ));
            }

            let (units, unit_index, available) = self.unread_raw_parts();
            debug_assert!(available >= min_units_per_value);
            let decode_result = unsafe {
                // SAFETY: `min_units_per_value <= available` guarantees
                // `decode_unchecked` preconditions for `unit_index`.
                codec.decode_unchecked(units, unit_index)
            };
            match decode_result {
                Ok((value, consumed)) => {
                    self.consume_units(consumed.get());
                    return Ok(value);
                }
                Err(error) => {
                    if let Some(required_total) = error.required_total() {
                        if available >= required_total {
                            if let Some(consumed) = error.consumed() {
                                debug_assert!(
                                    consumed.get() <= available,
                                    "decode error consumed bytes exceed unread window"
                                );
                                self.consume_units(consumed.get());
                            }
                            return Err(Error::new(
                                error.io_error_kind(),
                                error,
                            ));
                        }
                        if !self.fill_until(required_total)? {
                            self.consume_available();
                            return Err(Error::new(
                                error.io_error_kind(),
                                error,
                            ));
                        }
                    } else {
                        if let Some(consumed) = error.consumed() {
                            debug_assert!(
                                consumed.get() <= available,
                                "decode error consumed bytes exceed unread window"
                            );
                            self.consume_units(consumed.get());
                        }
                        return Err(Error::new(error.io_error_kind(), error));
                    }
                }
            }
        }
    }
}
