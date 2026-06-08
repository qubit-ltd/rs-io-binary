// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Byte-buffered output used by binary stream adapters.
pub(crate) type BufferedOutput<W> = qubit_codec::BufferedEncodeOutput<W>;
