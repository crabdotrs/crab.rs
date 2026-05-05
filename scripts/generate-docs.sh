#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../crab.rs"

echo "Generating Rust documentation..."
cargo doc --workspace --no-deps

echo "Documentation generated at:"
echo "  crab.rs/target/doc/index.html"
