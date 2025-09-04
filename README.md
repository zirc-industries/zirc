# Zirc Programming Language

**Zirc** is a lightweight, high-performance programming language designed for easy and efficient code. It features clean syntax, optional type annotations, and dual execution backends for both development and production use.

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)]
[![Version](https://img.shields.io/badge/version-0.0.1--dev-orange.svg)]

## ✨ Features

- **🚀 Dual Execution**: Tree-walking interpreter for development, bytecode VM for production
- **🎯 Simple Syntax**: Clean, readable syntax inspired by modern languages
- **🔒 Type Safety**: Optional type annotations with runtime type checking
- **⚡ Interactive REPL**: Full-featured development environment
- **📝 Rich Diagnostics**: Helpful error messages with suggestions
- **🧪 Comprehensive Tests**: Extensive test coverage across all components
- **📚 Great Documentation**: Complete language specification and tutorials

## 🚀 Quick Start

### Hello World

```zirc
~ My first Zirc program
showf("Hello, %s!", "World")
```

### Function Example

```zirc
fun factorial(n: int) (int):
  if n <= 1:
    return 1
  else:
    return n * factorial(n - 1)
  end
end

for i in 0..6:
  showf("factorial(%d) = %d", i, factorial(i))
end
```

## 📦 Installation

### Prerequisites
- **Rust** 1.70+ (install from [rustup.rs](https://rustup.rs/))

### Build from Source

```bash
# Clone the repository
git clone https://github.com/zirc-industries/zirc.git
cd zirc-language

# Build the project
cargo build --release

# Run the REPL
cargo run --bin zirc-cli

# Run a program
cargo run --bin zirc-cli examples/basic/hello.zirc
```

## 🎮 Usage

### Command Line Interface

```bash
# Run a Zirc program
zirc program.zirc

# Start the interactive REPL
zirc

# Use the bytecode VM backend
zirc --backend vm program.zirc

# Show help
zirc --help
```

### Interactive REPL

```
⬢ let x = 42
⬢ showf("Answer: %d", x)
Answer: 42

⬢ fun double(n): n * 2 end
⬢ double(21)
42

⬢ :help     ~ Show available commands
⬢ :vars     ~ List variables
⬢ :quit     ~ Exit REPL
```

## 📖 Language Overview

### Data Types

```zirc
~ Basic types
let number: int = 42
let text: string = "Hello, Zirc!"
let flag: bool = true
let items: list = [1, 2, 3, 4, 5]
```

### Control Flow

```zirc
~ Conditionals
if score >= 90:
  showf("Grade: A")
else:
  showf("Grade: B")
end

~ Loops
for i in 0..10:
  showf("Number: %d", i)
end

while condition:
  ~ loop body
end
```

### Built-in Functions

```zirc
~ Input/Output
show("Simple output")
showf("Formatted: %s = %d", "answer", 42)
let input = prompt("Enter name: ")

~ String/List operations
let length = len("programming")     ~ Returns 11
let part = slice("hello", 1, 4)     ~ Returns "ell"

~ List manipulation (interpreter mode)
let numbers = [1, 2, 3]
push(numbers, 4)        ~ Add element
let last = pop(numbers) ~ Remove last element

~ File operations
let content = rf("file.txt")     ~ Read file
wf("output.txt", "Hello!")       ~ Write file
```

## 🏗️ Architecture

Zirc features a modular architecture with dual execution backends:

### Execution Backends

1. **Tree-Walking Interpreter** (`zirc-interpreter`)
   - Direct AST execution
   - Great for development and debugging
   - Rich runtime features (push/pop operations)

2. **Bytecode Virtual Machine** (`zirc-vm`)
   - Compiled bytecode execution
   - Optimized for performance
   - Suitable for production deployment

## 🧪 Testing

Zirc has comprehensive test coverage:

```bash
# Run all tests
cargo test --workspace

# Test specific components
cargo test --package zirc-lexer
cargo test --package zirc-parser
cargo test --package zirc-interpreter

# Test examples
cargo test --package zirc-cli
```

## 🎯 Examples

Explore the `examples/` directory for more programs:

- **[hello.zirc](examples/basic/hello.zirc)** - Hello World
- **[factorial.zirc](examples/others/factorial.zirc)** - Recursive factorial
- **[for_loop.zirc](examples/basic/for_loop.zirc)** - Loop examples
- **[test_builtins.zirc](examples/basic/test_builtins.zirc)** - Built-in functions

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Quick Start for Contributors

```bash
# Fork and clone the repository
git clone your-fork-url
cd zirc-language

# Build and test
cargo build
cargo test --workspace

# Make changes and submit a PR
git checkout -b feature/my-feature
# ... make changes ...
cargo test --workspace  # Ensure tests pass
git commit -m "feat: add my feature"
git push origin feature/my-feature
```

### Areas for Contribution

- 🐛 **Bug fixes** and error message improvements
- 📖 **Documentation** and tutorial enhancements
- ⚡ **Performance** optimizations
- 🧪 **Testing** and edge case coverage
- 🚀 **New features** and language enhancements

## 🗺️ Roadmap

### Upcoming Features
- 🔄 **Language Server Protocol** (LSP) support
- 🧩 **Advanced type system** with inference
- 🎨 **Code formatter** (zirc-fmt)
- 📦 **Package management** system
- 🌐 **WebAssembly** target
- 🔍 **Debugging tools** and profiler

## 📄 License

This project is dual-licensed under either:

- **MIT License** ([LICENSE-MIT](LICENSE-MIT))
- **Apache License 2.0** ([LICENSE-APACHE](LICENSE-APACHE))

at your option.

## 🙏 Acknowledgments

- Inspired by modern programming languages
- Built with the amazing Rust ecosystem
- Thanks to all contributors and users

---

**Happy coding with Zirc!** 🎉

For questions, issues, or contributions, visit our [GitHub repository](https://github.com/zirc-industries/zirc).
