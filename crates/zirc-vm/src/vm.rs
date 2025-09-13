//! Zirc VM core.

use std::io::{self, Write};
use std::fs;
use std::collections::HashMap;

use crate::display::display_value;
use zirc_bytecode::{Builtin, Instruction, Program, Value};
use zirc_syntax::error::{Result, error};

#[derive(Clone)]
struct Frame {
    func_ref: CodeRef,
    ip: usize,
    locals: Vec<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use zirc_bytecode::{Function, Instruction};

    fn make_simple_program(main_code: Vec<Instruction>) -> Program {
        Program {
            functions: Vec::new(),
            main: Function {
                name: "main".to_string(),
                arity: 0,
                local_count: 1,
                code: main_code,
            },
        }
    }

    #[test]
    fn test_vm_basic_operations() {
        let mut vm = Vm::new();
        
        // Test basic arithmetic: 5 + 3
        let program = make_simple_program(vec![
            Instruction::PushInt(5),
            Instruction::PushInt(3),
            Instruction::Add,
        ]);
        
        let result = vm.run(&program).unwrap();
        assert_eq!(result, None); // No explicit return, so None
        
        // Check that the stack has the result
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack[0], Value::Int(8));
    }

    #[test]
    fn test_vm_arithmetic_operations() {
        let test_cases = vec![
            (vec![Instruction::PushInt(10), Instruction::PushInt(3), Instruction::Add], Value::Int(13)),
            (vec![Instruction::PushInt(10), Instruction::PushInt(3), Instruction::Sub], Value::Int(7)),
            (vec![Instruction::PushInt(10), Instruction::PushInt(3), Instruction::Mul], Value::Int(30)),
            (vec![Instruction::PushInt(10), Instruction::PushInt(3), Instruction::Div], Value::Int(3)),
        ];
        
        for (code, expected) in test_cases {
            let mut vm = Vm::new();
            let program = make_simple_program(code);
            
            vm.run(&program).unwrap();
            assert_eq!(vm.stack[0], expected);
        }
    }

    #[test]
    fn test_vm_comparison_operations() {
        let test_cases = vec![
            (vec![Instruction::PushInt(5), Instruction::PushInt(3), Instruction::Lt], Value::Bool(false)),
            (vec![Instruction::PushInt(3), Instruction::PushInt(5), Instruction::Lt], Value::Bool(true)),
            (vec![Instruction::PushInt(5), Instruction::PushInt(5), Instruction::Eq], Value::Bool(true)),
            (vec![Instruction::PushInt(5), Instruction::PushInt(3), Instruction::Eq], Value::Bool(false)),
            (vec![Instruction::PushInt(5), Instruction::PushInt(3), Instruction::Ne], Value::Bool(true)),
        ];
        
        for (code, expected) in test_cases {
            let mut vm = Vm::new();
            let program = make_simple_program(code);
            
            vm.run(&program).unwrap();
            assert_eq!(vm.stack[0], expected);
        }
    }

    #[test]
    fn test_vm_string_operations() {
        let mut vm = Vm::new();
        
        // Test string concatenation
        let program = make_simple_program(vec![
            Instruction::PushStr("Hello, ".to_string()),
            Instruction::PushStr("World!".to_string()),
            Instruction::Add,
        ]);
        
        vm.run(&program).unwrap();
        assert_eq!(vm.stack[0], Value::Str("Hello, World!".to_string()));
    }

    #[test]
    fn test_vm_boolean_operations() {
        let mut vm = Vm::new();
        
        // Test boolean negation
        let program = make_simple_program(vec![
            Instruction::PushBool(true),
            Instruction::Not,
        ]);
        
        vm.run(&program).unwrap();
        assert_eq!(vm.stack[0], Value::Bool(false));
    }

    #[test]
    fn test_vm_local_variables() {
        let mut vm = Vm::new();
        
        // Test local variable storage and retrieval
        let program = make_simple_program(vec![
            Instruction::PushInt(42),
            Instruction::StoreLocal(0),
            Instruction::LoadLocal(0),
        ]);
        
        vm.run(&program).unwrap();
        assert_eq!(vm.stack[0], Value::Int(42));
    }

    #[test]
    fn test_vm_global_variables() {
        let mut vm = Vm::new();
        
        // Test global variable storage and retrieval
        let program = make_simple_program(vec![
            Instruction::PushStr("test".to_string()),
            Instruction::StoreGlobal("x".to_string()),
            Instruction::LoadGlobal("x".to_string()),
        ]);
        
        vm.run(&program).unwrap();
        assert_eq!(vm.stack[0], Value::Str("test".to_string()));
        
        // Check globals snapshot
        let globals = vm.globals_snapshot();
        assert_eq!(globals.len(), 1);
        assert_eq!(globals[0], ("x".to_string(), Value::Str("test".to_string())));
    }

    #[test]
    fn test_vm_list_operations() {
        let mut vm = Vm::new();
        
        // Test list creation and indexing
        let program = make_simple_program(vec![
            Instruction::PushInt(1),
            Instruction::PushInt(2),
            Instruction::PushInt(3),
            Instruction::MakeList(3),
            Instruction::PushInt(1),
            Instruction::Index,
        ]);
        
        vm.run(&program).unwrap();
        assert_eq!(vm.stack[0], Value::Int(2)); // Index 1 should be 2
    }

    #[test]
    fn test_vm_conditional_jumps() {
        let mut vm = Vm::new();
        
        // Test conditional jump (if true, skip next instruction)
        let program = make_simple_program(vec![
            Instruction::PushBool(true),
            Instruction::JumpIfFalse(4), // Should not jump
            Instruction::PushInt(1),     // Should execute
            Instruction::Jump(5),        // Skip next instruction
            Instruction::PushInt(2),     // Should skip
        ]);
        
        vm.run(&program).unwrap();
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack[0], Value::Int(1));
    }

    #[test]
    fn test_vm_division_by_zero() {
        let mut vm = Vm::new();
        
        // Test division by zero error
        let program = make_simple_program(vec![
            Instruction::PushInt(10),
            Instruction::PushInt(0),
            Instruction::Div,
        ]);
        
        let result = vm.run(&program);
        assert!(result.is_err());
        assert!(result.unwrap_err().msg.contains("division by zero"));
    }

    #[test]
    fn test_vm_stack_underflow() {
        let mut vm = Vm::new();
        
        // Test stack underflow in Add
        let program = make_simple_program(vec![
            Instruction::PushInt(5),
            Instruction::Add, // Only one operand, should cause underflow
        ]);
        
        let result = vm.run(&program);
        assert!(result.is_err());
        assert!(result.unwrap_err().msg.contains("stack underflow"));
    }

    #[test]
    fn test_vm_invalid_index() {
        let mut vm = Vm::new();
        
        // Test out of bounds list index
        let program = make_simple_program(vec![
            Instruction::PushInt(1),
            Instruction::PushInt(2),
            Instruction::MakeList(2),
            Instruction::PushInt(5), // Index 5 is out of bounds
            Instruction::Index,
        ]);
        
        let result = vm.run(&program);
        assert!(result.is_err());
        assert!(result.unwrap_err().msg.contains("index out of bounds"));
    }

    #[test] 
    fn test_vm_builtin_len() {
        let mut vm = Vm::new();
        
        // Test len() builtin with string
        let program = make_simple_program(vec![
            Instruction::PushStr("hello".to_string()),
            Instruction::BuiltinCall(Builtin::Len, 1),
        ]);
        
        vm.run(&program).unwrap();
        assert_eq!(vm.stack[0], Value::Int(5));
    }

    #[test]
    fn test_vm_pop_operation() {
        let mut vm = Vm::new();
        
        // Test Pop instruction
        let program = make_simple_program(vec![
            Instruction::PushInt(42),
            Instruction::Pop,
        ]);
        
        let result = vm.run(&program).unwrap();
        assert_eq!(result, Some(Value::Int(42))); // Pop sets last_value
        assert_eq!(vm.stack.len(), 0); // Stack should be empty
    }
}

#[derive(Clone, Copy)]
enum CodeRef {
    Main,
    Func(usize),
}

pub struct Vm {
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
}

impl Default for Vm { fn default() -> Self { Self::new() } }

impl Vm {
    pub fn new() -> Self {
        Self { stack: Vec::new(), globals: HashMap::new() }
    }

    pub fn globals_snapshot(&self) -> Vec<(String, Value)> {
        let mut v: Vec<(String, Value)> = self.globals.iter().map(|(k, val)| (k.clone(), val.clone())).collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    }

    pub fn run(&mut self, program: &Program) -> Result<Option<Value>> {
        let mut frames: Vec<Frame> = Vec::new();
        frames.push(Frame {
            func_ref: CodeRef::Main,
            ip: 0,
            locals: vec![Value::Unit; program.main.local_count],
        });

        let mut last_value: Option<Value> = None;
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
                Instruction::Pop => { let v = self.stack.pop(); if let Some(val) = v { last_value = Some(val); } }
                Instruction::Add => {
                    let b = self.stack.pop().ok_or_else(|| "stack underflow in Add")?;
                    let a = self.stack.pop().ok_or_else(|| "stack underflow in Add")?;
                    match (a, b) {
                        (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(x + y)),
                        (Value::Str(x), Value::Str(y)) => self.stack.push(Value::Str(format!("{}{}", x, y))),
                        (Value::List(mut x), Value::List(y)) => { x.extend(y); self.stack.push(Value::List(x)); }
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
                Instruction::Jump(tgt) => { frame.ip = tgt; }
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
                                } else { return error("prompt() prompt must be string"); }
                            }
                            let input = if silent {
                                std::env::var("ZIRC_BENCH_PROMPT_REPLY").unwrap_or_default()
                            } else {
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
                        Builtin::Len => {
                            if args.len() != 1 { return error("len() expects exactly 1 argument"); }
                            match &args[0] {
                                Value::Str(s) => self.stack.push(Value::Int(s.chars().count() as i64)),
                                Value::List(items) => self.stack.push(Value::Int(items.len() as i64)),
                                other => return error(format!("len() expects string or list, got {:?}", other)),
                            }
                        }
                        Builtin::Push => {
                            return error("push() is not supported in VM mode - use the interpreter backend");
                        }
                        Builtin::Pop => {
                            return error("pop() is not supported in VM mode - use the interpreter backend");
                        }
                        Builtin::Slice => {
                            if args.len() != 3 { return error("slice() expects exactly 3 arguments: collection, start, end"); }
                            
                            let start = match &args[1] {
                                Value::Int(n) => *n,
                                other => return error(format!("slice() start index must be int, got {:?}", other)),
                            };
                            let end = match &args[2] {
                                Value::Int(n) => *n,
                                other => return error(format!("slice() end index must be int, got {:?}", other)),
                            };
                            
                            if start < 0 { return error("slice() start index cannot be negative"); }
                            if end < start { return error("slice() end index must be >= start index"); }
                            
                            match &args[0] {
                                Value::Str(s) => {
                                    let chars: Vec<char> = s.chars().collect();
                                    let start_idx = start as usize;
                                    let end_idx = (end as usize).min(chars.len());
                                    
                                    if start_idx >= chars.len() {
                                        self.stack.push(Value::Str(String::new()));
                                    } else {
                                        let slice: String = chars[start_idx..end_idx].iter().collect();
                                        self.stack.push(Value::Str(slice));
                                    }
                                },
                                Value::List(items) => {
                                    let start_idx = start as usize;
                                    let end_idx = (end as usize).min(items.len());
                                    
                                    if start_idx >= items.len() {
                                        self.stack.push(Value::List(Vec::new()));
                                    } else {
                                        self.stack.push(Value::List(items[start_idx..end_idx].to_vec()));
                                    }
                                },
                                other => return error(format!("slice() expects string or list, got {:?}", other)),
                            }
                        }
                        // Mathematical functions
                        Builtin::Abs => {
                            if args.len() != 1 { return error("abs() expects exactly 1 argument"); }
                            match &args[0] {
                                Value::Int(n) => self.stack.push(Value::Int(n.abs())),
                                other => return error(format!("abs() expects int, got {:?}", other)),
                            }
                        }
                        Builtin::Min => {
                            if args.len() != 2 { return error("min() expects exactly 2 arguments"); }
                            match (&args[0], &args[1]) {
                                (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(*x.min(y))),
                                _ => return error("min() expects two ints"),
                            }
                        }
                        Builtin::Max => {
                            if args.len() != 2 { return error("max() expects exactly 2 arguments"); }
                            match (&args[0], &args[1]) {
                                (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(*x.max(y))),
                                _ => return error("max() expects two ints"),
                            }
                        }
                        Builtin::Pow => {
                            if args.len() != 2 { return error("pow() expects exactly 2 arguments: base and exponent"); }
                            match (&args[0], &args[1]) {
                                (Value::Int(b), Value::Int(e)) => {
                                    if *e < 0 { return error("pow() exponent cannot be negative"); }
                                    let result = (*b as f64).powi(*e as i32) as i64;
                                    self.stack.push(Value::Int(result));
                                },
                                _ => return error("pow() expects two ints"),
                            }
                        }
                        Builtin::Sqrt => {
                            if args.len() != 1 { return error("sqrt() expects exactly 1 argument"); }
                            match &args[0] {
                                Value::Int(n) => {
                                    if *n < 0 { return error("sqrt() argument cannot be negative"); }
                                    let result = (*n as f64).sqrt() as i64;
                                    self.stack.push(Value::Int(result));
                                },
                                other => return error(format!("sqrt() expects int, got {:?}", other)),
                            }
                        }
                        // String functions
                        Builtin::Upper => {
                            if args.len() != 1 { return error("upper() expects exactly 1 argument"); }
                            match &args[0] {
                                Value::Str(s) => self.stack.push(Value::Str(s.to_uppercase())),
                                other => return error(format!("upper() expects string, got {:?}", other)),
                            }
                        }
                        Builtin::Lower => {
                            if args.len() != 1 { return error("lower() expects exactly 1 argument"); }
                            match &args[0] {
                                Value::Str(s) => self.stack.push(Value::Str(s.to_lowercase())),
                                other => return error(format!("lower() expects string, got {:?}", other)),
                            }
                        }
                        Builtin::Trim => {
                            if args.len() != 1 { return error("trim() expects exactly 1 argument"); }
                            match &args[0] {
                                Value::Str(s) => self.stack.push(Value::Str(s.trim().to_string())),
                                other => return error(format!("trim() expects string, got {:?}", other)),
                            }
                        }
                        Builtin::Split => {
                            if args.len() != 2 { return error("split() expects exactly 2 arguments: string and delimiter"); }
                            match (&args[0], &args[1]) {
                                (Value::Str(s), Value::Str(delim)) => {
                                    let parts: Vec<Value> = s.split(delim)
                                        .map(|part| Value::Str(part.to_string()))
                                        .collect();
                                    self.stack.push(Value::List(parts));
                                },
                                _ => return error("split() expects two strings"),
                            }
                        }
                        Builtin::Join => {
                            if args.len() != 2 { return error("join() expects exactly 2 arguments: list and separator"); }
                            match (&args[0], &args[1]) {
                                (Value::List(items), Value::Str(sep)) => {
                                    let strings: std::result::Result<Vec<String>, zirc_syntax::error::Error> = items.iter()
                                        .map(|item| match item {
                                            Value::Str(s) => Ok(s.clone()),
                                            other => error(format!("join() list must contain only strings, got {:?}", other)),
                                        })
                                        .collect();
                                    let result = strings?.join(sep);
                                    self.stack.push(Value::Str(result));
                                },
                                _ => return error("join() expects list and string"),
                            }
                        }
                        // Type conversion functions
                        Builtin::Int => {
                            if args.len() != 1 { return error("int() expects exactly 1 argument"); }
                            match &args[0] {
                                Value::Int(n) => self.stack.push(Value::Int(*n)),
                                Value::Str(s) => {
                                    match s.parse::<i64>() {
                                        Ok(n) => self.stack.push(Value::Int(n)),
                                        Err(_) => return error(format!("Cannot convert '{}' to int", s)),
                                    }
                                },
                                Value::Bool(true) => self.stack.push(Value::Int(1)),
                                Value::Bool(false) => self.stack.push(Value::Int(0)),
                                other => return error(format!("Cannot convert {:?} to int", other)),
                            }
                        }
                        Builtin::Str => {
                            if args.len() != 1 { return error("str() expects exactly 1 argument"); }
                            let result = match &args[0] {
                                Value::Str(s) => s.clone(),
                                Value::Int(n) => n.to_string(),
                                Value::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
                                Value::List(items) => format!("{}", display_value(&Value::List(items.clone()))),
                                Value::Unit => "<unit>".to_string(),
                            };
                            self.stack.push(Value::Str(result));
                        }
                        // Utility functions
                        Builtin::Type => {
                            if args.len() != 1 { return error("type() expects exactly 1 argument"); }
                            let type_name = match &args[0] {
                                Value::Int(_) => "int",
                                Value::Str(_) => "string",
                                Value::Bool(_) => "bool",
                                Value::List(_) => "list",
                                Value::Unit => "unit",
                            };
                            self.stack.push(Value::Str(type_name.to_string()));
                        }
                    }
                }
                Instruction::Halt => { break; }
                Instruction::LoadGlobal(name) => {
                    let v = self.globals.get(&name).cloned().ok_or_else(|| format!("Undefined variable '{}'", name))?;
                    self.stack.push(v);
                }
                Instruction::StoreGlobal(name) => {
                    let v = self.stack.pop().ok_or_else(|| "stack underflow in StoreGlobal")?;
                    self.globals.insert(name, v);
                }
            }
        }
        Ok(last_value)
    }
}

