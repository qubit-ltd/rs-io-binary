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

pub use crate::AsyncBinaryReadExt;
pub use crate::AsyncBinaryWriteExt;
pub use crate::AsyncLeb128ReadExt;
pub use crate::AsyncLeb128WriteExt;
pub use crate::AsyncStringReadExt;
pub use crate::AsyncStringWriteExt;
pub use crate::AsyncZigZagReadExt;
pub use crate::AsyncZigZagWriteExt;
pub use crate::BinaryReadExt;
pub use crate::BinaryWriteExt;
pub use crate::Leb128ReadExt;
pub use crate::Leb128WriteExt;
pub use crate::StringReadExt;
pub use crate::StringWriteExt;
pub use crate::ZigZagReadExt;
pub use crate::ZigZagWriteExt;
