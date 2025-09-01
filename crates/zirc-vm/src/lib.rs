//! Zirc VM: executes Zirc bytecode programs.

use std::io::{self, Write};
use std::fs;

use zirc_bytecode::{Builtin, Instruction, Program, Value};
use zirc_syntax::error::{Result, error};

#[derive(Clone)]
struct Frame {
    func_ref: CodeRef,
    ip: usize,
    locals: Vec<Value>,
}

#[derive(Clone, Copy)]
enum CodeRef {
    Main,
    Func(usize),
}

pub struct Vm {
    stack: Vec<Value>,
}

impl Default for Vm { fn default() -> Self { Self::new() } }

impl Vm {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn run(&mut self, program: &Program) -> Result<()> {
        let mut frames: Vec<Frame> = Vec::new();
        frames.push(Frame {
            func_ref: CodeRef::Main,
            ip: 0,
            locals: vec![Value::Unit; program.main.local_count],
        });

        while let Some(frame) = frames.last_mut() {
            let func = match frame.func_ref {
                CodeRef::Main => &program.main,
                CodeRef::Func(i) => &program.functions[i],
            };
            if frame.ip >= func.code.len() {
                // Implicit return Unit if we run off the end
                if frames.len() == 1 { break; } // main returns ends program
                let ret = Value::Unit;
                frames.pop();
                self.stack.push(ret);
                continue;
            }
            let instr = func.code[frame.ip].clone();
            // default ip increment; jumps will override
            frame.ip += 1;
            match instr {
                Instruction::PushInt(n) => self.stack.push(Value::Int(n)),
                Instruction::PushStr(s) => self.stack.push(Value::Str(s)),
                Instruction::PushBool(b) => self.stack.push(Value::Bool(b)),
                Instruction::PushUnit => self.stack.push(Value::Unit),
                Instruction::MakeList(n) => {
                    if self.stack.len() < n { return error("stack underflow in MakeList"); }
                    let start = self.stack.len() - n;
                    let elems = self.stack.drain(start..).collect::<Vec<_>>();
                    // elems are in original order already because we drained a slice
                    self.stack.push(Value::List(elems));
                }
                Instruction::Index => {
                    let idx = self.stack.pop().ok_or_else(|| "stack underflow in Index")?;
                    let base = self.stack.pop().ok_or_else(|| "stack underflow in Index")?;
                    let ix = match idx { Value::Int(n) => n, other => return error(format!("index expects int, got {:?}", other)) };
                    match base {
                        Value::List(items) => {
                            if ix < 0 || (ix as usize) >= items.len() { return error("index out of bounds"); }
                            self.stack.push(items[ix as usize].clone());
                        }
                        Value::Str(s) => {
                            let chars: Vec<char> = s.chars().collect();
                            if ix < 0 || (ix as usize) >= chars.len() { return error("index out of bounds"); }
                            self.stack.push(Value::Str(chars[ix as usize].to_string()));
                        }
                        other => return error(format!("indexing not supported for {:?}", other)),
                    }
                }
                Instruction::LoadLocal(i) => {
                    let i = i as usize;
                    let v = frame.locals.get(i).ok_or_else(|| "invalid local index")?.clone();
                    self.stack.push(v);
                }
                Instruction::StoreLocal(i) => {
                    let i = i as usize;
                    let v = self.stack.pop().ok_or_else(|| "stack underflow in StoreLocal")?;
                    let slot = frame.locals.get_mut(i).ok_or_else(|| "invalid local index")?;
                    *slot = v;
                }
                Instruction::Pop => { let _ = self.stack.pop(); }
                Instruction::Add => {
                    let b = self.stack.pop().ok_or_else(|| "stack underflow in Add")?;
                    let a = self.stack.pop().ok_or_else(|| "stack underflow in Add")?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x + y)),
                        (Value::Str(x), Value::Str(y)) => self.stack.push(Value::Str(format!("{}{}", x, y))),
                        (x, y) => return error(format!("Cannot add {:?} and {:?}", x, y)),
                    }
                }
                Instruction::Sub => {
                    let b = self.stack.pop().ok_or_else(|| "stack underflow in Sub")?;
                    let a = self.stack.pop().ok_or_else(|| "stack underflow in Sub")?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x - y)),
                        (x, y) => return error(format!("Cannot subtract {:?} and {:?}", x, y)),
                    }
                }
                Instruction::Mul => {
                    let b = self.stack.pop().ok_or_else(|| "stack underflow in Mul")?;
                    let a = self.stack.pop().ok_or_else(|| "stack underflow in Mul")?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x * y)),
                        (x, y) => return error(format!("Cannot multiply {:?} and {:?}", x, y)),
                    }
                }
                Instruction::Div => {
                    let b = self.stack.pop().ok_or_else(|| "stack underflow in Div")?;
                    let a = self.stack.pop().ok_or_else(|| "stack underflow in Div")?;
                    match (a, b) {
                        (Value::Int(_), Value::Int(0)) => return error("division by zero"),
                        (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x / y)),
                        (x, y) => return error(format!("Cannot divide {:?} and {:?}", x, y)),
                    }
                }
                Instruction::Eq => {
                    let b = self.stack.pop().ok_or_else(|| "stack underflow in Eq")?;
                    let a = self.stack.pop().ok_or_else(|| "stack underflow in Eq")?;
                    self.stack.push(Value::Bool(a == b));
                }
                Instruction::Ne => {
                    let b = self.stack.pop().ok_or_else(|| "stack underflow in Ne")?;
                    let a = self.stack.pop().ok_or_else(|| "stack underflow in Ne")?;
                    self.stack.push(Value::Bool(a != b));
                }
                Instruction::Lt => {
                    let b = self.stack.pop().ok_or_else(|| "stack underflow in Lt")?;
                    let a = self.stack.pop().ok_or_else(|| "stack underflow in Lt")?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Bool(x < y)),
                        _ => return error("< expects ints"),
                    }
                }
                Instruction::Le => {
                    let b = self.stack.pop().ok_or_else(|| "stack underflow in Le")?;
                    let a = self.stack.pop().ok_or_else(|| "stack underflow in Le")?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Bool(x <= y)),
                        _ => return error("<= expects ints"),
                    }
                }
                Instruction::Gt => {
                    let b = self.stack.pop().ok_or_else(|| "stack underflow in Gt")?;
                    let a = self.stack.pop().ok_or_else(|| "stack underflow in Gt")?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Bool(x > y)),
                        _ => return error("> expects ints"),
                    }
                }
                Instruction::Ge => {
                    let b = self.stack.pop().ok_or_else(|| "stack underflow in Ge")?;
                    let a = self.stack.pop().ok_or_else(|| "stack underflow in Ge")?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Bool(x >= y)),
                        _ => return error(">= expects ints"),
                    }
                }
                Instruction::Not => {
                    let a = self.stack.pop().ok_or_else(|| "stack underflow in Not")?;
                    match a { Value::Bool(b) => self.stack.push(Value::Bool(!b)), other => return error(format!("! expects bool, got {:?}", other)) }
                }
                Instruction::Jump(tgt) => {
                    frame.ip = tgt;
                }
                Instruction::JumpIfFalse(tgt) => {
                    let c = self.stack.pop().ok_or_else(|| "stack underflow in JumpIfFalse")?;
                    match c { Value::Bool(false) => frame.ip = tgt, Value::Bool(true) => (), other => return error(format!("condition must be bool, got {:?}", other)) }
                }
                Instruction::JumpIfTrue(tgt) => {
                    let c = self.stack.pop().ok_or_else(|| "stack underflow in JumpIfTrue")?;
                    match c { Value::Bool(true) => frame.ip = tgt, Value::Bool(false) => (), other => return error(format!("condition must be bool, got {:?}", other)) }
                }
                Instruction::Call(fi, argc) => {
                    // collect args
                    if self.stack.len() < argc { return error("stack underflow in Call"); }
                    let start = self.stack.len() - argc;
                    let mut args = self.stack.drain(start..).collect::<Vec<_>>();
                    // args now in original order
                    let func = program.functions.get(fi).ok_or_else(|| "invalid function index")?;
                    if func.arity != argc { return error(format!("Function '{}' expected {} args, got {}", func.name, func.arity, argc)); }
                    // prepare locals
                    let mut locals = vec![Value::Unit; func.local_count];
                    for (i, v) in args.drain(..).enumerate() { locals[i] = v; }
                    // push frame
                    frames.push(Frame { func_ref: CodeRef::Func(fi), ip: 0, locals });
                }
                Instruction::Return => {
                    let ret = self.stack.pop().unwrap_or(Value::Unit);
                    frames.pop();
                    if frames.is_empty() {
                        // returning from main -> end
                        break;
                    }
                    self.stack.push(ret);
                }
                Instruction::BuiltinCall(which, argc) => {
                    // collect args
                    if self.stack.len() < argc { return error("stack underflow in BuiltinCall"); }
                    let start = self.stack.len() - argc;
                    let args = self.stack.drain(start..).collect::<Vec<_>>();
                    let silent = std::env::var("ZIRC_BENCH_SILENT").is_ok();
                    match which {
                        Builtin::Show => {
                            if args.len() != 1 { return error("show() expects exactly 1 argument"); }
                            if !silent { println!("{}", display_value(&args[0])); }
                            self.stack.push(Value::Unit);
                        }
                        Builtin::ShowF => {
                            if args.is_empty() { return error("showf requires at least a format string"); }
                            let fmt = match &args[0] { Value::Str(s) => s.clone(), _ => return error("showf first argument must be a string") };
                            let mut out = String::new();
                            let mut arg_i = 1usize;
                            let mut chars = fmt.chars().peekable();
                            while let Some(c) = chars.next() {
                                if c == '%' {
                                    match chars.next() {
                                        Some('d') => {
                                            if arg_i >= args.len() { return error("showf missing %d argument"); }
                                            match &args[arg_i] { Value::Int(n) => out.push_str(&n.to_string()), other => return error(format!("%d expects int, got {:?}", other)) }
                                            arg_i += 1;
                                        }
                                        Some('s') => {
                                            if arg_i >= args.len() { return error("showf missing %s argument"); }
                                            match &args[arg_i] {
                                                Value::Str(s) => out.push_str(s),
                                                Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
                                                Value::List(items) => out.push_str(&display_value(&Value::List(items.clone()))),
                                                other => return error(format!("%s expects string/bool/list, got {:?}", other)),
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
                            if !silent { println!("{}", out); }
                            self.stack.push(Value::Unit);
                        }
                        Builtin::Prompt => {
                            if args.len() > 1 { return error("prompt() expects 0 or 1 arguments"); }
                            let silent = std::env::var("ZIRC_BENCH_SILENT").is_ok();
                            if args.len() == 1 {
                                if let Value::Str(s) = &args[0] {
                                    if !silent { print!("{}", s); io::stdout().flush().map_err(|e| format!("IO error: {}", e))?; }
                                } else {
                                    return error("prompt() prompt must be string");
                                }
                            }
                            let input = if silent { std::env::var("ZIRC_BENCH_PROMPT_REPLY").unwrap_or_default() } else {
                                let mut input = String::new();
                                io::stdin().read_line(&mut input).map_err(|e| format!("IO error: {}", e))?;
                                if input.ends_with('\n') { input.pop(); if input.ends_with('\r') { input.pop(); } }
                                input
                            };
                            self.stack.push(Value::Str(input));
                        }
                        Builtin::Rf => {
                            if args.len() != 1 { return error("rf() expects exactly 1 argument"); }
                            let path = match &args[0] { Value::Str(s) => s.clone(), _ => return error("rf() path must be string") };
                            let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read file '{}': {}", path, e))?;
                            self.stack.push(Value::Str(content));
                        }
                        Builtin::Wf => {
                            if args.len() != 2 { return error("wf() expects exactly 2 arguments: path and content"); }
                            let path = match &args[0] { Value::Str(s) => s.clone(), _ => return error("wf() path must be string") };
                            let content = match &args[1] { Value::Str(s) => s.clone(), _ => return error("wf() content must be string") };
                            fs::write(&path, &content).map_err(|e| format!("Failed to write file '{}': {}", path, e))?;
                            self.stack.push(Value::Unit);
                        }
                    }
                }
                Instruction::Halt => { break; }
            }
        }
        Ok(())
    }
}

fn display_value(v: &Value) -> String {
    match v {
        Value::Int(n) => n.to_string(),
        Value::Str(s) => s.clone(),
        Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
        Value::List(items) => {
            let mut s = String::from("[");
            for (i, it) in items.iter().enumerate() {
                if i > 0 { s.push_str(", "); }
                s.push_str(&display_value(it));
            }
            s.push(']');
            s
        }
        Value::Unit => "<unit>".to_string(),
    }
}

