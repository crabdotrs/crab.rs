#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../crab.rs"

echo "Building Crab transpiler..."
cargo build --release --workspace

echo "Build complete. Binary located at:"
echo "  crab.rs/target/release/crab"
