mod repl;

use std::fs;
use std::path::Path;

use owo_colors::OwoColorize;
use zirc_interpreter::Interpreter;
use zirc_lexer::Lexer;
use zirc_parser::Parser;
use zirc_syntax::error::Error;

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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        repl::start_repl();
        return;
    }
    let path = &args[1];
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

    let mut interp = Interpreter::new();
    if let Err(e) = interp.run(program) {
        render_error("Runtime error", &src, &e);
        std::process::exit(1);
    }
}
