// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! # Qubit Binary IO
//!
//! Binary stream I/O adapters for Rust.
//!
//! This crate combines `qubit-io-binary` stream helpers with
//! `qubit-codec-binary` buffer codecs to provide binary reader and writer
//! extension traits and wrapper types.

mod ext;
pub mod prelude;
mod stream;
mod util;

pub use ext::{
    BinaryReadExt, BinaryWriteExt, Leb128ReadExt, Leb128WriteExt, StringReadExt, StringWriteExt,
    ZigZagReadExt, ZigZagWriteExt,
};
pub use qubit_codec_binary::{
    BigEndian, BinaryCodec, ByteOrder, ByteOrderSpec, Leb128Codec, Leb128DecodeError,
    Leb128DecodeErrorKind, Leb128DecodePolicy, LittleEndian, NonStrict, Strict, ZigZagCodec,
};
pub use qubit_io::{ReadExt, WriteExt};
pub use stream::{
    BinaryReader, BinaryWriter, BufferedBinaryReader, BufferedBinaryWriter, BufferedLeb128Reader,
    BufferedLeb128Writer, BufferedZigZagReader, BufferedZigZagWriter, Leb128Reader, Leb128Writer,
    ZigZagReader, ZigZagWriter,
};
