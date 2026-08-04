// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Reader and writer wrapper types for codec-oriented I/O.

use std::io::{
    Result,
    SeekFrom,
};

use qubit_io::{
    Input,
    Output,
    Seekable,
};

mod binary_reader;
mod binary_writer;
mod buffered_binary_reader;
mod buffered_binary_writer;
mod buffered_leb128_reader;
mod buffered_leb128_writer;
mod buffered_zig_zag_reader;
mod buffered_zig_zag_writer;
mod internal;
mod leb128_reader;
mod leb128_writer;
mod zig_zag_reader;
mod zig_zag_writer;

pub use binary_reader::BinaryReader;
pub use binary_writer::BinaryWriter;
pub use buffered_binary_reader::BufferedBinaryReader;
pub use buffered_binary_writer::BufferedBinaryWriter;
pub use buffered_leb128_reader::BufferedLeb128Reader;
pub use buffered_leb128_writer::BufferedLeb128Writer;
pub use buffered_zig_zag_reader::BufferedZigZagReader;
pub use buffered_zig_zag_writer::BufferedZigZagWriter;
pub use leb128_reader::Leb128Reader;
pub use leb128_writer::Leb128Writer;
pub use zig_zag_reader::ZigZagReader;
pub use zig_zag_writer::ZigZagWriter;

macro_rules! impl_input_methods {
    ($type:ident<$($param:ident),+> where $($bound:tt)*) => {
        impl<$($param),+> $type<$($param),+>
        where
            $($bound)*
        {
            /// Returns whether this reader buffers input.
            #[inline(always)]
            pub fn is_buffered(&self) -> bool {
                Input::is_buffered(self)
            }

            /// Reads up to `count` bytes into an indexed output range.
            ///
            /// # Safety
            ///
            /// `index..index + count` must be a valid range within `output`.
            #[inline(always)]
            pub unsafe fn read_unchecked(
                &mut self,
                output: &mut [u8],
                index: usize,
                count: usize,
            ) -> Result<usize> {
                // SAFETY: The caller upholds the indexed range contract.
                unsafe { Input::read_unchecked(self, output, index, count) }
            }

            /// Reads bytes into the full output slice.
            #[inline(always)]
            pub fn read(&mut self, output: &mut [u8]) -> Result<usize> {
                Input::read(self, output)
            }

            /// Reads bytes into an indexed range until it is full or EOF.
            ///
            /// # Safety
            ///
            /// `index..index + count` must be a valid range within `output`.
            #[inline(always)]
            pub unsafe fn read_fully_unchecked(
                &mut self,
                output: &mut [u8],
                index: usize,
                count: usize,
            ) -> Result<usize> {
                // SAFETY: The caller upholds the indexed range contract.
                unsafe { Input::read_fully_unchecked(self, output, index, count) }
            }

            /// Reads bytes into the full output slice until it is full or EOF.
            #[inline(always)]
            pub fn read_fully(&mut self, output: &mut [u8]) -> Result<usize> {
                Input::read_fully(self, output)
            }

            /// Reads bytes until the full output slice is filled.
            #[inline(always)]
            pub fn read_exactly(&mut self, output: &mut [u8]) -> Result<()> {
                Input::read_exactly(self, output)
            }
        }
    };
}

macro_rules! impl_output_methods {
    ($type:ident<$($param:ident),+> where $($bound:tt)*) => {
        impl<$($param),+> $type<$($param),+>
        where
            $($bound)*
        {
            /// Returns whether this writer buffers output.
            #[inline(always)]
            pub fn is_buffered(&self) -> bool {
                Output::is_buffered(self)
            }

            /// Writes up to `count` bytes from an indexed input range.
            ///
            /// # Safety
            ///
            /// `index..index + count` must be a valid range within `input`.
            #[inline(always)]
            pub unsafe fn write_unchecked(
                &mut self,
                input: &[u8],
                index: usize,
                count: usize,
            ) -> Result<usize> {
                // SAFETY: The caller upholds the indexed range contract.
                unsafe { Output::write_unchecked(self, input, index, count) }
            }

            /// Writes bytes from the full input slice.
            #[inline(always)]
            pub fn write(&mut self, input: &[u8]) -> Result<usize> {
                Output::write(self, input)
            }

            /// Writes all bytes from an indexed input range.
            ///
            /// # Safety
            ///
            /// `index..index + count` must be a valid range within `input`.
            #[inline(always)]
            pub unsafe fn write_fully_unchecked(
                &mut self,
                input: &[u8],
                index: usize,
                count: usize,
            ) -> Result<()> {
                // SAFETY: The caller upholds the indexed range contract.
                unsafe { Output::write_fully_unchecked(self, input, index, count) }
            }

            /// Writes all bytes from the full input slice.
            #[inline(always)]
            pub fn write_fully(&mut self, input: &[u8]) -> Result<()> {
                Output::write_fully(self, input)
            }

            /// Flushes buffered output.
            #[inline(always)]
            pub fn flush(&mut self) -> Result<()> {
                Output::flush(self)
            }
        }
    };
}

macro_rules! impl_seekable_methods {
    ($type:ident<$($param:ident),+> where $($bound:tt)*) => {
        impl<$($param),+> $type<$($param),+>
        where
            $($bound)*
        {
            /// Seeks the wrapped byte stream.
            #[inline(always)]
            pub fn seek_to(&mut self, position: SeekFrom) -> Result<u64> {
                Seekable::seek_to(self, position)
            }
        }
    };
}

impl_input_methods!(BinaryReader<R, O> where R: Input<Item = u8>);
impl_input_methods!(BufferedBinaryReader<R, O> where R: Input<Item = u8>);
impl_input_methods!(Leb128Reader<R, P> where R: Input<Item = u8>);
impl_input_methods!(BufferedLeb128Reader<R, P> where R: Input<Item = u8>);
impl_input_methods!(ZigZagReader<R, P> where R: Input<Item = u8>);
impl_input_methods!(BufferedZigZagReader<R, P> where R: Input<Item = u8>);

impl_output_methods!(BinaryWriter<W, O> where W: Output<Item = u8>);
impl_output_methods!(BufferedBinaryWriter<W, O> where W: Output<Item = u8>);
impl_output_methods!(Leb128Writer<W> where W: Output<Item = u8>);
impl_output_methods!(BufferedLeb128Writer<W> where W: Output<Item = u8>);
impl_output_methods!(ZigZagWriter<W> where W: Output<Item = u8>);
impl_output_methods!(BufferedZigZagWriter<W> where W: Output<Item = u8>);

impl_seekable_methods!(BinaryReader<R, O> where R: Seekable<Unit = u8>);
impl_seekable_methods!(BufferedBinaryReader<R, O> where R: Input<Item = u8> + Seekable<Unit = u8>);
impl_seekable_methods!(Leb128Reader<R, P> where R: Seekable<Unit = u8>);
impl_seekable_methods!(BufferedLeb128Reader<R, P> where R: Input<Item = u8> + Seekable<Unit = u8>);
impl_seekable_methods!(ZigZagReader<R, P> where R: Seekable<Unit = u8>);
impl_seekable_methods!(BufferedZigZagReader<R, P> where R: Input<Item = u8> + Seekable<Unit = u8>);
impl_seekable_methods!(BinaryWriter<W, O> where W: Seekable<Unit = u8>);
impl_seekable_methods!(BufferedBinaryWriter<W, O> where W: Output<Item = u8> + Seekable<Unit = u8>);
impl_seekable_methods!(Leb128Writer<W> where W: Seekable<Unit = u8>);
impl_seekable_methods!(BufferedLeb128Writer<W> where W: Output<Item = u8> + Seekable<Unit = u8>);
impl_seekable_methods!(ZigZagWriter<W> where W: Seekable<Unit = u8>);
impl_seekable_methods!(BufferedZigZagWriter<W> where W: Output<Item = u8> + Seekable<Unit = u8>);
