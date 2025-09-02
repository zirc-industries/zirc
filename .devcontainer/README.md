# Dev Container for Zirc

## What’s included
- Base image: `mcr.microsoft.com/devcontainers/base:ubuntu`
- Rust via devcontainer features (rustup + stable toolchain)
- `rustfmt`, `clippy` components
- VS Code recommendations: rust-analyzer, TOML support, crates plugin, EditorConfig
- Cargo registry/git volumes to speed up rebuilds
- `postCreateCommand`: builds the workspace once to warm caches

## How to use with VS Code
1. Install the "Dev Containers" extension
2. Open the repository root in VS Code
3. When prompted, "Reopen in Container" (or run: Command Palette → Dev Containers: Reopen in Container)
4. The container will build, Rust toolchain will be provisioned, and the workspace will be built automatically

After it starts:
- Use the integrated terminal to run `cargo build`, `cargo test`, or `cargo run -p zirc-cli --bin zirc-cli -- examples/hello.zirc`
- The recommended extensions and rust-analyzer settings are applied automatically

## How to use with GitHub Codespaces
1. Click the "Code" button on GitHub → "Open with Codespaces" → "New codespace"
2. Your codespace will start using this `.devcontainer` configuration

## Customization
If you need additional packages or tools, you can either:
- Add another feature in `devcontainer.json` (see https://github.com/devcontainers/features)
- Or switch to a custom Dockerfile and reference it via the `build.dockerfile` field in `devcontainer.json`

Example additions:
- Profiler/debugger packages (`lldb`, `gdb`)
- LLVM/Cranelift toolchains if you add native codegen

## Notes
- The container uses cargo sparse registry by default for faster dependency resolution (`CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse`).
- The registry/git directories are mounted as Docker volumes to avoid re-downloading on each rebuild.

