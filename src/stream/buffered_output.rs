// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::{
    Result,
    Seek,
    SeekFrom,
    Write,
};

use qubit_io::{
    BufferedByteOutput,
    DEFAULT_BUFFER_CAPACITY,
};

/// Buffered output core shared by codec-oriented writers.
///
/// This type keeps a fixed-size byte buffer in front of an underlying writer so
/// small encoded values can be accumulated before they are written to the I/O
/// target.  Large raw writes may bypass the buffer after pending buffered bytes
/// have been flushed.
#[derive(Debug)]
pub(crate) struct BufferedOutput<W> {
    output: BufferedByteOutput<W>,
}

impl<W> BufferedOutput<W> {
    /// Creates a buffered output core with the default capacity.
    ///
    /// # Parameters
    ///
    /// * `inner` - The writer that receives bytes when the internal buffer is
    ///   flushed.
    ///
    /// # Returns
    ///
    /// A new buffered output wrapper using [`DEFAULT_BUFFER_CAPACITY`].
    #[inline]
    pub(crate) fn new(inner: W) -> Self {
        Self::with_capacity(inner, DEFAULT_BUFFER_CAPACITY)
    }

    /// Creates a buffered output core with at least the requested capacity.
    ///
    /// # Parameters
    ///
    /// * `inner` - The writer that receives bytes when the internal buffer is
    ///   flushed.
    /// * `capacity` - The requested internal buffer capacity in bytes.
    ///
    /// # Returns
    ///
    /// A new buffered output wrapper whose actual buffer capacity is
    /// `capacity.max(1)`.
    #[inline]
    pub(crate) fn with_capacity(inner: W, capacity: usize) -> Self {
        Self {
            output: BufferedByteOutput::with_capacity(inner, capacity),
        }
    }

    /// Returns a shared reference to the wrapped writer.
    ///
    /// # Returns
    ///
    /// An immutable reference to the underlying writer.  Pending bytes may
    /// still be present in the internal buffer and are not flushed by this
    /// method.
    #[inline]
    pub(crate) const fn inner(&self) -> &W {
        self.output.inner()
    }

    /// Returns the unused capacity in the internal buffer.
    ///
    /// # Returns
    ///
    /// The number of bytes that can still be appended to the internal buffer
    /// before it must be flushed.
    #[inline]
    fn spare_capacity(&self) -> usize {
        self.output.spare_capacity()
    }
}

impl<W> Write for BufferedOutput<W>
where
    W: Write,
{
    /// Writes bytes through the internal buffer.
    #[inline]
    fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        self.write_raw(buffer)
    }

    /// Writes all bytes through the internal buffer.
    #[inline]
    fn write_all(&mut self, buffer: &[u8]) -> Result<()> {
        self.write_all_buffered(buffer)
    }

    /// Flushes the internal buffer and then the wrapped writer.
    #[inline]
    fn flush(&mut self) -> Result<()> {
        self.flush_all()
    }
}

impl<W> BufferedOutput<W>
where
    W: Write,
{
    /// Consumes this buffered output after flushing pending bytes.
    ///
    /// # Returns
    ///
    /// The wrapped writer after all pending buffered bytes have been written.
    ///
    /// # Errors
    ///
    /// Returns any non-interrupted I/O error produced while flushing buffered
    /// bytes.  Also returns [`std::io::ErrorKind::WriteZero`] if the wrapped
    /// writer reports that zero bytes were written before the buffer is
    /// drained.
    #[inline]
    pub(crate) fn into_inner(mut self) -> Result<W> {
        self.output.flush_buffer()?;
        let (inner, pending) = self.output.into_parts();
        debug_assert!(pending.is_empty(), "buffer still has pending bytes");
        Ok(inner)
    }

    /// Encodes one value directly into the internal buffer.
    ///
    /// The encoder receives the entire internal buffer, the current write
    /// position, and the value to encode.  It must write no more than `max_len`
    /// bytes starting at that position and return the number of bytes written.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The value type accepted by the encoder.
    /// * `F` - The encoder callback type.
    ///
    /// # Parameters
    ///
    /// * `max_len` - The maximum number of bytes that the encoder may write.
    /// * `value` - The value passed to the encoder.
    /// * `encode` - The callback that serializes `value` into the internal
    ///   buffer.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the encoded bytes have been reserved in the buffer and
    /// the buffered length has been advanced.
    ///
    /// # Errors
    ///
    /// Returns any non-interrupted I/O error produced while flushing buffered
    /// bytes to make room for `max_len`. Also returns
    /// [`std::io::ErrorKind::WriteZero`] if the wrapped writer reports that
    /// zero bytes were written before the buffer is drained.
    #[inline]
    pub(crate) fn write_encoded<T, F>(
        &mut self,
        max_len: usize,
        value: T,
        encode: F,
    ) -> Result<()>
    where
        F: FnOnce(&mut [u8], usize, T) -> usize,
    {
        if self.spare_capacity() < max_len {
            self.output.ensure_spare_capacity(max_len)?;
        }
        let (bytes, index, available) = self.output.spare_raw_parts_mut();
        debug_assert!(available >= max_len, "insufficient spare buffer");
        let written = encode(bytes, index, value);
        debug_assert!(
            written <= max_len,
            "encoder wrote beyond declared maximum length"
        );
        // Keep the delegated buffer cursor update based on the range checked
        // before the encoder ran. On the fixed-width hot path, avoiding a
        // second round of cursor reasoning after the mutable buffer borrow has
        // measured impact in the production-style binary write benchmark.
        // SAFETY: The spare-capacity check and codec contract guarantee that
        // `written` initialized bytes fit in the spare output window.
        unsafe {
            self.output.advance_unchecked(written);
        }
        Ok(())
    }

    /// Encodes one fixed-width value directly into the internal buffer.
    ///
    /// The encoder receives the entire internal buffer, the current write
    /// position, and the value to encode.  It must write exactly `N` bytes
    /// starting at that position.
    ///
    /// # Type Parameters
    ///
    /// * `N` - The fixed encoded width, in bytes.
    /// * `T` - The value type accepted by the encoder.
    /// * `F` - The encoder callback type.
    ///
    /// # Parameters
    ///
    /// * `value` - The value passed to the encoder.
    /// * `encode` - The callback that serializes `value` into the internal
    ///   buffer.
    ///
    /// # Returns
    ///
    /// `Ok(())` after `N` bytes have been reserved in the buffer and the
    /// buffered length has been advanced.
    ///
    /// # Errors
    ///
    /// Returns any non-interrupted I/O error produced while flushing buffered
    /// bytes to make room for `N`. Also returns
    /// [`std::io::ErrorKind::WriteZero`] if the wrapped writer reports that
    /// zero bytes were written before the buffer is drained.
    #[inline]
    pub(crate) fn write_fixed<const N: usize, T, F>(
        &mut self,
        value: T,
        encode: F,
    ) -> Result<()>
    where
        F: FnOnce(&mut [u8], usize, T),
    {
        if self.spare_capacity() < N {
            self.output.ensure_spare_capacity(N)?;
        }
        let (bytes, index, available) = self.output.spare_raw_parts_mut();
        debug_assert!(available >= N, "insufficient spare buffer");
        encode(bytes, index, value);
        // SAFETY: The spare-capacity check and fixed-width codec contract
        // guarantee that `N` initialized bytes fit in the spare output window.
        unsafe {
            self.output.advance_unchecked(N);
        }
        Ok(())
    }

    /// Writes raw bytes through the internal buffer.
    ///
    /// Small inputs are appended to the internal buffer.  Inputs that do not
    /// fit may flush the buffer first, and inputs at least as large as the
    /// buffer may be written directly to the wrapped writer.
    ///
    /// # Parameters
    ///
    /// * `input` - The bytes to write.
    ///
    /// # Returns
    ///
    /// `Ok(())` after all bytes from `input` have been accepted.
    ///
    /// # Errors
    ///
    /// Returns any I/O error produced while flushing pending bytes or writing a
    /// large input directly to the wrapped writer. Flush failures include
    /// [`std::io::ErrorKind::WriteZero`] if the writer reports that zero bytes
    /// were written before the buffer is drained.
    #[inline]
    pub(crate) fn write_all_buffered(&mut self, input: &[u8]) -> Result<()> {
        self.output.write_all(input)
    }

    /// Flushes buffered bytes and then flushes the wrapped writer.
    ///
    /// # Returns
    ///
    /// `Ok(())` once pending buffered bytes have been written and the wrapped
    /// writer's own flush operation succeeds.
    ///
    /// # Errors
    ///
    /// Returns any non-interrupted I/O error produced while flushing buffered
    /// bytes, [`std::io::ErrorKind::WriteZero`] if the wrapped writer cannot
    /// make progress while draining the buffer, or any error returned by
    /// [`Write::flush`] on the wrapped writer.
    pub(crate) fn flush_all(&mut self) -> Result<()> {
        self.output.flush()
    }

    /// Writes raw bytes and reports the accepted byte count.
    ///
    /// This is the buffered implementation for [`Write::write`]-style callers.
    /// Small inputs are appended to the buffer and reported as fully accepted;
    /// large inputs may be delegated to the wrapped writer after pending bytes
    /// are flushed.
    ///
    /// # Parameters
    ///
    /// * `input` - The bytes to write.
    ///
    /// # Returns
    ///
    /// The number of bytes accepted.  Buffered writes return `input.len()`;
    /// direct writes return the byte count reported by the wrapped writer.
    ///
    /// # Errors
    ///
    /// Returns any I/O error produced while flushing pending bytes or writing a
    /// large input directly to the wrapped writer. Flush failures include
    /// [`std::io::ErrorKind::WriteZero`] if the writer reports that zero bytes
    /// were written before the buffer is drained.
    #[inline]
    pub(crate) fn write_raw(&mut self, input: &[u8]) -> Result<usize> {
        self.output.write(input)
    }

    /// Flushes pending bytes before seeking the wrapped writer.
    ///
    /// # Parameters
    ///
    /// * `position` - The target seek position passed to the wrapped writer.
    ///
    /// # Returns
    ///
    /// The new stream position reported by the wrapped writer.
    ///
    /// # Errors
    ///
    /// Returns any non-interrupted I/O error produced while flushing buffered
    /// bytes, [`std::io::ErrorKind::WriteZero`] if the wrapped writer cannot
    /// make progress while draining the buffer, or any error returned by
    /// [`Seek::seek`] on the wrapped writer.
    pub(crate) fn seek_raw(&mut self, position: SeekFrom) -> Result<u64>
    where
        W: Seek,
    {
        self.output.seek(position)
    }
}
