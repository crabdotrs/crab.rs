#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../../crab.rs"

echo "=== Running CI Tests ==="

echo "Step 1: Format check..."
cargo fmt --all -- --check

echo "Step 2: Clippy lint..."
cargo clippy --workspace -- -D warnings

echo "Step 3: Build workspace..."
cargo build --workspace

echo "Step 4: Run tests..."
cargo test --workspace

echo "Step 5: Build release..."
cargo build --release --workspace

echo "Step 6: Test cookie_shop_api example..."
cd ../examples/cookie_shop_api
../../crab.rs/target/release/crab build
../../crab.rs/target/release/crab run

echo "=== All CI Tests Passed ==="
