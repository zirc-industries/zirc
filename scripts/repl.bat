@echo off
REM REPL launcher script for Zirc language (Windows)

echo [REPL] Starting Zirc REPL...
cargo run -q -p zirc-cli --bin zirc-cli -- --backend
