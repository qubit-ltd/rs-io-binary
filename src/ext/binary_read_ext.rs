// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Extension methods for decoding fixed-width binary values from byte streams.

use std::io::Result;

use qubit_codec::BigEndian;
use qubit_codec::ByteOrder;
use qubit_codec::LittleEndian;
use qubit_codec_binary::BinaryCodec;
use qubit_io::Input;

use crate::util::decode_infallible_unchecked;

macro_rules! read_binary_value {
    ($reader:expr, $ty:ty, $order:ty) => {
        read_binary::<
            { BinaryCodec::<$ty, $order>::MIN_UNITS_PER_VALUE },
            _,
            _,
            _,
        >($reader, |bytes| {
            type Codec = BinaryCodec<$ty, $order>;
            // SAFETY: The local buffer is exactly the codec's minimum buffer
            // length.
            unsafe { decode_infallible_unchecked::<Codec>(bytes, 0) }
        })
    };
}

/// Resolves a runtime byte-order choice to a big-endian branch.
///
/// # Parameters
///
/// - `byte_order`: Runtime byte-order selection.
///
/// # Returns
///
/// Returns `true` for big endian, `false` for little endian, and the target's
/// native ordering for [`ByteOrder::NativeEndian`].
#[must_use]
#[inline]
const fn use_big_endian(byte_order: ByteOrder) -> bool {
    match byte_order {
        ByteOrder::BigEndian => true,
        ByteOrder::LittleEndian => false,
        ByteOrder::NativeEndian => cfg!(target_endian = "big"),
    }
}

/// Extension methods for reading fixed-width binary values from byte streams.
pub trait BinaryReadExt: Input<Item = u8> {
    /// Reads an unsigned 8-bit integer.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_u8(&mut self) -> Result<u8> {
        read_binary_value!(self, u8, BigEndian)
    }

    /// Reads a signed 8-bit integer.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_i8(&mut self) -> Result<i8> {
        read_binary_value!(self, i8, BigEndian)
    }

    /// Reads an unsigned 16-bit integer using a runtime byte order.
    ///
    /// # Parameters
    ///
    /// - `byte_order`: Byte order used to decode the value.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline]
    fn read_u16(&mut self, byte_order: ByteOrder) -> Result<u16> {
        if use_big_endian(byte_order) {
            self.read_u16_be()
        } else {
            self.read_u16_le()
        }
    }

    /// Reads a big-endian unsigned 16-bit integer.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_u16_be(&mut self) -> Result<u16> {
        read_binary_value!(self, u16, BigEndian)
    }

    /// Reads a little-endian unsigned 16-bit integer.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_u16_le(&mut self) -> Result<u16> {
        read_binary_value!(self, u16, LittleEndian)
    }

    /// Reads an unsigned 32-bit integer using a runtime byte order.
    ///
    /// # Parameters
    ///
    /// - `byte_order`: Byte order used to decode the value.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline]
    fn read_u32(&mut self, byte_order: ByteOrder) -> Result<u32> {
        if use_big_endian(byte_order) {
            self.read_u32_be()
        } else {
            self.read_u32_le()
        }
    }

    /// Reads a big-endian unsigned 32-bit integer.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_u32_be(&mut self) -> Result<u32> {
        read_binary_value!(self, u32, BigEndian)
    }

    /// Reads a little-endian unsigned 32-bit integer.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_u32_le(&mut self) -> Result<u32> {
        read_binary_value!(self, u32, LittleEndian)
    }

    /// Reads an unsigned 64-bit integer using a runtime byte order.
    ///
    /// # Parameters
    ///
    /// - `byte_order`: Byte order used to decode the value.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline]
    fn read_u64(&mut self, byte_order: ByteOrder) -> Result<u64> {
        if use_big_endian(byte_order) {
            self.read_u64_be()
        } else {
            self.read_u64_le()
        }
    }

    /// Reads a big-endian unsigned 64-bit integer.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_u64_be(&mut self) -> Result<u64> {
        read_binary_value!(self, u64, BigEndian)
    }

    /// Reads a little-endian unsigned 64-bit integer.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_u64_le(&mut self) -> Result<u64> {
        read_binary_value!(self, u64, LittleEndian)
    }

    /// Reads an unsigned 128-bit integer using a runtime byte order.
    ///
    /// # Parameters
    ///
    /// - `byte_order`: Byte order used to decode the value.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline]
    fn read_u128(&mut self, byte_order: ByteOrder) -> Result<u128> {
        if use_big_endian(byte_order) {
            self.read_u128_be()
        } else {
            self.read_u128_le()
        }
    }

    /// Reads a big-endian unsigned 128-bit integer.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_u128_be(&mut self) -> Result<u128> {
        read_binary_value!(self, u128, BigEndian)
    }

    /// Reads a little-endian unsigned 128-bit integer.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_u128_le(&mut self) -> Result<u128> {
        read_binary_value!(self, u128, LittleEndian)
    }

    /// Reads a signed 16-bit integer using a runtime byte order.
    ///
    /// # Parameters
    ///
    /// - `byte_order`: Byte order used to decode the value.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline]
    fn read_i16(&mut self, byte_order: ByteOrder) -> Result<i16> {
        if use_big_endian(byte_order) {
            self.read_i16_be()
        } else {
            self.read_i16_le()
        }
    }

    /// Reads a big-endian signed 16-bit integer.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_i16_be(&mut self) -> Result<i16> {
        read_binary_value!(self, i16, BigEndian)
    }

    /// Reads a little-endian signed 16-bit integer.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_i16_le(&mut self) -> Result<i16> {
        read_binary_value!(self, i16, LittleEndian)
    }

    /// Reads a signed 32-bit integer using a runtime byte order.
    ///
    /// # Parameters
    ///
    /// - `byte_order`: Byte order used to decode the value.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline]
    fn read_i32(&mut self, byte_order: ByteOrder) -> Result<i32> {
        if use_big_endian(byte_order) {
            self.read_i32_be()
        } else {
            self.read_i32_le()
        }
    }

    /// Reads a big-endian signed 32-bit integer.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_i32_be(&mut self) -> Result<i32> {
        read_binary_value!(self, i32, BigEndian)
    }

    /// Reads a little-endian signed 32-bit integer.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_i32_le(&mut self) -> Result<i32> {
        read_binary_value!(self, i32, LittleEndian)
    }

    /// Reads a signed 64-bit integer using a runtime byte order.
    ///
    /// # Parameters
    ///
    /// - `byte_order`: Byte order used to decode the value.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline]
    fn read_i64(&mut self, byte_order: ByteOrder) -> Result<i64> {
        if use_big_endian(byte_order) {
            self.read_i64_be()
        } else {
            self.read_i64_le()
        }
    }

    /// Reads a big-endian signed 64-bit integer.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_i64_be(&mut self) -> Result<i64> {
        read_binary_value!(self, i64, BigEndian)
    }

    /// Reads a little-endian signed 64-bit integer.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_i64_le(&mut self) -> Result<i64> {
        read_binary_value!(self, i64, LittleEndian)
    }

    /// Reads a signed 128-bit integer using a runtime byte order.
    ///
    /// # Parameters
    ///
    /// - `byte_order`: Byte order used to decode the value.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline]
    fn read_i128(&mut self, byte_order: ByteOrder) -> Result<i128> {
        if use_big_endian(byte_order) {
            self.read_i128_be()
        } else {
            self.read_i128_le()
        }
    }

    /// Reads a big-endian signed 128-bit integer.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_i128_be(&mut self) -> Result<i128> {
        read_binary_value!(self, i128, BigEndian)
    }

    /// Reads a little-endian signed 128-bit integer.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_i128_le(&mut self) -> Result<i128> {
        read_binary_value!(self, i128, LittleEndian)
    }

    /// Reads a 32-bit float using a runtime byte order.
    ///
    /// # Parameters
    ///
    /// - `byte_order`: Byte order used to decode the value.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline]
    fn read_f32(&mut self, byte_order: ByteOrder) -> Result<f32> {
        if use_big_endian(byte_order) {
            self.read_f32_be()
        } else {
            self.read_f32_le()
        }
    }

    /// Reads a big-endian 32-bit float.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_f32_be(&mut self) -> Result<f32> {
        read_binary_value!(self, f32, BigEndian)
    }

    /// Reads a little-endian 32-bit float.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_f32_le(&mut self) -> Result<f32> {
        read_binary_value!(self, f32, LittleEndian)
    }

    /// Reads a 64-bit float using a runtime byte order.
    ///
    /// # Parameters
    ///
    /// - `byte_order`: Byte order used to decode the value.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline]
    fn read_f64(&mut self, byte_order: ByteOrder) -> Result<f64> {
        if use_big_endian(byte_order) {
            self.read_f64_be()
        } else {
            self.read_f64_le()
        }
    }

    /// Reads a big-endian 64-bit float.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_f64_be(&mut self) -> Result<f64> {
        read_binary_value!(self, f64, BigEndian)
    }

    /// Reads a little-endian 64-bit float.
    ///
    /// # Returns
    ///
    /// Returns the decoded scalar.
    ///
    /// # Errors
    ///
    /// Returns an input error, including an unexpected-end-of-input error
    /// when a complete scalar is unavailable.
    #[inline(always)]
    fn read_f64_le(&mut self) -> Result<f64> {
        read_binary_value!(self, f64, LittleEndian)
    }
}

impl<R> BinaryReadExt for R where R: Input<Item = u8> + ?Sized {}

/// Reads and decodes one fixed-width value.
///
/// # Type Parameters
///
/// - `N`: Encoded scalar width in bytes.
/// - `T`: Decoded value type.
/// - `R`: Source byte input.
/// - `F`: Infallible decoding callback.
///
/// # Parameters
///
/// - `reader`: Source from which the fixed-width payload is read.
/// - `decode`: Callback that decodes the initialized local buffer.
///
/// # Returns
///
/// Returns the decoded value.
///
/// # Errors
///
/// Returns an input error, including an unexpected-end-of-input error when the
/// payload is truncated.
#[inline]
fn read_binary<const N: usize, T, R, F>(reader: &mut R, decode: F) -> Result<T>
where
    R: Input<Item = u8> + ?Sized,
    F: FnOnce(&[u8]) -> T,
{
    let mut bytes = [0u8; N];
    Input::read_exactly(reader, &mut bytes)?;
    Ok(decode(&bytes))
}
