//! Zirc compiler: AST -> bytecode.

use std::collections::HashMap;

use zirc_bytecode::{Builtin, Function as BcFunction, Instruction as BC, Program as BcProgram};
use zirc_syntax::ast::*;
use zirc_syntax::error::{Result, error};

pub struct Compiler {
    func_indices: HashMap<String, usize>,
    functions: Vec<BcFunction>,
}

impl Default for Compiler { fn default() -> Self { Self::new() } }

impl Compiler {
    pub fn new() -> Self {
        Self { func_indices: HashMap::new(), functions: Vec::new() }
    }

    pub fn compile(&mut self, program: Program) -> Result<BcProgram> {
        // First pass: collect function names to assign indices
        for item in &program.items {
            if let Item::Function(f) = item {
                if self.func_indices.contains_key(&f.name) {
                    return error(format!("Duplicate function '{}'", f.name));
                }
                let idx = self.functions.len();
                self.func_indices.insert(f.name.clone(), idx);
                // placeholder; fill later
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
        let mut main_builder = FuncBuilder::new("__main".to_string(), 0);
        for item in program.items.into_iter() {
            if let Item::Stmt(s) = item { main_builder.emit_stmt(self, &s)?; }
        }
        main_builder.emit(BC::Halt);
        let main = main_builder.finish();

        Ok(BcProgram { functions: self.functions.clone(), main })
    }

    fn compile_function(&mut self, f: &Function) -> Result<BcFunction> {
        let mut b = FuncBuilder::new(f.name.clone(), f.params.len());
        // declare params in order
        for p in &f.params { b.declare_param(p.name.clone())?; }
        for s in &f.body { b.emit_stmt(self, s)?; }
        // implicit return unit if not returned
        b.emit(BC::PushUnit);
        b.emit(BC::Return);
        Ok(b.finish())
    }
}

struct FuncBuilder {
    name: String,
    arity: usize,
    code: Vec<BC>,
    locals: Locals,
    // loop stack
    loop_stack: Vec<LoopCtx>,
}

impl FuncBuilder {
    fn new(name: String, arity: usize) -> Self {
        // Locals start at 0; params will occupy slots [0..arity)
        Self { name, arity, code: Vec::new(), locals: Locals::new(0), loop_stack: Vec::new() }
    }

    fn finish(self) -> BcFunction {
        BcFunction { name: self.name, arity: self.arity, local_count: self.locals.max_alloc as usize, code: self.code }
    }

    fn emit(&mut self, i: BC) -> usize { self.code.push(i); self.code.len() - 1 }
    fn here(&self) -> usize { self.code.len() }

    fn patch_to_here(&mut self, at: usize) -> Result<()> {
        let tgt = self.here();
        match &mut self.code[at] {
            BC::Jump(ref mut x) | BC::JumpIfFalse(ref mut x) | BC::JumpIfTrue(ref mut x) => { *x = tgt; Ok(()) }
            other => error(format!("cannot patch at {:?}", other)),
        }
    }

    fn declare_param(&mut self, name: String) -> Result<()> {
        self.locals.declare(name)?; Ok(())
    }

    fn declare_var(&mut self, name: String) -> Result<u16> { self.locals.declare(name) }

    fn resolve_var(&self, name: &str) -> Result<u16> { self.locals.resolve(name).ok_or_else(|| zirc_syntax::error::Error::new(format!("Undefined variable '{}'", name))) }

    fn emit_stmt(&mut self, c: &Compiler, s: &Stmt) -> Result<()> {
        match s {
            Stmt::Let { name, expr, .. } => {
                let slot = self.declare_var(name.clone())?;
                self.emit_expr(c, expr)?;
                self.emit(BC::StoreLocal(slot));
                Ok(())
            }
            Stmt::Assign { name, expr } => {
                let slot = self.resolve_var(name)?;
                self.emit_expr(c, expr)?;
                self.emit(BC::StoreLocal(slot));
                Ok(())
            }
            Stmt::Return(opt) => {
                if let Some(e) = opt { self.emit_expr(c, e)?; } else { self.emit(BC::PushUnit); }
                self.emit(BC::Return);
                Ok(())
            }
            Stmt::If { cond, then_body, else_body } => {
                self.emit_expr(c, cond)?;
                let jf_at = self.emit(BC::JumpIfFalse(0));
                for s in then_body { self.emit_stmt(c, s)?; }
                let jend_at = self.emit(BC::Jump(0));
                self.patch_to_here(jf_at)?; // else starts here
                for s in else_body { self.emit_stmt(c, s)?; }
                self.patch_to_here(jend_at)?;
                Ok(())
            }
            Stmt::While { cond, body } => {
                let loop_start = self.here();
                self.emit_expr(c, cond)?;
                let jf_at = self.emit(BC::JumpIfFalse(0));
                self.loop_stack.push(LoopCtx::new(loop_start));
                for s in body { self.emit_stmt(c, s)?; }
                // continue target is loop_start
                let ctx = self.loop_stack.pop().unwrap();
                // patch continues -> loop_start
                for at in ctx.continues { self.code[at] = BC::Jump(loop_start); }
                // jump back to start
                self.emit(BC::Jump(loop_start));
                // end label
                self.patch_to_here(jf_at)?;
                // patch breaks -> end
                let end = self.here();
                for at in ctx.breaks { self.code[at] = BC::Jump(end); }
                Ok(())
            }
            Stmt::For { var, start, end, body } => {
                // declare loop var and end bound temporary
                let i_slot = self.declare_var(var.clone())?;
                self.emit_expr(c, start)?; self.emit(BC::StoreLocal(i_slot));
                let end_slot = self.locals.alloc_temp();
                self.emit_expr(c, end)?; self.emit(BC::StoreLocal(end_slot));
                let loop_start = self.here();
                self.emit(BC::LoadLocal(i_slot));
                self.emit(BC::LoadLocal(end_slot));
                self.emit(BC::Lt);
                let jf_at = self.emit(BC::JumpIfFalse(0));
                self.loop_stack.push(LoopCtx::new(loop_start));
                for s in body { self.emit_stmt(c, s)?; }
                // continue target: increment
                let incr_ip = self.here();
                {
                    let ctx = self.loop_stack.last_mut().unwrap();
                    ctx.continue_target = Some(incr_ip);
                }
                self.emit(BC::LoadLocal(i_slot));
                self.emit(BC::PushInt(1));
                self.emit(BC::Add);
                self.emit(BC::StoreLocal(i_slot));
                self.emit(BC::Jump(loop_start));
                // end label
                self.patch_to_here(jf_at)?;
                let ctx = self.loop_stack.pop().unwrap();
                let end_ip = self.here();
                for at in ctx.breaks { self.code[at] = BC::Jump(end_ip); }
                let cont_ip = ctx.continue_target.unwrap_or(loop_start);
                for at in ctx.continues { self.code[at] = BC::Jump(cont_ip); }
                Ok(())
            }
            Stmt::Break => {
                let at = self.emit(BC::Jump(0));
                if let Some(ctx) = self.loop_stack.last_mut() {
                    ctx.breaks.push(at);
                    Ok(())
                } else { error("'break' outside of loop") }
            }
            Stmt::Continue => {
                let at = self.emit(BC::Jump(0));
                if let Some(ctx) = self.loop_stack.last_mut() {
                    ctx.continues.push(at);
                    Ok(())
                } else { error("'continue' outside of loop") }
            }
            Stmt::ExprStmt(e) => {
                self.emit_expr(c, e)?;
                self.emit(BC::Pop);
                Ok(())
            }
        }
    }

    fn emit_expr(&mut self, c: &Compiler, e: &Expr) -> Result<()> {
        match e {
            Expr::LiteralInt(n) => { self.emit(BC::PushInt(*n)); Ok(()) }
            Expr::LiteralString(s) => { self.emit(BC::PushStr(s.clone())); Ok(()) }
            Expr::LiteralBool(b) => { self.emit(BC::PushBool(*b)); Ok(()) }
            Expr::Ident(name) => {
                let slot = self.resolve_var(name)?;
                self.emit(BC::LoadLocal(slot));
                Ok(())
            }
            Expr::BinaryAdd(a,b) => { self.emit_expr(c,a)?; self.emit_expr(c,b)?; self.emit(BC::Add); Ok(()) }
            Expr::BinarySub(a,b) => { self.emit_expr(c,a)?; self.emit_expr(c,b)?; self.emit(BC::Sub); Ok(()) }
            Expr::BinaryMul(a,b) => { self.emit_expr(c,a)?; self.emit_expr(c,b)?; self.emit(BC::Mul); Ok(()) }
            Expr::BinaryDiv(a,b) => { self.emit_expr(c,a)?; self.emit_expr(c,b)?; self.emit(BC::Div); Ok(()) }
            Expr::Eq(a,b) => { self.emit_expr(c,a)?; self.emit_expr(c,b)?; self.emit(BC::Eq); Ok(()) }
            Expr::Ne(a,b) => { self.emit_expr(c,a)?; self.emit_expr(c,b)?; self.emit(BC::Ne); Ok(()) }
            Expr::Lt(a,b) => { self.emit_expr(c,a)?; self.emit_expr(c,b)?; self.emit(BC::Lt); Ok(()) }
            Expr::Le(a,b) => { self.emit_expr(c,a)?; self.emit_expr(c,b)?; self.emit(BC::Le); Ok(()) }
            Expr::Gt(a,b) => { self.emit_expr(c,a)?; self.emit_expr(c,b)?; self.emit(BC::Gt); Ok(()) }
            Expr::Ge(a,b) => { self.emit_expr(c,a)?; self.emit_expr(c,b)?; self.emit(BC::Ge); Ok(()) }
            Expr::LogicalNot(a) => { self.emit_expr(c,a)?; self.emit(BC::Not); Ok(()) }
            Expr::LogicalAnd(a,b) => {
                // short-circuit: if a is false, skip b
                self.emit_expr(c, a)?;
                let jf = self.emit(BC::JumpIfFalse(0));
                // if true, evaluate b and leave its value on stack
                self.emit_expr(c, b)?;
                // else-branch: push false
                let jend = self.emit(BC::Jump(0));
                self.patch_to_here(jf)?;
                self.emit(BC::PushBool(false));
                self.patch_to_here(jend)?;
                Ok(())
            }
            Expr::LogicalOr(a,b) => {
                // short-circuit: if a is true, skip b
                self.emit_expr(c, a)?;
                let jt = self.emit(BC::JumpIfTrue(0));
                // if false, evaluate b
                self.emit_expr(c, b)?;
                let jend = self.emit(BC::Jump(0));
                self.patch_to_here(jt)?;
                self.emit(BC::PushBool(true));
                self.patch_to_here(jend)?;
                Ok(())
            }
            Expr::Call { name, args } => {
                // builtins
                if let Some(bi) = builtin_of(name) {
                    for a in args { self.emit_expr(c, a)?; }
                    self.emit(BC::BuiltinCall(bi, args.len()));
                    return Ok(());
                }
                let &fi = c.func_indices.get(name).ok_or_else(|| zirc_syntax::error::Error::new(format!("Undefined function '{}'", name)))?;
                for a in args { self.emit_expr(c, a)?; }
                self.emit(BC::Call(fi, args.len()));
                Ok(())
            }
            Expr::List(elems) => {
                for a in elems { self.emit_expr(c, a)?; }
                self.emit(BC::MakeList(elems.len()));
                Ok(())
            }
            Expr::Index(base, idx) => {
                self.emit_expr(c, base)?;
                self.emit_expr(c, idx)?;
                self.emit(BC::Index);
                Ok(())
            }
        }
    }
}

struct Locals {
    scopes: Vec<HashMap<String, u16>>, // name -> slot
    next: u16,
    max_alloc: u16,
}

impl Locals {
    fn new(start: u16) -> Self { Self { scopes: vec![HashMap::new()], next: start, max_alloc: start } }
    fn declare(&mut self, name: String) -> Result<u16> {
        if self.scopes.last().unwrap().contains_key(&name) { return error(format!("Variable '{}' already defined in scope", name)); }
        let idx = self.next; self.next = self.next.checked_add(1).ok_or_else(|| zirc_syntax::error::Error::new("too many locals"))?;
        self.scopes.last_mut().unwrap().insert(name, idx);
        if idx + 1 > self.max_alloc { self.max_alloc = idx + 1; }
        Ok(idx)
    }
    fn resolve(&self, name: &str) -> Option<u16> {
        for scope in self.scopes.iter().rev() {
            if let Some(&i) = scope.get(name) { return Some(i); }
        }
        None
    }
    fn alloc_temp(&mut self) -> u16 {
        let idx = self.next; self.next += 1; if idx + 1 > self.max_alloc { self.max_alloc = idx + 1; } idx
    }
    #[allow(dead_code)]
    fn push_scope(&mut self) { self.scopes.push(HashMap::new()); }
    #[allow(dead_code)]
    fn pop_scope(&mut self) { let _ = self.scopes.pop(); }
}

struct LoopCtx { breaks: Vec<usize>, continues: Vec<usize>, start: usize, continue_target: Option<usize> }
impl LoopCtx { fn new(start: usize) -> Self { Self { breaks: Vec::new(), continues: Vec::new(), start, continue_target: None } } }

fn builtin_of(name: &str) -> Option<Builtin> {
    match name {
        "show" => Some(Builtin::Show),
        "showf" => Some(Builtin::ShowF),
        "prompt" => Some(Builtin::Prompt),
        "rf" => Some(Builtin::Rf),
        "wf" => Some(Builtin::Wf),
        _ => None,
    }
}

