# Contributing

Thank you for your interest in contributing to Zirc!

Getting started
- Install Rust (stable)
- Build and test the workspace:
  - `cargo build --workspace`
  - `cargo test -p zirc-cli`
- Try the REPL and examples:
  - `cargo run -p zirc-cli`
  - `cargo run -p zirc-cli -- examples/hello.zirc`

Development
- Crates are split by responsibility: syntax, lexer, parser, interpreter, and CLI
- Keep changes small and focused; prefer separate PRs for unrelated changes
- Add or update examples under examples/ where helpful
- Add tests for significant changes (CLI tests live at crates/zirc-cli/tests)

Style
- Rustfmt and Clippy are recommended (optional):
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets`

Pull requests
- Describe the problem and solution
- Link related issues
- Ensure CI passes

Code of conduct
- Be respectful and constructive. We’re all here to learn and build something cool.

