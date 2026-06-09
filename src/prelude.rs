// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Common binary I/O extension traits and codec markers.
//!
//! Importing this module brings binary stream extension traits and the
//! associated buffer codec marker types into scope.

pub use crate::{
    BigEndian, BinaryCodec, BinaryReadExt, BinaryWriteExt, ByteOrder, ByteOrderSpec, Leb128Codec,
    Leb128DecodeError, Leb128DecodeErrorKind, Leb128DecodePolicy, Leb128ReadExt, Leb128WriteExt,
    LittleEndian, NonStrict, Strict, StringReadExt, StringWriteExt, ZigZagCodec, ZigZagReadExt,
    ZigZagWriteExt,
};
