// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Inherent forwarding methods for stream wrappers.

macro_rules! impl_input_methods {
    ($type:ident<$($param:ident),+> where $($bound:tt)*) => {
        impl<$($param),+> $type<$($param),+>
        where
            $($bound)*
        {
            /// Returns whether this reader buffers input.
            ///
            /// # Returns
            ///
            /// Returns `true` when reads may prefetch bytes into an internal buffer.
            #[must_use]
            #[inline(always)]
            pub fn is_buffered(&self) -> bool {
                Input::is_buffered(self)
            }

            /// Reads up to `count` bytes into an indexed output range.
            ///
            /// # Parameters
            ///
            /// - `output`: Destination byte buffer.
            /// - `index`: Start of the writable range.
            /// - `count`: Maximum number of bytes to read.
            ///
            /// # Returns
            ///
            /// Returns the number of bytes read; a successful read may be short.
            /// For a non-empty request, `Ok(0)` means no bytes were available in this
            /// call or the input reached its end; it does not promise permanent EOF.
            /// A successful empty request returns zero without establishing EOF.
            /// Empty requests may still call the underlying input or return an error.
            /// See [`qubit_io::Input::read_unchecked`] for the underlying contract.
            ///
            /// # Errors
            ///
            /// Returns the underlying input error.
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
            ///
            /// # Parameters
            ///
            /// - `output`: Destination byte buffer.
            ///
            /// # Returns
            ///
            /// Returns the number of bytes read; a successful read may be short.
            /// For a non-empty request, `Ok(0)` means no bytes were available in this
            /// call or the input reached its end; it does not promise permanent EOF.
            /// A successful empty request returns zero without establishing EOF.
            /// Empty requests may still call the underlying input or return an error.
            /// See [`qubit_io::Input::read`] for the underlying contract.
            ///
            /// # Errors
            ///
            /// Returns the underlying input error.
            #[inline(always)]
            pub fn read(&mut self, output: &mut [u8]) -> Result<usize> {
                Input::read(self, output)
            }

            /// Reads bytes into an indexed range until it is full or a read returns zero.
            ///
            /// # Parameters
            ///
            /// - `output`: Destination byte buffer.
            /// - `index`: Start of the writable range.
            /// - `count`: Maximum number of bytes to read.
            ///
            /// # Returns
            ///
            /// Returns the number of bytes read before the range is full or a read
            /// returns zero.
            ///
            /// # Errors
            ///
            /// Returns the underlying input error.
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

            /// Reads bytes into the full output slice until it is full or a read returns zero.
            ///
            /// # Parameters
            ///
            /// - `output`: Destination byte buffer.
            ///
            /// # Returns
            ///
            /// Returns the number of bytes read before the slice is full or a read
            /// returns zero.
            ///
            /// # Errors
            ///
            /// Returns the underlying input error.
            #[inline(always)]
            pub fn read_fully(&mut self, output: &mut [u8]) -> Result<usize> {
                Input::read_fully(self, output)
            }

            /// Reads bytes until the full output slice is filled.
            ///
            /// # Parameters
            ///
            /// - `output`: Destination byte buffer.
            ///
            /// # Errors
            ///
            /// Returns the underlying input error, including an unexpected-EOF
            /// error when the slice cannot be filled.
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
            ///
            /// # Returns
            ///
            /// Returns `true` when writes may be retained in an internal buffer.
            #[must_use]
            #[inline(always)]
            pub fn is_buffered(&self) -> bool {
                Output::is_buffered(self)
            }

            /// Writes up to `count` bytes from an indexed input range.
            ///
            /// # Parameters
            ///
            /// - `input`: Source byte buffer.
            /// - `index`: Start of the readable range.
            /// - `count`: Maximum number of bytes to write.
            ///
            /// # Returns
            ///
            /// Returns the number of bytes written.
            ///
            /// # Errors
            ///
            /// Returns the underlying output error.
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
            ///
            /// # Parameters
            ///
            /// - `input`: Source byte buffer.
            ///
            /// # Returns
            ///
            /// Returns the number of bytes written.
            ///
            /// # Errors
            ///
            /// Returns the underlying output error.
            #[inline(always)]
            pub fn write(&mut self, input: &[u8]) -> Result<usize> {
                Output::write(self, input)
            }

            /// Writes all bytes from an indexed input range.
            ///
            /// # Parameters
            ///
            /// - `input`: Source byte buffer.
            /// - `index`: Start of the readable range.
            /// - `count`: Number of bytes to write.
            ///
            /// # Errors
            ///
            /// Returns the underlying output error if the complete range cannot
            /// be written.
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
            ///
            /// # Parameters
            ///
            /// - `input`: Source byte buffer.
            ///
            /// # Errors
            ///
            /// Returns the underlying output error if the complete slice cannot
            /// be written.
            #[inline(always)]
            pub fn write_fully(&mut self, input: &[u8]) -> Result<()> {
                Output::write_fully(self, input)
            }

            /// Flushes buffered output.
            ///
            /// # Errors
            ///
            /// Returns the underlying output error if buffered bytes cannot be
            /// flushed.
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
            /// Seeks the wrapped byte stream; see [`Seekable::seek_to`] for
            /// the seek contract. Buffered wrappers interpret relative offsets
            /// and report positions relative to their logical stream position.
            /// A position query does not flush buffered output.
            ///
            /// # Parameters
            ///
            /// - `position`: Relative or absolute target position.
            ///
            /// # Returns
            ///
            /// Returns the resulting stream position in bytes. For buffered
            /// wrappers this is the logical position, including buffered data.
            ///
            /// # Errors
            ///
            /// Returns the underlying seek error.
            #[inline(always)]
            pub fn seek_to(&mut self, position: SeekFrom) -> Result<u64> {
                Seekable::seek_to(self, position)
            }
        }
    };
}
