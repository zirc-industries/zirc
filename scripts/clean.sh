#!/bin/bash
# Clean script for Zirc language (Unix)

set -e

echo "[CLEAN] Cleaning build artifacts..."
cargo clean

echo "[CLEAN] Build artifacts cleaned successfully!"
