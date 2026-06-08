// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Byte-buffered input used by binary stream adapters.
pub(crate) type BufferedInput<R> = qubit_codec::BufferedDecodeInput<R>;
