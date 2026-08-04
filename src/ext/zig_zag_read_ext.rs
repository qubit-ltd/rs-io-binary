// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::Result;

use crate::util::read_leb128_payload;
use qubit_codec_binary::{
    NonStrict,
    Strict,
    ZigZagCodec,
};
use qubit_io::Input;

macro_rules! read_zig_zag_value {
    ($reader:expr, $ty:ty, $policy:ty) => {
        read_leb128_payload::<
            { ZigZagCodec::<$ty, $policy>::MAX_DECODE_UNITS_PER_VALUE },
            ZigZagCodec<$ty, $policy>,
            _,
        >($reader)
    };
}

/// Extension methods for reading ZigZag + unsigned LEB128 integers.
///
/// # Target-width integers
///
/// `isize` methods use the current Rust target's pointer width. Prefer
/// fixed-width integer methods such as [`Self::read_zig_zag_i64_non_strict`]
/// for persistent files and cross-platform protocols.
/// Non-strict methods use the `_non_strict` suffix; canonical-only methods use
/// the corresponding `_strict` suffix.
pub trait ZigZagReadExt: Input<Item = u8> {
    /// Reads a non-strict ZigZag `i8`.
    ///
    /// # Returns
    ///
    /// Returns the decoded integer.
    ///
    /// # Errors
    ///
    /// Returns an input error or an invalid-data error when the payload is
    /// malformed or violates the selected canonicality policy.
    #[inline(always)]
    fn read_zig_zag_i8_non_strict(&mut self) -> Result<i8> {
        read_zig_zag_value!(self, i8, NonStrict)
    }

    /// Reads a strict ZigZag `i8`.
    ///
    /// # Returns
    ///
    /// Returns the decoded integer.
    ///
    /// # Errors
    ///
    /// Returns an input error or an invalid-data error when the payload is
    /// malformed or violates the selected canonicality policy.
    #[inline(always)]
    fn read_zig_zag_i8_strict(&mut self) -> Result<i8> {
        read_zig_zag_value!(self, i8, Strict)
    }

    /// Reads a non-strict ZigZag `i16`.
    ///
    /// # Returns
    ///
    /// Returns the decoded integer.
    ///
    /// # Errors
    ///
    /// Returns an input error or an invalid-data error when the payload is
    /// malformed or violates the selected canonicality policy.
    #[inline(always)]
    fn read_zig_zag_i16_non_strict(&mut self) -> Result<i16> {
        read_zig_zag_value!(self, i16, NonStrict)
    }

    /// Reads a strict ZigZag `i16`.
    ///
    /// # Returns
    ///
    /// Returns the decoded integer.
    ///
    /// # Errors
    ///
    /// Returns an input error or an invalid-data error when the payload is
    /// malformed or violates the selected canonicality policy.
    #[inline(always)]
    fn read_zig_zag_i16_strict(&mut self) -> Result<i16> {
        read_zig_zag_value!(self, i16, Strict)
    }

    /// Reads a non-strict ZigZag `i32`.
    ///
    /// # Returns
    ///
    /// Returns the decoded integer.
    ///
    /// # Errors
    ///
    /// Returns an input error or an invalid-data error when the payload is
    /// malformed or violates the selected canonicality policy.
    #[inline(always)]
    fn read_zig_zag_i32_non_strict(&mut self) -> Result<i32> {
        read_zig_zag_value!(self, i32, NonStrict)
    }

    /// Reads a strict ZigZag `i32`.
    ///
    /// # Returns
    ///
    /// Returns the decoded integer.
    ///
    /// # Errors
    ///
    /// Returns an input error or an invalid-data error when the payload is
    /// malformed or violates the selected canonicality policy.
    #[inline(always)]
    fn read_zig_zag_i32_strict(&mut self) -> Result<i32> {
        read_zig_zag_value!(self, i32, Strict)
    }

    /// Reads a non-strict ZigZag `i64`.
    ///
    /// # Returns
    ///
    /// Returns the decoded integer.
    ///
    /// # Errors
    ///
    /// Returns an input error or an invalid-data error when the payload is
    /// malformed or violates the selected canonicality policy.
    #[inline(always)]
    fn read_zig_zag_i64_non_strict(&mut self) -> Result<i64> {
        read_zig_zag_value!(self, i64, NonStrict)
    }

    /// Reads a strict ZigZag `i64`.
    ///
    /// # Returns
    ///
    /// Returns the decoded integer.
    ///
    /// # Errors
    ///
    /// Returns an input error or an invalid-data error when the payload is
    /// malformed or violates the selected canonicality policy.
    #[inline(always)]
    fn read_zig_zag_i64_strict(&mut self) -> Result<i64> {
        read_zig_zag_value!(self, i64, Strict)
    }

    /// Reads a non-strict ZigZag `i128`.
    ///
    /// # Returns
    ///
    /// Returns the decoded integer.
    ///
    /// # Errors
    ///
    /// Returns an input error or an invalid-data error when the payload is
    /// malformed or violates the selected canonicality policy.
    #[inline(always)]
    fn read_zig_zag_i128_non_strict(&mut self) -> Result<i128> {
        read_zig_zag_value!(self, i128, NonStrict)
    }

    /// Reads a strict ZigZag `i128`.
    ///
    /// # Returns
    ///
    /// Returns the decoded integer.
    ///
    /// # Errors
    ///
    /// Returns an input error or an invalid-data error when the payload is
    /// malformed or violates the selected canonicality policy.
    #[inline(always)]
    fn read_zig_zag_i128_strict(&mut self) -> Result<i128> {
        read_zig_zag_value!(self, i128, Strict)
    }

    /// Reads a non-strict ZigZag `isize`.
    ///
    /// # Returns
    ///
    /// Returns the decoded integer.
    ///
    /// # Errors
    ///
    /// Returns an input error or an invalid-data error when the payload is
    /// malformed or violates the selected canonicality policy.
    #[inline(always)]
    fn read_zig_zag_isize_non_strict(&mut self) -> Result<isize> {
        read_zig_zag_value!(self, isize, NonStrict)
    }

    /// Reads a strict ZigZag `isize`.
    ///
    /// # Returns
    ///
    /// Returns the decoded integer.
    ///
    /// # Errors
    ///
    /// Returns an input error or an invalid-data error when the payload is
    /// malformed or violates the selected canonicality policy.
    #[inline(always)]
    fn read_zig_zag_isize_strict(&mut self) -> Result<isize> {
        read_zig_zag_value!(self, isize, Strict)
    }
}

impl<R> ZigZagReadExt for R where R: Input<Item = u8> + ?Sized {}
