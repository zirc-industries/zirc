//! Bytecode IR for the Zirc programming language.
//! 
//! This crate defines a simple stack-based bytecode, a program container,
//! and value representation used by the Zirc VM backend.

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Str(String),
    Bool(bool),
    List(Vec<Value>),
    Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Builtin {
    Show,
    ShowF,
    Prompt,
    Rf,
    Wf,
}

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

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub arity: usize,
    pub local_count: usize,
    pub code: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
    pub main: Function,
}

