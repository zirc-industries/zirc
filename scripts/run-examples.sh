#!/bin/bash
# Examples runner script for Zirc language (Unix)

echo "[EXAMPLES] Running all Zirc examples..."
echo

for file in examples/*.zirc; do
    echo "[RUN] $file"
    cargo run --bin zirc-cli -- "$file"
    echo
done

echo "[EXAMPLES] All examples completed!"
