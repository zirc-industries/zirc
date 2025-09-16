use std::io::{self, Write};

use winapi::shared::minwindef::DWORD;
use windows::Win32::System::Console::{GetConsoleCP, SetConsoleOutputCP};

use owo_colors::OwoColorize;
use zirc_interpreter::{Env, Interpreter, MemoryStats, Value};
use zirc_lexer::Lexer;
use zirc_parser::Parser;
use zirc_syntax::error::Error;
use zirc_syntax::token::TokenKind;

use zirc_compiler::Compiler;
use zirc_vm::Vm;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Backend { Interp, Vm }

pub fn start_repl_with_backend(backend: Backend) {
    let version = env!("CARGO_PKG_VERSION");
    println!(
        "{}",
        format!("Zirc v{} REPL (backend: {}). Type :help for help, :quit to exit.", version, match backend { Backend::Interp => "interp", Backend::Vm => "vm" })
            .bold()
            .bright_black()
    );

    match backend {
        Backend::Interp => repl_interpreter(),
        Backend::Vm => repl_vm(),
    }
}


fn repl_interpreter() {
    let mut interpreter = Interpreter::new();
    let mut env = Env::new_root();

    let utf8_cp: DWORD = 65001;
    let result = unsafe { SetConsoleOutputCP(utf8_cp) };
    if !result.as_bool() {
        let err_code = unsafe { GetConsoleCP() };
        let err_msg = format!("Failed to set console code page to UTF-8. Error code: {}", err_code);
        eprintln!("{}", err_msg.red());
        return;
    }

    let mut buffer = String::new();
    loop {
        // Use a simple, compatible character for the prompt
        // Original hexagon \u{2b22} showed as [?] in some terminals
        let prompt_char = '>';
        let prompt = if buffer.is_empty() { format!("{} ", prompt_char).white().to_string() } else { "... > ".bright_white().to_string() };
        print!("{}", prompt);
        let _ = io::stdout().flush();

        let mut line = String::new();
        let n = match io::stdin().read_line(&mut line) { Ok(n) => n, Err(_) => { println!("<input error>"); break; } };
        if n == 0 { println!("\nGoodbye."); break; }
        let trimmed = line.trim_end();

        if buffer.is_empty() && trimmed.starts_with(':') {
            match trimmed {
                ":quit" | ":q" | ":exit" => { println!("Goodbye."); break; }
                ":help" | ":h" => { print_help(); continue; }
                ":vars" => { print_vars_interp(&env); continue; }
                ":funcs" => { print_funcs_interp(&interpreter); continue; }
                ":mem" => { print_mem(&interpreter); continue; }
                ":reset" => { interpreter.reset(); env = Env::new_root(); println!("{}", "State reset.".green()); continue; }
                _ => { println!("{}", "Unknown command. Type :help.".red()); continue; }
            }
        }

        buffer.push_str(&line);
        if !is_complete(&buffer) { continue; }

        let mut lexer = Lexer::new(&buffer);
        match lexer.tokenize() {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens);
                match parser.parse_program() {
                    Ok(program) => match interpreter.run_with_env(program, &mut env) {
                        Ok(last) => {
                            if let Some(val) = last { if val != Value::Unit { println!("{}", format!("{}", val).bright_blue()); } }
                        }
                        Err(e) => render_error("Runtime error", &buffer, &e),
                    },
                    Err(e) => render_error("Parse error", &buffer, &e),
                }
            }
            Err(e) => render_error("Lex error", &buffer, &e),
        }
        buffer.clear();
    }
}

fn repl_vm() {
    let mut compiler = Compiler::new();
    let mut vm = Vm::new();
    let mut buffer = String::new();

    loop {
        let prompt = if buffer.is_empty() { "zirc(vm)> ".cyan().to_string() } else { "... > ".cyan().to_string() };
        print!("{}", prompt);
        let _ = io::stdout().flush();
        let mut line = String::new();
        let n = match io::stdin().read_line(&mut line) { Ok(n) => n, Err(_) => { println!("<input error>"); break; } };
        if n == 0 { println!("\nGoodbye."); break; }
        let trimmed = line.trim_end();

        if buffer.is_empty() && trimmed.starts_with(':') {
            match trimmed {
                ":quit" | ":q" | ":exit" => { println!("Goodbye."); break; }
                ":help" | ":h" => { print_help(); continue; }
                ":vars" => { print_vars_vm(&vm); continue; }
                ":funcs" => { print_funcs_vm(&compiler); continue; }
                ":mem" => { println!("{}", "<no memory stats in VM>".dimmed()); continue; }
                ":reset" => { compiler = Compiler::new(); vm = Vm::new(); println!("{}", "State reset.".yellow()); continue; }
                _ => { println!("{}", "Unknown command. Type :help.".red()); continue; }
            }
        }

        buffer.push_str(&line);
        if !is_complete(&buffer) { continue; }

        let mut lexer = Lexer::new(&buffer);
        match lexer.tokenize() {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens);
                match parser.parse_program() {
                    Ok(program) => match compiler.compile(program) {
                        Ok(bprog) => match vm.run(&bprog) {
                            Ok(last) => {
                                if let Some(val) = last { println!("{}", format_vm_value(&val).bright_blue()); }
                            }
                            Err(e) => render_error("VM error", &buffer, &e),
                        },
                        Err(e) => render_error("Compile error", &buffer, &e),
                    },
                    Err(e) => render_error("Parse error", &buffer, &e),
                }
            }
            Err(e) => render_error("Lex error", &buffer, &e),
        }
        buffer.clear();
    }
}

fn print_help() {
    println!(
        "{}\n  {}  Show this help\n  {}  Exit the REPL\nType code to evaluate. Use 'fun...end' and 'if...end'. Multi-line input is supported.",
        "Commands:".bold(), ":help".yellow(), ":quit".yellow()
    );
    println!(
        "  {}  List top-level variables\n  {}  List defined functions",
        ":vars".yellow(), ":funcs".yellow()
    );
    println!(
        "  {}  Show memory stats (interpreter only)\n  {}  Clear state (env/functions/mem)",
        ":mem".yellow(), ":reset".yellow()
    );
}

fn print_vars_interp(env: &Env) {
    let mut vars = env.vars_snapshot();
    vars.sort_by(|a, b| a.0.cmp(&b.0));
    if vars.is_empty() { println!("{}", "<no vars>".dimmed()); return; }
    for (k, v) in vars { println!("{} = {}", k.yellow(), format!("{}", v).bright_blue()); }
}

fn print_funcs_interp(interp: &Interpreter) {
    let names = interp.function_names();
    if names.is_empty() { println!("{}", "<no functions>".dimmed()); return; }
    for n in names { println!("{}", n.yellow()); }
}

fn print_vars_vm(vm: &Vm) {
    let vars = vm.globals_snapshot();
    if vars.is_empty() { println!("{}", "<no vars>".dimmed()); return; }
    for (k, v) in vars { println!("{} = {}", k.yellow(), format_vm_value(&v).bright_blue()); }
}

fn print_funcs_vm(compiler: &Compiler) {
    let names = compiler.function_names();
    if names.is_empty() { println!("{}", "<no functions>".dimmed()); return; }
    for n in names { println!("{}", n.yellow()); }
}

fn format_vm_value(v: &zirc_bytecode::Value) -> String {
    match v {
        zirc_bytecode::Value::Int(n) => n.to_string(),
        zirc_bytecode::Value::Str(s) => s.clone(),
        zirc_bytecode::Value::Bool(b) => if *b { "true".into() } else { "false".into() },
        zirc_bytecode::Value::List(items) => {
            let mut s = String::from("[");
            for (i, it) in items.iter().enumerate() { if i > 0 { s.push_str(", "); } s.push_str(&format_vm_value(it)); }
            s.push(']'); s
        }
        zirc_bytecode::Value::Unit => "<unit>".into(),
    }
}

fn print_mem(interp: &Interpreter) {
    let MemoryStats { strings_allocated, bytes_allocated, } = interp.memory_stats();
    println!("{}: {}", "strings".yellow(), strings_allocated);
    println!("{}: {} bytes", "bytes".yellow(), bytes_allocated);
}

fn render_error(kind: &str, source: &str, err: &Error) {
    use owo_colors::OwoColorize;
    eprintln!("{}: {}", kind.red().bold(), err.msg.red());
    if let (Some(line), Some(col)) = (err.line, err.col) {
        if let Some(src_line) = source.lines().nth(line - 1) {
            eprintln!("  {}", src_line.bright_black());
            let mut marker = String::new();
            if col > 1 { marker.push_str(&" ".repeat(col - 1)); }
            marker.push('^');
            eprintln!("  {}", marker.red());
        } else {
            eprintln!("  at {}:{}", line, col);
        }
    }
    
    // Use the same enhanced error suggestions from main.rs
    crate::provide_error_suggestions(&err.msg);
}

fn is_complete(input: &str) -> bool {
    let mut lexer = Lexer::new(input);
    let tokens = match lexer.tokenize() { Ok(t) => t, Err(_) => return false };
    let mut paren = 0i32;
    let mut starts = 0i32; // fun + if
    let mut ends = 0i32;
    for tk in tokens.iter() {
        match &tk.kind {
            TokenKind::LParen => paren += 1,
            TokenKind::RParen => paren -= 1,
            TokenKind::Fun | TokenKind::If => starts += 1,
            TokenKind::End => ends += 1,
            _ => {}
        }
    }
    paren == 0 && starts == ends
}
