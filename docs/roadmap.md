# Roadmap

This is a living document for upcoming features and improvements.

Core language
- [x] let, assignments, and basic types (int, string, bool, unit)
- [x] Functions with return types, runtime enforcement
- [x] if/else, while, break, continue
- [x] Logical operators: &&, ||, !
- [ ] Typed parameters: fun f(a: int) (int)
- [ ] Variable annotations: let x: int = 0
- [ ] Else-if sugar: elif or else if
- [ ] Standard library: print(value), len(string), to_string(x), input()

Tooling and quality
- [x] Workspace crates, examples, and tests
- [x] GitHub Actions CI across OS matrix
- [ ] Formatter/linter integration (rustfmt/clippy checks in CI)
- [ ] Benchmarks and performance profiling

Docs and UX
- [x] Language reference and grammar docs
- [x] REPL colors and commands (:vars, :funcs)
- [ ] Tutorial series in docs/tutorials/
- [ ] Website/docs hosting (e.g., GitHub Pages)

Stretch goals
- [ ] Bytecode VM or JIT backend
- [ ] Module system and imports
- [ ] Static type checker

