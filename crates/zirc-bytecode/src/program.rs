//! Program components for Zirc bytecode.

use crate::instruction::Instruction;

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

