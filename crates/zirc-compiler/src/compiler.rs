//! Bytecode compiler from AST to IR.

use std::collections::HashMap;

use zirc_bytecode::{Function as BcFunction, Instruction as BC, Program as BcProgram};
use zirc_syntax::ast::*;
use zirc_syntax::error::{Result, error};

use crate::builder::FuncBuilder;

pub struct Compiler {
    pub(crate) func_indices: HashMap<String, usize>,
    pub(crate) functions: Vec<BcFunction>,
}

impl Default for Compiler { fn default() -> Self { Self::new() } }

impl Compiler {
    pub fn new() -> Self {
        Self { func_indices: HashMap::new(), functions: Vec::new() }
    }

    pub fn function_names(&self) -> Vec<String> {
        let mut v: Vec<String> = self.func_indices.keys().cloned().collect();
        v.sort();
        v
    }

    pub fn compile(&mut self, program: Program) -> Result<BcProgram> {
        // First pass: collect function names to assign indices
        for item in &program.items {
            if let Item::Function(f) = item {
                if self.func_indices.contains_key(&f.name) { return error(format!("Duplicate function '{}'", f.name)); }
                let idx = self.functions.len();
                self.func_indices.insert(f.name.clone(), idx);
                self.functions.push(BcFunction { name: f.name.clone(), arity: f.params.len(), local_count: 0, code: Vec::new() });
            }
        }
        // Second pass: compile functions
        for item in &program.items {
            if let Item::Function(f) = item {
                let idx = *self.func_indices.get(&f.name).unwrap();
                let compiled = self.compile_function(f)?;
                self.functions[idx] = compiled;
            }
        }
        // Compile main (top-level statements)
        let mut main_builder = FuncBuilder::new("__main".to_string(), 0, true);
        for item in program.items.into_iter() {
            if let Item::Stmt(s) = item { main_builder.emit_stmt(self, &s)?; }
        }
        main_builder.emit(BC::Halt);
        let main = main_builder.finish();
        Ok(BcProgram { functions: self.functions.clone(), main })
    }

    fn compile_function(&mut self, f: &Function) -> Result<BcFunction> {
        let mut b = FuncBuilder::new(f.name.clone(), f.params.len(), false);
        for p in &f.params { b.declare_param(p.name.clone())?; }
        for s in &f.body { b.emit_stmt(self, s)?; }
        b.emit(BC::PushUnit);
        b.emit(BC::Return);
        Ok(b.finish())
    }
}

pub(crate) fn builtin_of(name: &str) -> Option<zirc_bytecode::Builtin> {
    match name {
        "show" => Some(zirc_bytecode::Builtin::Show),
        "showf" => Some(zirc_bytecode::Builtin::ShowF),
        "prompt" => Some(zirc_bytecode::Builtin::Prompt),
        "rf" => Some(zirc_bytecode::Builtin::Rf),
        "wf" => Some(zirc_bytecode::Builtin::Wf),
        "len" => Some(zirc_bytecode::Builtin::Len),
        "push" => Some(zirc_bytecode::Builtin::Push),
        "pop" => Some(zirc_bytecode::Builtin::Pop),
        "slice" => Some(zirc_bytecode::Builtin::Slice),
        // Mathematical functions
        "abs" => Some(zirc_bytecode::Builtin::Abs),
        "min" => Some(zirc_bytecode::Builtin::Min),
        "max" => Some(zirc_bytecode::Builtin::Max),
        "pow" => Some(zirc_bytecode::Builtin::Pow),
        "sqrt" => Some(zirc_bytecode::Builtin::Sqrt),
        // TODO: check if hex/bin need special handling here or move separately
        "bin" => Some(zirc_bytecode::Builtin::Bin),
        "hex" => Some(zirc_bytecode::Builtin::Hex),
        // String functions
        "upper" => Some(zirc_bytecode::Builtin::Upper),
        "lower" => Some(zirc_bytecode::Builtin::Lower),
        "trim" => Some(zirc_bytecode::Builtin::Trim),
        "split" => Some(zirc_bytecode::Builtin::Split),
        "join" => Some(zirc_bytecode::Builtin::Join),
        // Type conversion
        "int" => Some(zirc_bytecode::Builtin::Int),
        "str" => Some(zirc_bytecode::Builtin::Str),
        // Utility functions
        "type" => Some(zirc_bytecode::Builtin::Type),
        _ => None,
    }
}

