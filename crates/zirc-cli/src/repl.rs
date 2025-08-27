use std::io::{self, Write};

use owo_colors::OwoColorize;
use zirc_interpreter::{Env, Interpreter, MemoryStats, Value};
use zirc_lexer::Lexer;
use zirc_parser::Parser;
use zirc_syntax::error::Error;
use zirc_syntax::token::TokenKind;

pub fn start_repl() {
    println!("{}", "Zirc REPL. Type :help for help, :quit to exit.".bold().green());

    let mut interpreter = Interpreter::new();
    let mut env = Env::new_root();

    let mut buffer = String::new();
    loop {
        let prompt = if buffer.is_empty() { "zirc> ".cyan().to_string() } else { "... > ".cyan().to_string() };
        print!("{}", prompt);
        let _ = io::stdout().flush();

        let mut line = String::new();
        let n = match io::stdin().read_line(&mut line) {
            Ok(n) => n,
            Err(_) => { println!("<input error>"); break; }
        };
        if n == 0 { // EOF
            println!("\nGoodbye.");
            break;
        }
        let trimmed = line.trim_end();

        if buffer.is_empty() && trimmed.starts_with(':') {
            match trimmed {
                ":quit" | ":q" | ":exit" => { println!("Goodbye."); break; }
                ":help" | ":h" => {
                    println!("{}\n  {}  {}\n  {}  {}\n{}", 
                        "Commands:".bold(),
                        ":help".yellow(), "Show this help",
                        ":quit".yellow(), "Exit the REPL",
                        "Type code to evaluate. Use 'fun...end' and 'if...end'. Multi-line input is supported.");
                    println!("  {}  {}\n  {}  {}",
                        ":vars".yellow(), "List top-level variables",
                        ":funcs".yellow(), "List defined functions");
                    println!("  {}  {}\n  {}  {}",
                        ":mem".yellow(), "Show memory stats",
                        ":reset".yellow(), "Clear state (env/functions/mem)");
                    continue;
                }
                ":vars" => { print_vars(&env); continue; }
                ":funcs" => { print_funcs(&interpreter); continue; }
                ":mem" => { print_mem(&interpreter); continue; }
                ":reset" => { interpreter.reset(); env = Env::new_root(); println!("{}", "State reset.".yellow()); continue; }
                _ => { println!("{}", "Unknown command. Type :help.".red()); continue; }
            }
        }

        buffer.push_str(&line);

        if !is_complete(&buffer) {
            continue;
        }

        let mut lexer = Lexer::new(&buffer);
        match lexer.tokenize() {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens);
                match parser.parse_program() {
                    Ok(program) => {
                        match interpreter.run_with_env(program, &mut env) {
                            Ok(last) => {
                                if let Some(val) = last { if val != Value::Unit { println!("{}", format!("{}", val).bright_blue()); } }
                            }
                            Err(e) => render_error("Runtime error", &buffer, &e),
                        }
                    }
                    Err(e) => render_error("Parse error", &buffer, &e),
                }
            }
            Err(e) => render_error("Lex error", &buffer, &e),
        }

        buffer.clear();
    }
}

fn print_vars(env: &Env) {
    let mut vars = env.vars_snapshot();
    vars.sort_by(|a,b| a.0.cmp(&b.0));
    if vars.is_empty() { println!("{}", "<no vars>".dimmed()); return; }
    for (k,v) in vars { println!("{} = {}", k.yellow(), format!("{}", v).bright_blue()); }
}

fn print_funcs(interp: &Interpreter) {
    let names = interp.function_names();
    if names.is_empty() { println!("{}", "<no functions>".dimmed()); return; }
    for n in names { println!("{}", n.yellow()); }
}

fn print_mem(interp: &Interpreter) {
    let MemoryStats { strings_allocated, bytes_allocated } = interp.memory_stats();
    println!("{}: {}", "strings".yellow(), strings_allocated);
    println!("{}: {} bytes", "bytes".yellow(), bytes_allocated);
}

fn render_error(kind: &str, source: &str, err: &Error) {
    use owo_colors::OwoColorize;
    eprintln!("{}: {}", kind.red().bold(), err.msg.red());
    if let (Some(line), Some(col)) = (err.line, err.col) {
        if let Some(src_line) = source.lines().nth(line - 1) {
            eprintln!("  {}", format!("{}", src_line).bright_black());
            let mut marker = String::new();
            if col > 1 { marker.push_str(&" ".repeat(col - 1)); }
            marker.push('^');
            eprintln!("  {}", marker.red());
        } else {
            eprintln!("  at {}:{}", line, col);
        }
    }
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

