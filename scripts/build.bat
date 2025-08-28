@echo off
REM Build script for Zirc language (Windows)

echo [BUILD] Building Zirc workspace...
cargo build --workspace --release
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Build failed!
    exit /b 1
)

echo [BUILD] Build completed successfully!
echo [INFO] Binaries available at: target/release/zirc-cli.exe and target/release/zirc-repl.exe
