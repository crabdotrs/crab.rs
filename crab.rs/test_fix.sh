#!/bin/bash
set -e
cd /Codes/Crab/crab.rs
cargo build --release 2>&1 | tee /Codes/Crab/build_output.txt
cd /Codes/Crab/examples/cookie_shop_api
rm -rf .crab_cache
/Codes/Crab/crab.rs/target/release/crab-cli build 2>&1 | tee /Codes/Crab/cli_output.txt
echo "=== cookie.rs ===" > /Codes/Crab/generated_output.txt
cat .crab_cache/cookie.rs >> /Codes/Crab/generated_output.txt
echo "" >> /Codes/Crab/generated_output.txt
echo "=== customer.rs ===" >> /Codes/Crab/generated_output.txt
cat .crab_cache/customer.rs >> /Codes/Crab/generated_output.txt
