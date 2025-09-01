//! Bytecode IR for the Zirc programming language.
//!
//! This crate defines a simple stack-based bytecode, a program container,
//! and value representation used by the Zirc VM backend.

pub mod value;
pub mod builtin;
pub mod instruction;
pub mod program;

pub use value::Value;
pub use builtin::Builtin;
pub use instruction::Instruction;
pub use program::{Function, Program};

