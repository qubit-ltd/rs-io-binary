// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Common binary I/O extension traits.
//!
//! Importing this module brings binary stream extension traits into scope.
//! Codec and byte-order types must be imported directly from their owning
//! `qubit-codec` or `qubit-codec-binary` crate.

pub use crate::{
    AsyncBinaryReadExt,
    AsyncBinaryWriteExt,
    AsyncLeb128ReadExt,
    AsyncLeb128WriteExt,
    AsyncStringReadExt,
    AsyncStringWriteExt,
    AsyncZigZagReadExt,
    AsyncZigZagWriteExt,
    BinaryReadExt,
    BinaryWriteExt,
    Leb128ReadExt,
    Leb128WriteExt,
    StringReadExt,
    StringWriteExt,
    ZigZagReadExt,
    ZigZagWriteExt,
};
