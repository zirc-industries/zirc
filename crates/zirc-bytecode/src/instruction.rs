//! Instruction set for Zirc bytecode.

use crate::builtin::Builtin;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // Constants
    PushInt(i64),
    PushStr(String),
    PushBool(bool),
    PushUnit,

    // Data structures
    MakeList(usize), // pops N items -> pushes List in original order
    Index,           // pops index, base -> pushes element

    // Locals
    LoadLocal(u16),
    StoreLocal(u16),

    // Stack
    Pop,

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,

    // Comparisons
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical
    Not,
    // Short-circuit handled with jumps

    // Control flow (absolute instruction index targets)
    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize),

    // Calls
    Call(usize, usize),     // (function_index, arg_count)
    BuiltinCall(Builtin, usize),
    Return,                 // expects a value on stack (push Unit beforehand if none)

    // Program control
    Halt,
}

