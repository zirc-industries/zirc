mod repl;

use std::fs;

use owo_colors::OwoColorize;
use zirc_interpreter::Interpreter;
use zirc_lexer::Lexer;
use zirc_parser::Parser;
use zirc_syntax::error::Error;

// VM backend imports
use zirc_compiler::Compiler;
use zirc_vm::Vm;

pub fn provide_error_suggestions(err_msg: &str) {
    use owo_colors::OwoColorize;
    
    // Variable-related errors
    if err_msg.contains("Undefined variable") {
        eprintln!("{}", "💡 Help: Did you forget to declare this variable with 'let'?".yellow());
        eprintln!("    {}", "Example: let my_var = 42".bright_black());
    }
    
    // Function-related errors
    else if err_msg.contains("Undefined function") {
        eprintln!("{}", "💡 Help: Check if the function name is spelled correctly or if it's defined.".yellow());
        eprintln!("    {}", "Available built-ins: show, showf, len, abs, min, max, pow, sqrt, upper, lower, trim, split, join, int, str, type".bright_black());
        eprintln!("    {}", "Example: fun my_func(x): x * 2 end".bright_black());
        
        // Suggest common typos
        if err_msg.contains("'show'") {
            eprintln!("    {}", "Did you mean: show() or showf()?".cyan());
        } else if err_msg.contains("'print'") {
            eprintln!("    {}", "Did you mean: show() (Zirc uses 'show', not 'print')?".cyan());
        } else if err_msg.contains("'len'") {
            eprintln!("    {}", "Make sure you're calling it as: len(my_list) or len(my_string)".cyan());
        }
    }
    
    // Type-related errors
    else if err_msg.contains("Type mismatch") {
        eprintln!("{}", "💡 Help: Make sure the value matches the declared type annotation.".yellow());
        eprintln!("    {}", "Zirc has types: int, string, bool, list, unit".bright_black());
        eprintln!("    {}", "Example: let x: int = 42".bright_black());
    }
    
    // Arithmetic errors
    else if err_msg.contains("Cannot add") {
        eprintln!("{}", "💡 Help: Addition works with compatible types:".yellow());
        eprintln!("    {}", "• Numbers: 5 + 3 = 8".bright_black());
        eprintln!("    {}", "• Strings: \"hello\" + \" world\" = \"hello world\"".bright_black());
        eprintln!("    {}", "• Lists: [1, 2] + [3, 4] = [1, 2, 3, 4]".bright_black());
    }
    else if err_msg.contains("Cannot subtract") || err_msg.contains("Cannot multiply") || err_msg.contains("Cannot divide") {
        eprintln!("{}", "💡 Help: Arithmetic operations work only with numbers.".yellow());
        eprintln!("    {}", "Example: 10 - 3, 4 * 5, 15 / 3".bright_black());
    }
    else if err_msg.contains("division by zero") {
        eprintln!("{}", "💡 Help: You cannot divide by zero.".yellow());
        eprintln!("    {}", "Check if the divisor is zero before the operation.".bright_black());
    }
    
    // Index errors
    else if err_msg.contains("index out of bounds") {
        eprintln!("{}", "💡 Help: Index is outside the valid range.".yellow());
        eprintln!("    {}", "• Lists and strings are 0-indexed".bright_black());
        eprintln!("    {}", "• Use len() to check size: if i < len(my_list): my_list[i] end".bright_black());
    }
    
    // Syntax errors
    else if err_msg.contains("Unexpected token") {
        eprintln!("{}", "💡 Help: Syntax error detected.".yellow());
        if err_msg.contains("'end'") {
            eprintln!("    {}", "Did you forget an 'end' keyword for a function or if statement?".bright_black());
        } else if err_msg.contains("'('") {
            eprintln!("    {}", "Check if parentheses are balanced".bright_black());
        } else if err_msg.contains("'='") {
            eprintln!("    {}", "Use '==' for comparison, '=' for assignment".bright_black());
        }
    }
    else if err_msg.contains("Expected") {
        eprintln!("{}", "💡 Help: Missing required syntax element.".yellow());
        if err_msg.contains("'end'") {
            eprintln!("    {}", "Every 'fun' and 'if' needs a matching 'end'".bright_black());
            eprintln!("    {}", "Example: fun test(): showf(\"hello\") end".bright_black());
        } else if err_msg.contains("identifier") {
            eprintln!("    {}", "Expected a variable or function name".bright_black());
        }
    }
    
    // Control flow errors
    else if err_msg.contains("'break' outside of loop") {
        eprintln!("{}", "💡 Help: 'break' can only be used inside while or for loops.".yellow());
        eprintln!("    {}", "Example: while condition: if done: break end end".bright_black());
    }
    else if err_msg.contains("'continue' outside of loop") {
        eprintln!("{}", "💡 Help: 'continue' can only be used inside while or for loops.".yellow());
        eprintln!("    {}", "Example: for i in 0..10: if i == 5: continue end end".bright_black());
    }
    
    // Function call errors
    else if err_msg.contains("expected") && err_msg.contains("args") {
        eprintln!("{}", "💡 Help: Function called with wrong number of arguments.".yellow());
        eprintln!("    {}", "Check the function signature and provide the correct number of arguments".bright_black());
    }
    
    // File-related errors
    else if err_msg.contains("Failed to read file") {
        eprintln!("{}", "💡 Help: File operation failed.".yellow());
        eprintln!("    {}", "Check if the file exists and you have permission to read it".bright_black());
    }
    else if err_msg.contains("Failed to write file") {
        eprintln!("{}", "💡 Help: File write operation failed.".yellow());
        eprintln!("    {}", "Check if you have permission to write to that location".bright_black());
    }
    
    // Built-in function specific errors
    else if err_msg.contains("showf missing") {
        eprintln!("{}", "💡 Help: Format string needs more arguments.".yellow());
        eprintln!("    {}", "Use %d for numbers, %s for strings: showf(\"Number: %d\", 42)".bright_black());
    }
    else if err_msg.contains("sqrt() argument cannot be negative") {
        eprintln!("{}", "💡 Help: Square root of negative numbers is not supported.".yellow());
        eprintln!("    {}", "Use abs() first if needed: sqrt(abs(x))".bright_black());
    }
    else if err_msg.contains("pow() exponent cannot be negative") {
        eprintln!("{}", "💡 Help: Negative exponents are not supported in pow().".yellow());
        eprintln!("    {}", "Use only non-negative integers: pow(2, 3) = 8".bright_black());
    }
    
    // General parsing errors
    else if err_msg.contains("Unterminated string") {
        eprintln!("{}", "💡 Help: String is missing closing quote.".yellow());
        eprintln!("    {}", "Make sure every \" has a matching closing \"".bright_black());
    }
    else if err_msg.contains("Invalid number") {
        eprintln!("{}", "💡 Help: Number format is not recognized.".yellow());
        eprintln!("    {}", "Use integers like: 42, 100, -5".bright_black());
    }
    
    // Stack/memory errors
    else if err_msg.contains("stack underflow") || err_msg.contains("stack overflow") {
        eprintln!("{}", "💡 Help: Internal VM error - this might be a compiler bug.".yellow());
        eprintln!("    {}", "Try using the interpreter backend: zirc-cli --backend interp file.zirc".bright_black());
    }
}

fn render_error(kind: &str, source: &str, err: &Error) {
    eprintln!("{}: {}", kind.red().bold(), err.msg.red());
    if let (Some(line), Some(col)) = (err.line, err.col) {
        eprintln!("  --> line {}, column {}", line, col);
        if let Some(src_line) = source.lines().nth(line - 1) {
            let line_num_str = format!("{:3} | ", line);
            eprintln!("     |");
            eprintln!("{}{}", line_num_str.bright_black(), src_line);
            
            let mut marker = String::new();
            marker.push_str(&" ".repeat(line_num_str.len()));
            if col > 1 {
                marker.push_str(&" ".repeat(col - 1));
            }
            marker.push_str("^");
            eprintln!("{}{}", marker.red(), " error here".red());
            eprintln!("     |");
        }
    }
    
    // Add helpful suggestions based on common errors
    provide_error_suggestions(&err.msg);
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

fn normalize_path(p: &str) -> std::path::PathBuf {
    let pb = std::path::PathBuf::from(p);
    if pb.exists() {
        return pb;
    }
    #[cfg(windows)]
    {
        let alt = p.replace('/', std::path::MAIN_SEPARATOR_STR);
        let altpb = std::path::PathBuf::from(&alt);
        if altpb.exists() { return altpb; }
    }
    pb
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Support --version / -V for installer validation and quick checks
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("Zirc {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if args.len() < 2 {
        let backend = parse_backend(&args);
        let mode = if backend == "vm" { repl::Backend::Vm } else { repl::Backend::Interp };
        repl::start_repl_with_backend(mode);
        return;
    }

    let backend = parse_backend(&args);

    // first non-flag arg treated as path, skipping flag values
    let path_str = match parse_path(&args) {
        Some(p) => p,
        None => {
            let mode = if backend == "vm" { repl::Backend::Vm } else { repl::Backend::Interp };
            repl::start_repl_with_backend(mode);
            return;
        }
    };
    let path_buf = normalize_path(path_str);
    if !path_buf.exists() {
        eprintln!(
            "{}: {}",
            "error".red().bold(),
            format!("File not found: {}", path_str).red()
        );
        std::process::exit(1);
    }
    let src = match fs::read_to_string(&path_buf) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "{}: {}",
                "error".red().bold(),
                format!("Failed to read {}: {}", path_buf.display(), e).red()
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
