//! Zirc interpreter: evaluates AST nodes with a simple tree-walking interpreter.
//!
//! This crate provides the runtime evaluation system for the Zirc programming language.
//! It implements a tree-walking interpreter that directly executes Abstract Syntax Tree (AST) nodes
//! produced by the parser.

pub mod value;
pub mod env;
pub mod flow;
pub mod interpreter;

pub use value::Value;
pub use env::Env;
pub use interpreter::{Interpreter, MemoryStats};
