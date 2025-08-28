#!/bin/bash
# Test script for Zirc language (Unix)

set -e

echo "[TEST] Running tests for Zirc CLI..."
cargo test -p zirc-cli

echo "[TEST] Running all workspace tests..."
cargo test --workspace

echo "[TEST] All tests passed!"
