#!/usr/bin/env bash
set -euo pipefail

echo "Installing Rust toolchain..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

echo "Installing Rust 2024 edition components..."
rustup update stable
rustup component add clippy rustfmt

echo "Verifying installation..."
rustc --version
cargo --version
echo "Dependencies installed successfully."
