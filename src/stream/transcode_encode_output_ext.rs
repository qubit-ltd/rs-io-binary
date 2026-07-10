// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
use std::error::Error as StdError;
use std::io::{
    self,
    Error,
    ErrorKind,
};

use qubit_codec::{
    Codec,
    TranscodeEncodeOutput,
};
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
        self.write_encoded_with(&mut codec, &value, |source| {
            Error::new(ErrorKind::InvalidData, source)
        })
    }
}
