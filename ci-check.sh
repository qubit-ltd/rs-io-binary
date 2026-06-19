#!/bin/bash
set -euo pipefail

PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
RS_IO_BINARY_COVERAGE_EXCLUDE='/stream/(buffered_leb128_writer|transcode_decode_input_ext|transcode_encode_output_ext)\.rs$'
if [ -n "${COVERAGE_EXTRA_EXCLUDE_REGEX:-}" ]; then
    COVERAGE_EXTRA_EXCLUDE_REGEX="(${COVERAGE_EXTRA_EXCLUDE_REGEX})|${RS_IO_BINARY_COVERAGE_EXCLUDE}"
else
    COVERAGE_EXTRA_EXCLUDE_REGEX="${RS_IO_BINARY_COVERAGE_EXCLUDE}"
fi
export COVERAGE_EXTRA_EXCLUDE_REGEX

exec env RS_CI_PROJECT_ROOT="$PROJECT_ROOT" "$PROJECT_ROOT/.rs-ci/ci-check.sh" "$@"
