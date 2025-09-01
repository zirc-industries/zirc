//! Main interpreter engine and builtins.

use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};

use crate::env::Env;
use crate::flow::Flow;
use crate::value::Value;
use zirc_syntax::ast::*;
use zirc_syntax::error::{Result, error};

#[derive(Default, Debug, Clone)]
pub struct MemoryStats {
    /// Number of string values allocated during execution
    pub strings_allocated: usize,
    /// Total bytes allocated for string storage
    pub bytes_allocated: usize,
}

pub struct Interpreter {
    /// Global function definitions available to all scopes
    functions: HashMap<String, Function>,
    /// Memory usage tracking for observability
    mem: MemoryStats,
}

impl Default for Interpreter {
    fn default() -> Self { Self::new() }
}

impl Interpreter {
    pub fn new() -> Self {
        Self { functions: HashMap::new(), mem: MemoryStats::default() }
    }

    pub fn memory_stats(&self) -> MemoryStats { self.mem.clone() }

    pub fn reset(&mut self) {
        self.functions.clear();
        self.mem = MemoryStats::default();
    }

    pub fn function_names(&self) -> Vec<String> {
        let mut v: Vec<String> = self.functions.keys().cloned().collect();
        v.sort();
        v
    }

    pub fn run(&mut self, program: Program) -> Result<()> {
        let mut env = Env::new_root();
        let _ = self.run_with_env(program, &mut env)?;
        Ok(())
    }

    pub fn run_with_env(&mut self, program: Program, env: &mut Env<'_>) -> Result<Option<Value>> {
        for item in &program.items {
            if let Item::Function(f) = item {
                self.functions.insert(f.name.clone(), f.clone());
            }
        }
        let mut last: Option<Value> = None;
        for item in program.items {
            if let Item::Stmt(s) = item {
                match self.exec_stmt(env, &s)? {
                    Flow::Continue(v) => last = Some(v),
                    Flow::Return(_) => return error("'return' outside of function"),
                    Flow::Break => return error("'break' outside of loop"),
                    Flow::ContinueLoop => return error("'continue' outside of loop"),
                }
            }
        }
        Ok(last)
    }

    fn exec_block(&mut self, env: &mut Env<'_>, body: &[Stmt]) -> Result<Flow> {
        let mut last = Value::Unit;
        for s in body {
            match self.exec_stmt(env, s)? {
                Flow::Continue(v) => { last = v; }
                Flow::Return(v) => return Ok(Flow::Return(v)),
                Flow::Break => return Ok(Flow::Break),
                Flow::ContinueLoop => return Ok(Flow::ContinueLoop),
            }
        }
        Ok(Flow::Continue(last))
    }

    fn exec_stmt(&mut self, env: &mut Env<'_>, stmt: &Stmt) -> Result<Flow> {
        match stmt {
            Stmt::Let { name, ty, expr } => {
                let v = self.eval_expr(env, expr)?;
                if let Some(t) = ty { Interpreter::check_type(&v, t)?; }
                env.define(name.clone(), v, ty.clone());
                Ok(Flow::Continue(Value::Unit))
            }
            Stmt::Assign { name, expr } => {
                let v = self.eval_expr(env, expr)?;
                env.assign(name, v)?;
                Ok(Flow::Continue(Value::Unit))
            }
            Stmt::Return(opt) => {
                let v = match opt { Some(e) => self.eval_expr(env, e)?, None => Value::Unit };
                Ok(Flow::Return(v))
            }
            Stmt::If { cond, then_body, else_body } => {
                let c = self.eval_expr(env, cond)?;
                match c {
                    Value::Bool(true) => self.exec_block(env, then_body),
                    Value::Bool(false) => self.exec_block(env, else_body),
                    other => error(format!("if condition must be bool, got {:?}", other)),
                }
            }
            Stmt::While { cond, body } => {
                loop {
                    let c = self.eval_expr(env, cond)?;
                    let go = match c { Value::Bool(b) => b, other => { return error(format!("while condition must be bool, got {:?}", other)); } };
                    if !go { break; }
                    match self.exec_block(env, body)? {
                        Flow::Continue(_) => {}
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Break => break,
                        Flow::ContinueLoop => continue,
                    }
                }
                Ok(Flow::Continue(Value::Unit))
            }
            Stmt::For { var, start, end, body } => {
                let s = self.eval_expr(env, start)?;
                let e = self.eval_expr(env, end)?;
                let (mut i, e) = match (s, e) {
                    (Value::Int(a), Value::Int(b)) => (a, b),
                    (a, b) => { return error(format!("for bounds must be ints, got {:?} and {:?}", a, b)); }
                };
                while i < e {
                    if env.get(var).is_some() {
                        env.assign(var, Value::Int(i))?;
                    } else {
                        env.define(var.clone(), Value::Int(i), Some(Type::Int));
                    }
                    match self.exec_block(env, body)? {
                        Flow::Continue(_) => {}
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Break => break,
                        Flow::ContinueLoop => { i += 1; continue; }
                    }
                    i += 1;
                }
                Ok(Flow::Continue(Value::Unit))
            }
            Stmt::Break => Ok(Flow::Break),
            Stmt::Continue => Ok(Flow::ContinueLoop),
            Stmt::ExprStmt(e) => {
                let v = self.eval_expr(env, e)?;
                Ok(Flow::Continue(v))
            }
        }
    }

    fn eval_expr(&mut self, env: &mut Env<'_>, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::LiteralInt(n) => Ok(Value::Int(*n)),
            Expr::LiteralString(s) => { self.mem.strings_allocated += 1; self.mem.bytes_allocated += s.len(); Ok(Value::Str(s.clone())) }
            Expr::LiteralBool(b) => Ok(Value::Bool(*b)),
            Expr::Ident(name) => match env.get(name) { Some(b) => Ok(b.value), None => zirc_syntax::error::error(format!("Undefined variable '{}'", name)) },
            Expr::BinaryAdd(a, b) => match (self.eval_expr(env, a)?, self.eval_expr(env, b)?) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
                (Value::Str(x), Value::Str(y)) => { let r = format!("{}{}", x, y); self.mem.strings_allocated += 1; self.mem.bytes_allocated += r.len(); Ok(Value::Str(r)) }
                (x, y) => error(format!("Cannot add {:?} and {:?}", x, y)),
            },
            Expr::BinarySub(a, b) => match (self.eval_expr(env, a)?, self.eval_expr(env, b)?) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)),
                (x, y) => error(format!("Cannot subtract {:?} and {:?}", x, y)),
            },
            Expr::BinaryMul(a, b) => match (self.eval_expr(env, a)?, self.eval_expr(env, b)?) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)),
                (x, y) => error(format!("Cannot multiply {:?} and {:?}", x, y)),
            },
            Expr::BinaryDiv(a, b) => match (self.eval_expr(env, a)?, self.eval_expr(env, b)?) {
                (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x / y)),
                (x, y) => error(format!("Cannot divide {:?} and {:?}", x, y)),
            },
            Expr::Eq(a, b) => Ok(Value::Bool(self.eval_expr(env, a)? == self.eval_expr(env, b)?)),
            Expr::Ne(a, b) => Ok(Value::Bool(self.eval_expr(env, a)? != self.eval_expr(env, b)?)),
            Expr::LogicalAnd(a, b) => match self.eval_expr(env, a)? {
                Value::Bool(false) => Ok(Value::Bool(false)),
                Value::Bool(true) => match self.eval_expr(env, b)? { Value::Bool(bb) => Ok(Value::Bool(bb)), other => error(format!("&& expects bool, got {:?}", other)) },
                other => error(format!("&& expects bool, got {:?}", other)),
            },
            Expr::LogicalOr(a, b) => match self.eval_expr(env, a)? {
                Value::Bool(true) => Ok(Value::Bool(true)),
                Value::Bool(false) => match self.eval_expr(env, b)? { Value::Bool(bb) => Ok(Value::Bool(bb)), other => error(format!("|| expects bool, got {:?}", other)) },
                other => error(format!("|| expects bool, got {:?}", other)),
            },
            Expr::LogicalNot(e) => match self.eval_expr(env, e)? { Value::Bool(b) => Ok(Value::Bool(!b)), other => error(format!("! expects bool, got {:?}", other)) },
            Expr::Lt(a, b) => match (self.eval_expr(env, a)?, self.eval_expr(env, b)?) { (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x < y)), _ => error("< expects ints") },
            Expr::Le(a, b) => match (self.eval_expr(env, a)?, self.eval_expr(env, b)?) { (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x <= y)), _ => error("<= expects ints") },
            Expr::Gt(a, b) => match (self.eval_expr(env, a)?, self.eval_expr(env, b)?) { (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x > y)), _ => error("> expects ints") },
            Expr::Ge(a, b) => match (self.eval_expr(env, a)?, self.eval_expr(env, b)?) { (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x >= y)), _ => error(">= expects ints") },
            Expr::List(elems) => { let mut v = Vec::with_capacity(elems.len()); for e in elems { v.push(self.eval_expr(env, e)?); } Ok(Value::List(v)) }
            Expr::Index(base, idx) => {
                let b = self.eval_expr(env, base)?;
                let i = self.eval_expr(env, idx)?;
                let ix = match i { Value::Int(n) => n, other => return error(format!("index expects int, got {:?}", other)) };
                match b {
                    Value::List(items) => {
                        if ix < 0 || (ix as usize) >= items.len() { return error("index out of bounds"); }
                        Ok(items[ix as usize].clone())
                    }
                    Value::Str(s) => {
                        let chars: Vec<char> = s.chars().collect();
                        if ix < 0 || (ix as usize) >= chars.len() { return error("index out of bounds"); }
                        let ch = chars[ix as usize];
                        let ss = ch.to_string();
                        self.mem.strings_allocated += 1;
                        self.mem.bytes_allocated += ss.len();
                        Ok(Value::Str(ss))
                    }
                    other => error(format!("indexing not supported for {:?}", other)),
                }
            }
            Expr::Call { name, args } => {
                // builtins
                match name.as_str() {
                    "showf" => return self.call_showf(env, args),
                    "show" => return self.call_show(env, args),
                    "prompt" => return self.call_prompt(env, args),
                    "rf" => return self.call_rf(env, args),
                    "wf" => return self.call_wf(env, args),
                    _ => {}
                }
                let func = self
                    .functions
                    .get(name)
                    .cloned()
                    .ok_or_else(|| format!("Undefined function '{}'", name))?;
                if func.params.len() != args.len() {
                    return error(format!("Function '{}' expected {} args, got {}", name, func.params.len(), args.len()));
                }
                let mut evaluated_args = Vec::with_capacity(args.len());
                for a in args.iter() { evaluated_args.push(self.eval_expr(env, a)?); }
                let mut child = env.child();
                for (p, v) in func.params.iter().zip(evaluated_args.into_iter()) {
                    if let Some(t) = &p.ty { Interpreter::check_type(&v, t)?; }
                    child.define(p.name.clone(), v, p.ty.clone());
                }
                let mut inner = child;
                let flow = self.exec_block(&mut inner, &func.body)?;
                let ret_val = match flow {
                    Flow::Continue(v) => v, // implicit last value
                    Flow::Return(v) => v,
                    Flow::Break => return error("'break' outside of loop"),
                    Flow::ContinueLoop => return error("'continue' outside of loop"),
                };
                if let Some(expected) = func.return_type.clone() { Interpreter::check_type(&ret_val, &expected)?; }
                Ok(ret_val)
            }
        }
    }

    pub(crate) fn check_type(val: &Value, ty: &Type) -> Result<()> {
        let ok = matches!((val, ty),
            (Value::Int(_), Type::Int)
            | (Value::Str(_), Type::String)
            | (Value::Bool(_), Type::Bool)
            | (Value::Unit, Type::Unit)
        );
        if ok { Ok(()) } else { error(format!("Type mismatch: value {:?} does not match type {:?}", val, ty)) }
    }

    fn call_showf(&mut self, env: &mut Env<'_>, args: &[Expr]) -> Result<Value> {
        if args.is_empty() { return error("showf requires at least a format string"); }
        let fmt = match self.eval_expr(env, &args[0])? { Value::Str(s) => s, _ => return error("showf first argument must be a string") };
        let mut out = String::new();
        let mut arg_i = 1usize;
        let mut chars = fmt.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '%' {
                match chars.next() {
                    Some('d') => {
                        if arg_i >= args.len() { return error("showf missing %d argument"); }
                        match self.eval_expr(env, &args[arg_i])? { Value::Int(n) => out.push_str(&n.to_string()), other => return error(format!("%d expects int, got {:?}", other)) }
                        arg_i += 1;
                    }
                    Some('s') => {
                        if arg_i >= args.len() { return error("showf missing %s argument"); }
                        match self.eval_expr(env, &args[arg_i])? {
                            Value::Str(s) => out.push_str(&s),
                            Value::Bool(b) => out.push_str(if b { "true" } else { "false" }),
                            Value::List(items) => out.push_str(&format!("{}", Value::List(items))),
                            other => { return error(format!("%s expects string/bool/list, got {:?}", other)); }
                        }
                        arg_i += 1;
                    }
                    Some('%') => out.push('%'),
                    Some(other) => { return error(format!("Unsupported format specifier %{}", other)); }
                    None => return error("Dangling % at end of format string"),
                }
            } else {
                out.push(c);
            }
        }
        if std::env::var("ZIRC_BENCH_SILENT").is_err() { println!("{}", out); }
        Ok(Value::Unit)
    }

    /// Simple show function - prints a single value
    fn call_show(&mut self, env: &mut Env<'_>, args: &[Expr]) -> Result<Value> {
        if args.len() != 1 { return error("show() expects exactly 1 argument"); }
        let val = self.eval_expr(env, &args[0])?;
        if std::env::var("ZIRC_BENCH_SILENT").is_err() { println!("{}", val); }
        Ok(Value::Unit)
    }

    /// Prompt function - reads a line from stdin and returns as string
    fn call_prompt(&mut self, env: &mut Env<'_>, args: &[Expr]) -> Result<Value> {
        if args.len() > 1 { return error("prompt() expects 0 or 1 arguments"); }
        let silent = std::env::var("ZIRC_BENCH_SILENT").is_ok();
        // Optional prompt string
        if args.len() == 1 {
            let prompt = self.eval_expr(env, &args[0])?;
            match prompt {
                Value::Str(s) => {
                    if !silent { print!("{}", s); io::stdout().flush().map_err(|e| format!("IO error: {}", e))?; }
                }
                other => return error(format!("prompt() prompt must be string, got {:?}", other)),
            }
        }
        let input = if silent {
            std::env::var("ZIRC_BENCH_PROMPT_REPLY").unwrap_or_default()
        } else {
            let mut input = String::new();
            io::stdin().read_line(&mut input).map_err(|e| format!("IO error: {}", e))?;
            // Remove trailing newline
            if input.ends_with('\n') { input.pop(); if input.ends_with('\r') { input.pop(); } }
            input
        };
        self.mem.strings_allocated += 1;
        self.mem.bytes_allocated += input.len();
        Ok(Value::Str(input))
    }

    /// Read file function - reads entire file content as string
    fn call_rf(&mut self, env: &mut Env<'_>, args: &[Expr]) -> Result<Value> {
        if args.len() != 1 { return error("rf() expects exactly 1 argument"); }
        let path = match self.eval_expr(env, &args[0])? { Value::Str(s) => s, other => return error(format!("rf() path must be string, got {:?}", other)) };
        let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read file '{}': {}", path, e))?;
        self.mem.strings_allocated += 1;
        self.mem.bytes_allocated += content.len();
        Ok(Value::Str(content))
    }

    /// Write file function - writes string content to file
    fn call_wf(&mut self, env: &mut Env<'_>, args: &[Expr]) -> Result<Value> {
        if args.len() != 2 { return error("wf() expects exactly 2 arguments: path and content"); }
        let path = match self.eval_expr(env, &args[0])? { Value::Str(s) => s, other => return error(format!("wf() path must be string, got {:?}", other)) };
        let content = match self.eval_expr(env, &args[1])? { Value::Str(s) => s, other => return error(format!("wf() content must be string, got {:?}", other)) };
        fs::write(&path, &content).map_err(|e| format!("Failed to write file '{}': {}", path, e))?;
        Ok(Value::Unit)
    }
}

