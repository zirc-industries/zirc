# Grammar (approximate)

This grammar is intentionally simplified.

Program
- program := item*
- item := function | stmt

Functions
- function := "fun" ident "(" param_list? ")" return_type? ":" block "end"
- param_list := ident ("," ident)*
- return_type := "(" type_name ")"
- type_name := "int" | "string" | "bool" | "unit"

Blocks and statements
- block := stmt*
- stmt := let_stmt | assign_stmt | if_stmt | while_stmt | return_stmt | break_stmt | continue_stmt | expr_stmt
- let_stmt := "let" ident "=" expr
- assign_stmt := ident "=" expr
- if_stmt := "if" expr ":" block ("else" ":" block)? "end"
- while_stmt := "while" expr ":" block "end"
- return_stmt := "return" expr?
- break_stmt := "break"
- continue_stmt := "continue"
- expr_stmt := expr

Expressions (precedence)
- expr := logical_or
- logical_or := logical_and ("||" logical_and)*
- logical_and := equality ("&&" equality)*
- equality := comparison (("==" | "!=") comparison)*
- comparison := term (("<" | "<=" | ">" | ">=") term)*
- term := factor (("+" | "-") factor)*
- factor := unary (("*" | "/") unary)*
- unary := "!" unary | primary
- primary := number | string | "true" | "false" | ident | call | "(" expr ")"
- call := ident "(" arg_list? ")"
- arg_list := expr ("," expr)*

Lexical
- Comments start with `~` to end-of-line
- Identifiers: `[A-Za-z_][A-Za-z0-9_]*`
- Numbers: decimal integers
- Strings: `"..."` with escapes `\n`, `\t`, `\r`, `\"`, `\\`

