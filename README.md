# Zirc Language (zirc-lang)

A small programming language implemented in Rust with a REPL and examples. Built as a learning-friendly project with a simple lexer, parser, and tree-walking interpreter.

Highlights
- Written in Rust, split into workspace crates: syntax, lexer, parser, interpreter, and CLI
- REPL with friendly output, colors, and commands (:help, :vars, :funcs)
- Language features:
  - let declarations and assignments
  - Functions (fun name(params) (ret_type): ... end)
  - Integers, strings, booleans
  - Arithmetic (+, -, *, /)
  - Comparisons (==, !=, <, <=, >, >=)
  - Logical operators (&&, ||, !) with short-circuiting
  - if/else blocks
  - while loops with break and continue
  - Builtin showf("fmt", ...) with %d and %s

Quick start
- Prerequisites: Rust (stable)
- Build everything:
  - `cargo build --workspace`
- Run the REPL:
  - `cargo run -p zirc-cli`
- Run a file:
  - `cargo run -p zirc-cli -- examples/hello.zirc`

Examples
- See the examples/ directory for runnable programs. For instance:
  - `cargo run -p zirc-cli -- examples/factorial.zirc`

Language snapshot
```text
~ Comment
fun plus(a, b) (int):
  let result = a + b
  if result > 0:
    showf("sum = %d", result)
  else:
    showf("non-positive: %d", result)
  end
  return result
end

let x = plus(4, 6)
showf("x == 10? %s", x == 10)
```

Project layout
- crates/
  - zirc-syntax: AST, tokens, and common error types
  - zirc-lexer: tokenizer
  - zirc-parser: recursive-descent parser
  - zirc-interpreter: tree-walking interpreter and runtime
  - zirc-cli: command-line runner and REPL
- examples/: runnable .zirc sample programs
- docs/: language documentation and roadmap

Testing
- Integration tests live in zirc-cli/tests
- Run tests: `cargo test -p zirc-cli`

CI
- GitHub Actions builds and tests the workspace on Windows, Linux, and macOS (see .github/workflows/ci.yml)

Docs
- See docs/ for language reference, grammar, and roadmap

Contributing
- Issues and PRs are welcome! See docs/contributing.md for guidelines

License
- Choose a license (e.g., MIT/Apache-2.0). If you want, I can add a LICENSE file now.

