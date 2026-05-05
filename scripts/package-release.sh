#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:-0.1.0}"
TARGET_DIR="$(dirname "$0")/../releases"

cd "$(dirname "$0")/../crab.rs"

echo "Packaging Crab v$VERSION release..."
mkdir -p "$TARGET_DIR"

cargo build --release --workspace

for triple in x86_64-unknown-linux-gnu x86_64-apple-darwin aarch64-apple-darwin; do
  if rustup target add "$triple" 2>/dev/null; then
    echo "Building for $triple..."
    cargo build --release --target "$triple" -p crab-cli || echo "Skipping $triple"
    if [ -f "target/$triple/release/crab" ]; then
      tar czf "$TARGET_DIR/crab-$VERSION-$triple.tar.gz" -C "target/$triple/release" crab
      echo "Packaged: crab-$VERSION-$triple.tar.gz"
    fi
  fi
done

echo "Release packages in: $TARGET_DIR"
