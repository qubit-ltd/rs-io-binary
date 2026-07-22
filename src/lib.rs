// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! # Qubit Binary IO
//!
//! Synchronous and asynchronous binary stream I/O adapters for Rust.
//!
//! Synchronous APIs operate on [`qubit_io::Input`] and
//! [`qubit_io::Output`]. Runtime-neutral asynchronous extension traits operate
//! on [`qubit_io::AsyncInput`] and [`qubit_io::AsyncOutput`]. Both layers reuse
//! the same codec implementations for fixed-width values, LEB128, ZigZag, and
//! length-prefixed UTF-8 strings.
//!
//! This crate combines `qubit-io-binary` stream helpers with
//! `qubit-codec-binary` buffer codecs to provide binary reader and writer
//! extension traits and wrapper types.

mod ext;
pub mod prelude;
mod stream;
mod util;

pub use ext::{
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
pub use stream::{
    BinaryReader,
    BinaryWriter,
    BufferedBinaryReader,
    BufferedBinaryWriter,
    BufferedLeb128Reader,
    BufferedLeb128Writer,
    BufferedZigZagReader,
    BufferedZigZagWriter,
    Leb128Reader,
    Leb128Writer,
    ZigZagReader,
    ZigZagWriter,
};
#[doc(hidden)]
pub use stream::{
    StreamCodecDecodeError,
    TranscodeDecodeInputExt,
    TranscodeEncodeOutputExt,
};
