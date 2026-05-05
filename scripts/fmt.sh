#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../crab.rs"

echo "Formatting Rust code..."
cargo fmt --all

echo "Formatting complete."
