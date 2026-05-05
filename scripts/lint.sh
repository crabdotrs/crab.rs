#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../crab.rs"

echo "Running clippy..."
cargo clippy --workspace -- -D warnings

echo "Running cargo check..."
cargo check --workspace

echo "Linting complete."
