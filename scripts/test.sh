#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../crab.rs"

echo "Running all tests..."
cargo test --workspace

echo "Testing cookie_shop_api example..."
cd ../examples/cookie_shop_api
../../crab.rs/target/release/crab build

echo "All tests passed."
