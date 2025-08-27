use std::collections::HashMap;

use zirc_syntax::ast::*;
use zirc_syntax::error::{error, Result};

#[derive(Debug, Clone, PartialEq)]
/// Zirc interpreter: evaluates AST nodes with a simple runtime.

/// Runtime value.
pub enum Value {
    Int(i64),
    Str(String),
    Bool(bool),
    List(Vec<Value>),
    Unit,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, it) in items.iter().enumerate() { if i>0 { write!(f, ", ")?; } write!(f, "{}", it)?; }
                write!(f, "]")
            }
            Value::Unit => write!(f, "<unit>"),
        }
    }
}

#[derive(Clone)]
/// A variable binding holding a value and optional static type.
pub struct Binding {
    pub value: Value,
    pub ty: Option<Type>,
}

#[derive(Clone)]
/// Lexically nested environment of variable bindings.
pub struct Env<'a> {
    vars: HashMap<String, Binding>,
    parent: Option<&'a Env<'a>>,
}

impl<'a> Env<'a> {
    pub fn new_root() -> Self { Self { vars: HashMap::new(), parent: None } }
    fn child(&'a self) -> Env<'a> { Env { vars: HashMap::new(), parent: Some(self) } }

    pub fn vars_snapshot(&self) -> Vec<(String, Value)> {
        self.vars.iter().map(|(k,b)| (k.clone(), b.value.clone())).collect()
    }

    fn get(&self, name: &str) -> Option<Binding> {
        if let Some(b) = self.vars.get(name) { Some(b.clone()) }
        else { self.parent.and_then(|p| p.get(name)) }
    }

    fn define(&mut self, name: String, val: Value, ty: Option<Type>) { self.vars.insert(name, Binding { value: val, ty }); }

    fn assign(&mut self, name: &str, val: Value) -> Result<()> {
        if let Some(b) = self.vars.get_mut(name) {
            if let Some(t) = &b.ty { Interpreter::check_type(&val, t)?; }
            b.value = val;
            Ok(())
        } else {
            zirc_syntax::error::error(format!("Assignment to undefined variable '{}'", name))
        }
    }
}

#[derive(Default, Debug, Clone)]
/// Simple memory statistics (currently strings only) for observability.
pub struct MemoryStats {
    pub strings_allocated: usize,
    pub bytes_allocated: usize,
}

/// The interpreter state and function table.
pub struct Interpreter {
    functions: HashMap<String, Function>,
    mem: MemoryStats,
}

impl Interpreter {
    pub fn new() -> Self { Self { functions: HashMap::new(), mem: MemoryStats::default() } }

    pub fn memory_stats(&self) -> MemoryStats { self.mem.clone() }
    pub fn reset(&mut self) { self.functions.clear(); self.mem = MemoryStats::default(); }

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

    pub fn run_with_env<'a>(&mut self, program: Program, env: &mut Env<'a>) -> Result<Option<Value>> {
        for item in &program.items {
            if let Item::Function(f) = item { self.functions.insert(f.name.clone(), f.clone()); }
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

    fn exec_block<'a>(&mut self, env: &mut Env<'a>, body: &[Stmt]) -> Result<Flow> {
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

    fn exec_stmt<'a>(&mut self, env: &mut Env<'a>, stmt: &Stmt) -> Result<Flow> {
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
                    let go = match c { Value::Bool(b) => b, other => return error(format!("while condition must be bool, got {:?}", other)) };
                    if !go { break; }
                    match self.exec_block(env, body)? {
                        Flow::Continue(v) => { let _ = v; }
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Break => break,
                        Flow::ContinueLoop => continue,
                    }
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

    fn eval_expr<'a>(&mut self, env: &mut Env<'a>, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::LiteralInt(n) => Ok(Value::Int(*n)),
            Expr::LiteralString(s) => { self.mem.strings_allocated += 1; self.mem.bytes_allocated += s.len(); Ok(Value::Str(s.clone())) },
            Expr::LiteralBool(b) => Ok(Value::Bool(*b)),
            Expr::Ident(name) => {
                match env.get(name) {
                    Some(b) => Ok(b.value),
                    None => zirc_syntax::error::error(format!("Undefined variable '{}'", name)),
                }
            },
            Expr::BinaryAdd(a, b) => {
                match (self.eval_expr(env, a)?, self.eval_expr(env, b)?) {
                    (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x + y)),
                    (Value::Str(x), Value::Str(y)) => { let r = format!("{}{}", x, y); self.mem.strings_allocated += 1; self.mem.bytes_allocated += r.len(); Ok(Value::Str(r)) },
                    (x, y) => error(format!("Cannot add {:?} and {:?}", x, y)),
                }
            }
            Expr::BinarySub(a, b) => match (self.eval_expr(env, a)?, self.eval_expr(env, b)?) { (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x - y)), (x, y) => error(format!("Cannot subtract {:?} and {:?}", x, y)) },
            Expr::BinaryMul(a, b) => match (self.eval_expr(env, a)?, self.eval_expr(env, b)?) { (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x * y)), (x, y) => error(format!("Cannot multiply {:?} and {:?}", x, y)) },
            Expr::BinaryDiv(a, b) => match (self.eval_expr(env, a)?, self.eval_expr(env, b)?) { (Value::Int(x), Value::Int(y)) => Ok(Value::Int(x / y)), (x, y) => error(format!("Cannot divide {:?} and {:?}", x, y)) },
            Expr::Eq(a, b) => Ok(Value::Bool(self.eval_expr(env, a)? == self.eval_expr(env, b)?)),
            Expr::Ne(a, b) => Ok(Value::Bool(self.eval_expr(env, a)? != self.eval_expr(env, b)?)),
            Expr::LogicalAnd(a, b) => {
                match self.eval_expr(env, a)? {
                    Value::Bool(false) => Ok(Value::Bool(false)),
                    Value::Bool(true) => match self.eval_expr(env, b)? { Value::Bool(bb) => Ok(Value::Bool(bb)), other => error(format!("&& expects bool, got {:?}", other)) },
                    other => error(format!("&& expects bool, got {:?}", other)),
                }
            }
            Expr::LogicalOr(a, b) => {
                match self.eval_expr(env, a)? {
                    Value::Bool(true) => Ok(Value::Bool(true)),
                    Value::Bool(false) => match self.eval_expr(env, b)? { Value::Bool(bb) => Ok(Value::Bool(bb)), other => error(format!("|| expects bool, got {:?}", other)) },
                    other => error(format!("|| expects bool, got {:?}", other)),
                }
            }
            Expr::LogicalNot(e) => match self.eval_expr(env, e)? { Value::Bool(b) => Ok(Value::Bool(!b)), other => error(format!("! expects bool, got {:?}", other)) },
            Expr::Lt(a, b) => match (self.eval_expr(env, a)?, self.eval_expr(env, b)?) { (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x < y)), _ => error("< expects ints") },
            Expr::Le(a, b) => match (self.eval_expr(env, a)?, self.eval_expr(env, b)?) { (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x <= y)), _ => error("<= expects ints") },
            Expr::Gt(a, b) => match (self.eval_expr(env, a)?, self.eval_expr(env, b)?) { (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x > y)), _ => error("> expects ints") },
            Expr::Ge(a, b) => match (self.eval_expr(env, a)?, self.eval_expr(env, b)?) { (Value::Int(x), Value::Int(y)) => Ok(Value::Bool(x >= y)), _ => error(">= expects ints") },
            Expr::List(elems) => {
                let mut v = Vec::with_capacity(elems.len());
                for e in elems { v.push(self.eval_expr(env, e)?); }
                Ok(Value::List(v))
            }
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
                        self.mem.strings_allocated += 1; self.mem.bytes_allocated += ss.len();
                        Ok(Value::Str(ss))
                    }
                    other => error(format!("indexing not supported for {:?}", other))
                }
            }
            Expr::Call { name, args } => {
                if name == "showf" {
                    return self.call_showf(env, args);
                }
                let func = self.functions.get(name).cloned().ok_or_else(|| format!("Undefined function '{}'", name))?;
                if func.params.len() != args.len() {
                    return error(format!("Function '{}' expected {} args, got {}", name, func.params.len(), args.len()));
                }
                let mut evaluated_args = Vec::with_capacity(args.len());
                for a in args.iter() {
                    evaluated_args.push(self.eval_expr(env, a)?);
                }
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
                if let Some(expected) = func.return_type.clone() {
                    Interpreter::check_type(&ret_val, &expected)?;
                }
                Ok(ret_val)
            }
        }
    }

    fn check_type(val: &Value, ty: &Type) -> Result<()> {
        let ok = match (val, ty) {
            (Value::Int(_), Type::Int) => true,
            (Value::Str(_), Type::String) => true,
            (Value::Bool(_), Type::Bool) => true,
            (Value::Unit, Type::Unit) => true,
            _ => false,
        };
        if ok { Ok(()) } else { error(format!("Type mismatch: value {:?} does not match type {:?}", val, ty)) }
    }

    fn call_showf<'a>(&mut self, env: &mut Env<'a>, args: &[Expr]) -> Result<Value> {
        if args.is_empty() { return error("showf requires at least a format string"); }
        let fmt = match self.eval_expr(env, &args[0])? {
            Value::Str(s) => s,
            _ => return error("showf first argument must be a string"),
        };
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
                            other => return error(format!("%s expects string/bool/list, got {:?}", other))
                        }
                        arg_i += 1;
                    }
                    Some('%') => out.push('%'),
                    Some(other) => return error(format!("Unsupported format specifier %{}", other)),
                    None => return error("Dangling % at end of format string"),
                }
            } else {
                out.push(c);
            }
        }
        println!("{}", out);
        Ok(Value::Unit)
    }
}

#[derive(Debug)]
enum Flow {
    Continue(Value),
    Return(Value),
    Break,
    ContinueLoop,
}
