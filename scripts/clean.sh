#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "Cleaning build artifacts..."

rm -rf crab.rs/target
rm -rf tools/*/target
rm -rf examples/*/.crab_cache
rm -rf test_project/.crab_cache

echo "Clean complete."
