#!/bin/bash
# Build script for Zirc language (Unix)

set -e

echo "[BUILD] Building Zirc workspace..."
cargo build --workspace --release

echo "[BUILD] Build completed successfully!"
echo "[INFO] Binaries available at: target/release/zirc-cli and target/release/zirc-repl"
