#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../crab.rs"

echo "Building and installing Crab CLI locally..."
cargo install --path crab-cli --force

echo "Crab CLI installed to ~/.cargo/bin/crab"
echo "Ensure ~/.cargo/bin is in your PATH."
