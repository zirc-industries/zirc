@echo off
REM Test script for Zirc language (Windows)

echo [TEST] Running tests for Zirc CLI...
cargo test -p zirc-cli
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Tests failed!
    exit /b 1
)

echo [TEST] Running all workspace tests...
cargo test --workspace
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Workspace tests failed!
    exit /b 1
)

echo [TEST] All tests passed!
