#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
cd "$project_root"
cargo test --locked --manifest-path tests/fixtures/readme_quick_start/Cargo.toml --test readme_extractor_tests
cargo run --locked --manifest-path tests/fixtures/readme_quick_start/Cargo.toml
cargo run --locked --manifest-path tests/fixtures/user_guide/Cargo.toml
