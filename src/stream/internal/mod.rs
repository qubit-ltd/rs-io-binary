// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Internal codec-to-stream adapters used by buffered wrappers.

mod stream_codec_decode_error;
mod transcode_decode_input_ext;
mod transcode_encode_output_ext;

pub(super) use transcode_decode_input_ext::TranscodeDecodeInputExt;
pub(super) use transcode_encode_output_ext::TranscodeEncodeOutputExt;
