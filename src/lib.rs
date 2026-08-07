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
//! This crate combines `qubit-io` stream helpers with
//! `qubit-codec-binary` buffer codecs to provide binary reader and writer
//! extension traits and wrapper types.
//!
//! ## Asynchronous extension traits
//!
//! Async extension methods are runtime-neutral and only require `Unpin`
//! inputs and outputs:
//!
//! ```
//! use qubit_io::{
//!     AsyncInput,
//!     AsyncOutput,
//! };
//! use qubit_io_binary::{
//!     AsyncBinaryReadExt,
//!     AsyncBinaryWriteExt,
//! };
//!
//! async fn relay<I, O>(input: &mut I, output: &mut O) -> std::io::Result<()>
//! where
//!     I: AsyncInput<Item = u8> + Unpin,
//!     O: AsyncOutput<Item = u8> + Unpin,
//! {
//!     let value = input.read_u32_be_async().await?;
//!     output.write_u32_be_async(value).await
//! }
//! ```

mod ext;
pub mod prelude;
mod stream;
mod util;

pub use ext::AsyncBinaryReadExt;
pub use ext::AsyncBinaryWriteExt;
pub use ext::AsyncLeb128ReadExt;
pub use ext::AsyncLeb128WriteExt;
pub use ext::AsyncStringReadExt;
pub use ext::AsyncStringWriteExt;
pub use ext::AsyncZigZagReadExt;
pub use ext::AsyncZigZagWriteExt;
pub use ext::BinaryReadExt;
pub use ext::BinaryWriteExt;
pub use ext::Leb128ReadExt;
pub use ext::Leb128WriteExt;
pub use ext::StringReadExt;
pub use ext::StringWriteExt;
pub use ext::ZigZagReadExt;
pub use ext::ZigZagWriteExt;
pub use stream::BinaryReader;
pub use stream::BinaryWriter;
pub use stream::BufferedBinaryReader;
pub use stream::BufferedBinaryWriter;
pub use stream::BufferedLeb128Reader;
pub use stream::BufferedLeb128Writer;
pub use stream::BufferedZigZagReader;
pub use stream::BufferedZigZagWriter;
pub use stream::Leb128Reader;
pub use stream::Leb128Writer;
pub use stream::ZigZagReader;
pub use stream::ZigZagWriter;
