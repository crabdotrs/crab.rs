#!/bin/bash
cd /Codes/Crab/examples/cookie_shop_api
rm -rf .crab_cache
/Codes/Crab/crab.rs/target/release/crab-cli build > /Codes/Crab/test_output.txt 2>&1
echo "=== Generated cookie.rs ===" >> /Codes/Crab/test_output.txt
cat .crab_cache/cookie.rs >> /Codes/Crab/test_output.txt
