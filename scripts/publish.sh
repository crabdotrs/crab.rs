#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../crab.rs"

echo "Publishing order: core -> lexer -> parser -> codegen -> ffi -> cli"

cd crab-core
cargo publish

cd ../crab-lexer
cargo publish

cd ../crab-parser
cargo publish

cd ../crab-codegen
cargo publish

cd ../crab-ffi
cargo publish

cd ../crab-cli
cargo publish

echo "All crates published successfully."
