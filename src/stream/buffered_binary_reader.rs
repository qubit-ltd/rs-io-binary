// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::marker::PhantomData;
use std::io::{
    Result,
    SeekFrom,
};

use crate::util::MIN_CODEC_BUFFER_CAPACITY;
use qubit_codec::{
    BigEndian,
    ByteOrder,
    ByteOrderSpec,
    Codec as CodecTrait,
    LittleEndian,
    TranscodeDecodeInput,
};
use qubit_codec_binary::BinaryCodec;
use qubit_io::{
    Input,
    Seekable,
};

use super::transcode_decode_input_ext::read_decoded_with_scratch;

/// Buffered reader for fixed-width binary values.
///
/// Scalar reads decode directly from the internal input buffer whenever enough
/// bytes are available, avoiding the per-value temporary buffer used by the
/// extension trait helpers.
///
/// # Buffered state
///
/// This reader may prefetch bytes from the wrapped reader. As a result,
/// [`Self::inner`] can observe an underlying stream position ahead of the
/// logical position exposed by this wrapper.
pub struct BufferedBinaryReader<R, O = BigEndian>
where
    R: Input<Item = u8>,
{
    input: TranscodeDecodeInput<R>,
    marker: PhantomData<fn() -> O>,
}

impl<R, O> BufferedBinaryReader<R, O>
where
    R: Input<Item = u8>,
    O: ByteOrderSpec,
{
    /// Creates a buffered binary reader with the default buffer capacity.
    #[must_use]
    #[inline]
    pub fn new(inner: R) -> Self {
        Self {
            input: TranscodeDecodeInput::new(inner),
            marker: PhantomData,
        }
    }

    /// Creates a buffered binary reader with at least `capacity` bytes.
    #[must_use]
    #[inline]
    pub fn with_capacity(inner: R, capacity: usize) -> Self {
        Self {
            input: TranscodeDecodeInput::with_capacity(
                inner,
                capacity.max(MIN_CODEC_BUFFER_CAPACITY),
            ),
            marker: PhantomData,
        }
    }

    /// Returns the byte order selected by this reader.
    #[must_use]
    #[inline]
    pub const fn byte_order(&self) -> ByteOrder {
        O::ORDER
    }

    /// Returns a shared reference to the underlying reader.
    ///
    /// The underlying reader may already be positioned past unread bytes held
    /// in this wrapper's internal buffer.
    #[must_use]
    #[inline]
    pub const fn inner(&self) -> &R {
        self.input.inner()
    }
}

macro_rules! impl_value_read {
    ($order:ty, $method:ident, $ty:ty, $doc:literal) => {
        #[doc = $doc]
        #[inline]
        pub fn $method(&mut self) -> Result<$ty> {
            type Codec = BinaryCodec<$ty, $order>;
            const SCRATCH_LEN: usize =
                <Codec as CodecTrait>::MAX_DECODE_LIFECYCLE_VALUES;

            let mut scratch = [<$ty>::default(); SCRATCH_LEN];
            read_decoded_with_scratch::<_, Codec>(&mut self.input, &mut scratch)
        }
    };
}

macro_rules! impl_for_order {
    ($order:ty) => {
        impl<R> BufferedBinaryReader<R, $order>
        where
            R: Input<Item = u8>,
        {
            impl_value_read!(
                $order,
                read_u8,
                u8,
                "Reads an unsigned 8-bit integer."
            );
            impl_value_read!(
                $order,
                read_i8,
                i8,
                "Reads a signed 8-bit integer."
            );
            impl_value_read!(
                $order,
                read_u16,
                u16,
                "Reads an unsigned 16-bit integer."
            );
            impl_value_read!(
                $order,
                read_u32,
                u32,
                "Reads an unsigned 32-bit integer."
            );
            impl_value_read!(
                $order,
                read_u64,
                u64,
                "Reads an unsigned 64-bit integer."
            );
            impl_value_read!(
                $order,
                read_u128,
                u128,
                "Reads an unsigned 128-bit integer."
            );
            impl_value_read!(
                $order,
                read_i16,
                i16,
                "Reads a signed 16-bit integer."
            );
            impl_value_read!(
                $order,
                read_i32,
                i32,
                "Reads a signed 32-bit integer."
            );
            impl_value_read!(
                $order,
                read_i64,
                i64,
                "Reads a signed 64-bit integer."
            );
            impl_value_read!(
                $order,
                read_i128,
                i128,
                "Reads a signed 128-bit integer."
            );
            impl_value_read!($order, read_f32, f32, "Reads a 32-bit float.");
            impl_value_read!($order, read_f64, f64, "Reads a 64-bit float.");
        }
    };
}

impl_for_order!(BigEndian);
impl_for_order!(LittleEndian);

impl<R, O> Input for BufferedBinaryReader<R, O>
where
    R: Input<Item = u8>,
{
    type Item = u8;

    #[inline(always)]
    fn is_buffered(&self) -> bool {
        true
    }

    /// Reads bytes from the buffered input.
    #[inline]
    unsafe fn read_unchecked(
        &mut self,
        output: &mut [u8],
        index: usize,
        count: usize,
    ) -> Result<usize> {
        // SAFETY: The caller upholds the indexed destination range contract.
        unsafe { self.input.read_unchecked(output, index, count) }
    }
}

impl<R, O> Seekable for BufferedBinaryReader<R, O>
where
    R: Input<Item = u8> + Seekable<Unit = u8>,
{
    type Unit = u8;

    /// Seeks the wrapped reader and discards buffered bytes after success.
    #[inline]
    fn seek_to(&mut self, position: SeekFrom) -> Result<u64> {
        self.input.seek(position)
    }
}
