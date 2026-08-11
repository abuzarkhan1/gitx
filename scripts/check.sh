#!/usr/bin/env bash
# GitX development check: formatting, lint, compilation, tests (docs/19 gates).
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> cargo fmt --check"
cargo fmt --check

echo "==> cargo clippy --workspace --all-targets -- -D warnings"
cargo clippy --workspace --all-targets -- -D warnings

echo "==> cargo check --workspace"
cargo check --workspace

echo "==> cargo test --workspace"
cargo test --workspace

echo "==> all checks passed"
