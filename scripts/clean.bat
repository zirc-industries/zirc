@echo off
REM Clean script for Zirc language (Windows)

echo [CLEAN] Cleaning build artifacts...
cargo clean
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Clean failed!
    exit /b 1
)

echo [CLEAN] Build artifacts cleaned successfully!
