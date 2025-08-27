# Zirc Language

This document describes the current state of the Zirc language and serves as a beginner-friendly reference.

Comments
- Start-of-line comments use `~`. Everything after `~` on a line is ignored.

Values and types
- int, string, bool, list, unit (special value returned when no value is produced)
- Literals: `123`, `"hello"`, `true`, `false`, `[1, 2, 3]`

Variables
- Declare with `let` in the current scope: `let x = 1`
- Reassign in the same scope: `x = x + 1`
- Assignment to an undeclared variable is an error (declare first with `let`)

Expressions
- Arithmetic: `+ - * /`
- Comparisons: `== != < <= > >=` (return bool)
- Logical: `&& || !` (short-circuiting)
- Group with parentheses: `(a + b) * c`

Functions
- Define with `fun name(params) (ret_type): ... end`
- Return with `return expr` or use the last expression value (implicit) if you prefer
- Parameters currently untyped; return type supports: `(int)`, `(string)`, `(bool)`, `(unit)`

Example
```text
fun greet(name) (unit):
  showf("Hello, %s!", name)
end

let x = 2 + 3
showf("x == 5? %s", x == 5)
```

Control flow
- If/else:
```text
if cond:
  ~ then block
else:
  ~ else block
end
```
- While loop with `break` and `continue`:
```text
let i = 0
while i < 5:
  i = i + 1
  if i == 3:
    continue
  end
  if i == 4:
    break
  end
  showf("%d", i)
end
```

Lists
- Literals with square brackets: `[a, b, c]`
- Indexing with `list[index]` (0-based). Indexing strings returns a 1-character string.

Builtin functions
- `showf(fmt, ...)` prints a formatted string to stdout, supporting:
  - `%d` for integers
  - `%s` for strings and booleans

REPL tips
- Start: `cargo run -p zirc-cli`
- Commands: `:help`, `:vars`, `:funcs`, `:quit`

Notes and limits
- Runtime type checks for function return types
- Parameters and variables are dynamically typed (runtime validated)
- Indentation is not significant; blocks are controlled by keywords and `end`

