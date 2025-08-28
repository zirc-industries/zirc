@echo off
REM Examples runner script for Zirc language (Windows)

echo [EXAMPLES] Running all Zirc examples...
echo.

for %%f in (examples\*.zirc) do (
    echo [RUN] %%f
    cargo run --bin zirc-cli -- %%f
    echo.
)

echo [EXAMPLES] All examples completed!
