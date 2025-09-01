mod repl;

use std::fs;
use std::path::Path;

use owo_colors::OwoColorize;
use zirc_interpreter::Interpreter;
use zirc_lexer::Lexer;
use zirc_parser::Parser;
use zirc_syntax::error::Error;

// VM backend imports
use zirc_compiler::Compiler;
use zirc_vm::Vm;

fn render_error(kind: &str, source: &str, err: &Error) {
    eprintln!("{}: {}", kind.red().bold(), err.msg.red());
    if let (Some(line), Some(col)) = (err.line, err.col) {
        if let Some(src_line) = source.lines().nth(line - 1) {
            eprintln!("  {}", src_line.bright_black());
            let mut marker = String::new();
            if col > 1 {
                marker.push_str(&" ".repeat(col - 1));
            }
            marker.push('^');
            eprintln!("  {}", marker.red());
        } else {
            eprintln!("  at {}:{}", line, col);
        }
    }
}

fn parse_backend(args: &[String]) -> String {
    // default backend is interpreter; allow --backend vm or env var ZIRC_BACKEND=vm
    if let Ok(b) = std::env::var("ZIRC_BACKEND") {
        return b;
    }
    let mut i = 1usize;
    while i + 1 < args.len() {
        if args[i] == "--backend" || args[i] == "-b" {
            return args[i + 1].clone();
        }
        i += 1;
    }
    "interp".to_string()
}

fn parse_path<'a>(args: &'a [String]) -> Option<&'a str> {
    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--backend" | "-b" => { i += 2; }
            s if s.starts_with('-') => { i += 1; }
            _ => { return Some(args[i].as_str()); }
        }
    }
    None
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        let backend = parse_backend(&args);
        let mode = if backend == "vm" { repl::Backend::Vm } else { repl::Backend::Interp };
        repl::start_repl_with_backend(mode);
        return;
    }

    let backend = parse_backend(&args);

    // first non-flag arg treated as path, skipping flag values
    let path = match parse_path(&args) {
        Some(p) => p,
        None => {
            let mode = if backend == "vm" { repl::Backend::Vm } else { repl::Backend::Interp };
            repl::start_repl_with_backend(mode);
            return;
        }
    };
    if !Path::new(path).exists() {
        eprintln!(
            "{}: {}",
            "error".red().bold(),
            format!("File not found: {}", path).red()
        );
        std::process::exit(1);
    }
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "{}: {}",
                "error".red().bold(),
                format!("Failed to read {}: {}", path, e).red()
            );
            std::process::exit(1);
        }
    };

    let mut lexer = Lexer::new(&src);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            render_error("Lex error", &src, &e);
            std::process::exit(1);
        }
    };

    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            render_error("Parse error", &src, &e);
            std::process::exit(1);
        }
    };

    if backend == "vm" {
        let mut compiler = Compiler::new();
        let bprog = match compiler.compile(program) {
            Ok(p) => p,
            Err(e) => {
                render_error("Compile error", &src, &e);
                std::process::exit(1);
            }
        };
        let mut vm = Vm::new();
        if let Err(e) = vm.run(&bprog) {
            render_error("VM error", &src, &e);
            std::process::exit(1);
        }
    } else {
        let mut interp = Interpreter::new();
        if let Err(e) = interp.run(program) {
            render_error("Runtime error", &src, &e);
            std::process::exit(1);
        }
    }
}
