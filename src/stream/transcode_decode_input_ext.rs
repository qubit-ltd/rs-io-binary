// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
use std::io::{
    Error,
    ErrorKind,
    Result,
};

use qubit_codec::{
    Codec,
    CodecDecodeErrorSignal,
    TranscodeDecodeInput,
};
use qubit_io::Input;

use super::stream_codec_decode_error::StreamCodecDecodeError;

/// Codec-oriented helpers for [`TranscodeDecodeInput`].
pub trait TranscodeDecodeInputExt<I> {
    /// Decodes one value through the underlying buffered input.
    fn read_decoded<C>(&mut self) -> Result<C::Value>
    where
        I: Input,
        C: Codec<Unit = I::Item> + Default,
        C::DecodeError: StreamCodecDecodeError;
}

impl<I> TranscodeDecodeInputExt<I> for TranscodeDecodeInput<I>
where
    I: Input,
    I::Item: Copy + Default,
{
    fn read_decoded<C>(&mut self) -> Result<C::Value>
    where
        C: Codec<Unit = I::Item> + Default,
        C::DecodeError: StreamCodecDecodeError,
    {
        let mut codec = C::default();
        let min_units_per_value = codec.min_units_per_value().get();
        let max_units_per_value =
            codec.max_units_per_value().get().max(min_units_per_value);
        if min_units_per_value > self.capacity() {
            return read_decoded_via_scratch(
                self,
                &mut codec,
                min_units_per_value,
            );
        }

        loop {
            let available = self.available();
            if available < min_units_per_value
                && !self.fill_until(min_units_per_value)?
            {
                let available = self.available();
                self.consume(available);
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "failed to decode complete value",
                ));
            }

            if self.available() < max_units_per_value
                && max_units_per_value <= self.capacity()
            {
                let _ = self.fill_until(max_units_per_value)?;
            }

            let available = self.available();
            let unit_count = available.min(max_units_per_value);
            let units = &self.unread()[..unit_count];
            debug_assert!(units.len() >= min_units_per_value);
            let decode_result = unsafe {
                // SAFETY: `min_units_per_value <= units.len()` guarantees
                // `decode` preconditions for this slice.
                codec.decode(units, 0)
            };
            match decode_result {
                Ok((value, consumed)) => {
                    self.consume(consumed.get());
                    return Ok(value);
                }
                Err(error) => {
                    if let Some(required_total) = error.required_total() {
                        if units.len() >= required_total {
                            if let Some(consumed) = error.consumed_units() {
                                debug_assert!(
                                    consumed.get() <= units.len(),
                                    "decode error consumed bytes exceed unread window"
                                );
                                self.consume(consumed.get());
                            }
                            return Err(Error::new(
                                error.io_error_kind(),
                                error,
                            ));
                        }
                        if !self.fill_until(required_total)? {
                            let available = self.available();
                            self.consume(available);
                            return Err(Error::new(
                                error.io_error_kind(),
                                error,
                            ));
                        }
                    } else {
                        if let Some(consumed) = error.consumed_units() {
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

fn read_decoded_via_scratch<I, C>(
    input: &mut TranscodeDecodeInput<I>,
    codec: &mut C,
    mut required_total: usize,
) -> Result<C::Value>
where
    I: Input,
    I::Item: Copy + Default,
    C: Codec<Unit = I::Item>,
    C::DecodeError: StreamCodecDecodeError,
{
    let mut units = vec![I::Item::default(); required_total];
    let mut loaded = 0;
    loop {
        while loaded < required_total {
            let remaining = required_total - loaded;
            // SAFETY: `units[loaded..required_total]` is a valid destination
            // range inside the scratch buffer.
            let read =
                unsafe { input.read_unchecked(&mut units, loaded, remaining) }?;
            if read == 0 {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "failed to decode complete value",
                ));
            }
            loaded += read;
        }
        let decode_result = unsafe {
            // SAFETY: `loaded >= required_total >= min_units_per_value`, so
            // the scratch buffer contains the required prefix for decoding.
            codec.decode(&units, 0)
        };
        match decode_result {
            Ok((value, _)) => return Ok(value),
            Err(error) => {
                if let Some(next_required_total) = error.required_total() {
                    if next_required_total <= loaded {
                        return Err(Error::new(error.io_error_kind(), error));
                    }
                    units.resize(next_required_total, I::Item::default());
                    required_total = next_required_total;
                } else {
                    return Err(Error::new(error.io_error_kind(), error));
                }
            }
        }
    }
}
