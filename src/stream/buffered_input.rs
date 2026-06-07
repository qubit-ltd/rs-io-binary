// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::io::{
    Error,
    ErrorKind,
    Read,
    Result,
    Seek,
    SeekFrom,
};

use qubit_io::{
    BufferedByteInput,
    DEFAULT_BUFFER_CAPACITY,
};

/// Buffered input core shared by codec-oriented readers.
///
/// This type delegates byte buffering to [`BufferedByteInput`] and exposes
/// codec-oriented helpers that decode fixed-width or variable-width values
/// directly from the unread byte window.
#[derive(Debug)]
pub(crate) struct BufferedInput<R> {
    input: BufferedByteInput<R>,
}

impl<R> BufferedInput<R> {
    /// Creates a buffered input core with the default capacity.
    ///
    /// # Arguments
    ///
    /// * `inner` - The input object wrapped by this buffer.
    ///
    /// # Returns
    ///
    /// A new buffered input whose internal buffer has at least
    /// [`DEFAULT_BUFFER_CAPACITY`] bytes.
    #[inline]
    pub(crate) fn new(inner: R) -> Self {
        Self::with_capacity(inner, DEFAULT_BUFFER_CAPACITY)
    }

    /// Creates a buffered input core with at least the requested capacity.
    ///
    /// The actual capacity is raised to `1` when the requested value is `0`.
    ///
    /// # Arguments
    ///
    /// * `inner` - The input object wrapped by this buffer.
    /// * `capacity` - The requested internal buffer capacity, in bytes.
    ///
    /// # Returns
    ///
    /// A new buffered input whose internal buffer capacity is
    /// `capacity.max(1)`.
    #[inline]
    pub(crate) fn with_capacity(inner: R, capacity: usize) -> Self {
        Self {
            input: BufferedByteInput::with_capacity(inner, capacity),
        }
    }

    /// Returns a shared reference to the wrapped input object.
    ///
    /// # Returns
    ///
    /// A shared reference to the inner input object.
    #[inline]
    pub(crate) const fn inner(&self) -> &R {
        self.input.inner()
    }

    /// Consumes this buffered input and returns the wrapped input object.
    ///
    /// Any unread bytes currently held in the internal buffer are discarded.
    ///
    /// # Returns
    ///
    /// The wrapped input object.
    #[inline]
    pub(crate) fn into_inner(self) -> R {
        let (inner, _) = self.input.into_parts();
        inner
    }
}

impl<R> Read for BufferedInput<R>
where
    R: Read,
{
    /// Reads bytes through the internal buffer.
    #[inline]
    fn read(&mut self, output: &mut [u8]) -> Result<usize> {
        self.read_raw(output)
    }
}

impl<R> BufferedInput<R>
where
    R: Read,
{
    /// Reads one fixed-width value directly from the internal buffer.
    ///
    /// The method ensures that `N` bytes are buffered, calls `decode` at the
    /// current buffer position, and then advances the position by `N` bytes.
    ///
    /// # Type Parameters
    ///
    /// * `N` - The exact number of bytes consumed by the fixed-width value.
    /// * `T` - The decoded value type.
    /// * `F` - The decoder function type.
    ///
    /// # Arguments
    ///
    /// * `decode` - Function that decodes a value from the internal buffer and
    ///   the starting index of the value.
    ///
    /// # Returns
    ///
    /// The decoded value.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::UnexpectedEof`] if EOF is reached before `N` bytes
    /// are available. Returns any non-interrupted I/O error produced by the
    /// wrapped reader while refilling the buffer.
    #[inline]
    pub(crate) fn read_fixed<const N: usize, T, F>(
        &mut self,
        decode: F,
    ) -> Result<T>
    where
        F: FnOnce(&[u8], usize) -> T,
    {
        self.input.ensure_available(N)?;
        let (bytes, index, available) = self.input.unread_raw_parts();
        debug_assert!(available >= N, "requested range is not buffered");
        let value = decode(bytes, index);
        // Keep the cursor update based on the saved `index` instead of
        // writing `self.position += N`. This fixed-width read path is hot
        // enough that the exact expression shape has measured impact: using
        // `index + N` states the real invariant directly, namely that the
        // cursor advances from the position that was checked before `decode`
        // ran. Re-reading and incrementing `self.position` after the callback
        // makes the optimizer reason about the field again and was slower in
        // the production-style binary read benchmark.
        // SAFETY: `ensure_available` proved that at least `N` bytes are
        // readable from the current cursor.
        unsafe {
            self.input.consume_unchecked(N);
        }
        Ok(value)
    }

    /// Reads one variable-width value while the decoder scans available bytes.
    ///
    /// The method calls `decode_available` with the unread byte range currently
    /// available in the internal buffer, capped at `N`. The decoder must
    /// scan for the variable-width terminator and decode the payload in the
    /// same pass. This avoids the older buffered path that first scanned
    /// for a terminator and then asked the codec to scan and decode the
    /// same bytes again.
    ///
    /// # Type Parameters
    ///
    /// * `N` - The maximum number of bytes that can belong to the
    ///   variable-width payload.
    /// * `T` - The decoded value type.
    /// * `E` - The decoder-specific error type.
    /// * `F` - The decoder function type.
    /// * `M` - The error mapping function type.
    ///
    /// # Arguments
    ///
    /// * `decode_available` - Function that decodes from currently buffered
    ///   bytes. It returns `Ok(None)` when more input is needed and
    ///   `Err((error, consumed))` when invalid bytes should be consumed before
    ///   reporting the mapped error.
    /// * `map_error` - Function that converts decoder errors into
    ///   [`std::io::Error`].
    ///
    /// # Returns
    ///
    /// The decoded value.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::UnexpectedEof`] if EOF is reached before a complete
    /// or maximum-width payload is buffered. Returns any non-interrupted I/O
    /// error produced by the wrapped reader while refilling the buffer. Returns
    /// `map_error(error)` when `decode_available` rejects the buffered payload.
    #[inline(always)]
    pub(crate) fn read_variable_decoded<const N: usize, T, E, F, M>(
        &mut self,
        mut decode_available: F,
        map_error: M,
    ) -> Result<T>
    where
        F: FnMut(
            &[u8],
            usize,
            usize,
        )
            -> std::result::Result<Option<(T, usize)>, (E, usize)>,
        M: FnOnce(E) -> Error,
    {
        debug_assert!(
            N <= self.input.capacity(),
            "variable payload length exceeds buffer capacity"
        );
        loop {
            let available = self.input.available().min(N);
            if available > 0 {
                let (bytes, index, _) = self.input.unread_raw_parts();
                match decode_available(bytes, index, available) {
                    Ok(Some((value, consumed))) => {
                        debug_assert!(
                            consumed > 0,
                            "decoded payload consumed no bytes"
                        );
                        debug_assert!(
                            consumed <= available,
                            "decoded beyond available bytes"
                        );
                        // SAFETY: The decoder reported a consumed byte count
                        // within the current unread window.
                        unsafe {
                            self.input.consume_unchecked(consumed);
                        }
                        return Ok(value);
                    }
                    Ok(None) => {
                        debug_assert!(
                            available < N,
                            "decoder must reject maximum-width unterminated payload"
                        );
                    }
                    Err((error, consumed)) => {
                        debug_assert!(
                            consumed > 0,
                            "invalid payload consumed no bytes"
                        );
                        debug_assert!(
                            consumed <= available,
                            "invalid payload exceeded buffer"
                        );
                        // SAFETY: The decoder reported a consumed byte count
                        // within the current unread window.
                        unsafe {
                            self.input.consume_unchecked(consumed);
                        }
                        return Err(map_error(error));
                    }
                }
            }
            if !self.input.fill_more()? {
                let available = self.input.available();
                // SAFETY: `available` is the current unread byte count.
                unsafe {
                    self.input.consume_unchecked(available);
                }
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "failed to fill whole buffer",
                ));
            }
        }
    }

    /// Reads raw bytes through the internal buffer.
    ///
    /// If the internal buffer is empty and `output` is at least as large as the
    /// buffer, the read is delegated directly to the wrapped reader to avoid an
    /// unnecessary copy. Otherwise, bytes are served from the internal buffer.
    ///
    /// # Arguments
    ///
    /// * `output` - Destination slice that receives the bytes read.
    ///
    /// # Returns
    ///
    /// The number of bytes written to `output`. A return value of `0` means
    /// that `output` was empty or EOF was reached before any bytes were
    /// read.
    ///
    /// # Errors
    ///
    /// Returns any I/O error produced by the wrapped reader. Refills and
    /// large direct reads follow the delegated [`BufferedByteInput`] behavior.
    pub(crate) fn read_raw(&mut self, output: &mut [u8]) -> Result<usize> {
        self.input.read(output)
    }

    /// Seeks the wrapped reader and discards buffered bytes after success.
    ///
    /// For [`SeekFrom::Current`], the offset is adjusted by the number of
    /// unread bytes already buffered, so seeking is relative to the logical
    /// position observed by callers of this buffered input.
    ///
    /// # Arguments
    ///
    /// * `position` - The target seek position.
    ///
    /// # Returns
    ///
    /// The new absolute stream position reported by the wrapped reader.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::InvalidInput`] if a [`SeekFrom::Current`] offset
    /// cannot be adjusted by the unread buffered byte count. Returns any seek
    /// error produced by the wrapped reader.
    pub(crate) fn seek_raw(&mut self, position: SeekFrom) -> Result<u64>
    where
        R: Seek,
    {
        self.input.seek(position)
    }
}
