pub mod builder;
pub mod compiler;

pub use compiler::Compiler;

#[cfg(test)]
mod tests {
    use super::*;
    use zirc_syntax::ast::*;
    use zirc_bytecode::{Instruction, Builtin};

    fn create_simple_program(items: Vec<Item>) -> Program {
        Program { items }
    }

    #[test]
    fn test_compiler_new() {
        let compiler = Compiler::new();
        assert!(compiler.function_names().is_empty());
    }

    #[test]
    fn test_compile_simple_expression() {
        let mut compiler = Compiler::new();
        
        // Program: let x = 5 + 3
        let program = create_simple_program(vec![
            Item::Stmt(Stmt::Let {
                name: "x".to_string(),
                ty: None,
                expr: Expr::BinaryAdd(
                    Box::new(Expr::LiteralInt(5)),
                    Box::new(Expr::LiteralInt(3)),
                ),
            }),
        ]);
        
        let bytecode = compiler.compile(program).unwrap();
        
        // Check that main function was created
        assert_eq!(bytecode.main.name, "__main");
        assert_eq!(bytecode.main.arity, 0);
        
        // Check the bytecode instructions
        let expected_ops = [
            Instruction::PushInt(5),
            Instruction::PushInt(3),
            Instruction::Add,
            Instruction::StoreGlobal("x".to_string()),
        ];
        
        for (i, expected) in expected_ops.iter().enumerate() {
            assert_eq!(&bytecode.main.code[i], expected);
        }
    }

    #[test]
    fn test_compile_function_definition() {
        let mut compiler = Compiler::new();
        
        // Program: fun add(a, b): a + b end
        let program = create_simple_program(vec![
            Item::Function(Function {
                name: "add".to_string(),
                params: vec![
                    Param { name: "a".to_string(), ty: None },
                    Param { name: "b".to_string(), ty: None },
                ],
                return_type: None,
                body: vec![
                    Stmt::Return(Some(Expr::BinaryAdd(
                        Box::new(Expr::Ident("a".to_string())),
                        Box::new(Expr::Ident("b".to_string())),
                    ))),
                ],
            }),
        ]);
        
        let bytecode = compiler.compile(program).unwrap();
        
        // Check that function was registered
        assert_eq!(compiler.function_names(), vec!["add".to_string()]);
        
        // Check the function in the program
        assert_eq!(bytecode.functions.len(), 1);
        let func = &bytecode.functions[0];
        assert_eq!(func.name, "add");
        assert_eq!(func.arity, 2);
        
        // Check first few instructions (load a, load b, add)
        assert_eq!(func.code[0], Instruction::LoadLocal(0)); // load a
        assert_eq!(func.code[1], Instruction::LoadLocal(1)); // load b
        assert_eq!(func.code[2], Instruction::Add);
        assert_eq!(func.code[3], Instruction::Return);
    }

    #[test]
    fn test_compile_builtin_function_call() {
        let mut compiler = Compiler::new();
        
        // Program: show(42)
        let program = create_simple_program(vec![
            Item::Stmt(Stmt::ExprStmt(Expr::Call {
                name: "show".to_string(),
                args: vec![Expr::LiteralInt(42)],
            })),
        ]);
        
        let bytecode = compiler.compile(program).unwrap();
        
        // Check the bytecode contains builtin call
        assert_eq!(bytecode.main.code[0], Instruction::PushInt(42));
        assert_eq!(bytecode.main.code[1], Instruction::BuiltinCall(Builtin::Show, 1));
        assert_eq!(bytecode.main.code[2], Instruction::Pop);
    }

    #[test]
    fn test_compile_if_statement() {
        let mut compiler = Compiler::new();
        
        // Program: if true: let x = 1 else let x = 2 end
        let program = create_simple_program(vec![
            Item::Stmt(Stmt::If {
                cond: Expr::LiteralBool(true),
                then_body: vec![
                    Stmt::Let {
                        name: "x".to_string(),
                        ty: None,
                        expr: Expr::LiteralInt(1),
                    },
                ],
                else_body: vec![
                    Stmt::Let {
                        name: "x".to_string(),
                        ty: None,
                        expr: Expr::LiteralInt(2),
                    },
                ],
            }),
        ]);
        
        let bytecode = compiler.compile(program).unwrap();
        
        // Should have conditional jumps
        assert_eq!(bytecode.main.code[0], Instruction::PushBool(true));
        assert!(matches!(bytecode.main.code[1], Instruction::JumpIfFalse(_)));
    }

    #[test]
    fn test_compile_while_loop() {
        let mut compiler = Compiler::new();
        
        // Program: while true: break end
        let program = create_simple_program(vec![
            Item::Stmt(Stmt::While {
                cond: Expr::LiteralBool(true),
                body: vec![Stmt::Break],
            }),
        ]);
        
        let bytecode = compiler.compile(program).unwrap();
        
        // Should have loop structure with jumps
        assert_eq!(bytecode.main.code[0], Instruction::PushBool(true));
        assert!(matches!(bytecode.main.code[1], Instruction::JumpIfFalse(_)));
        assert!(matches!(bytecode.main.code[2], Instruction::Jump(_))); // break
    }

    #[test]
    fn test_compile_list_operations() {
        let mut compiler = Compiler::new();
        
        // Program: let arr = [1, 2, 3]; arr[1]
        let program = create_simple_program(vec![
            Item::Stmt(Stmt::Let {
                name: "arr".to_string(),
                ty: None,
                expr: Expr::List(vec![
                    Expr::LiteralInt(1),
                    Expr::LiteralInt(2),
                    Expr::LiteralInt(3),
                ]),
            }),
            Item::Stmt(Stmt::ExprStmt(Expr::Index(
                Box::new(Expr::Ident("arr".to_string())),
                Box::new(Expr::LiteralInt(1)),
            ))),
        ]);
        
        let bytecode = compiler.compile(program).unwrap();
        
        // Check list creation
        assert_eq!(bytecode.main.code[0], Instruction::PushInt(1));
        assert_eq!(bytecode.main.code[1], Instruction::PushInt(2));
        assert_eq!(bytecode.main.code[2], Instruction::PushInt(3));
        assert_eq!(bytecode.main.code[3], Instruction::MakeList(3));
        assert_eq!(bytecode.main.code[4], Instruction::StoreGlobal("arr".to_string()));
        
        // Check indexing
        assert_eq!(bytecode.main.code[5], Instruction::LoadGlobal("arr".to_string()));
        assert_eq!(bytecode.main.code[6], Instruction::PushInt(1));
        assert_eq!(bytecode.main.code[7], Instruction::Index);
    }

    #[test]
    fn test_builtin_of_function() {
        use crate::compiler::builtin_of;
        
        assert_eq!(builtin_of("show"), Some(Builtin::Show));
        assert_eq!(builtin_of("showf"), Some(Builtin::ShowF));
        assert_eq!(builtin_of("len"), Some(Builtin::Len));
        assert_eq!(builtin_of("unknown"), None);
    }

    #[test]
    fn test_duplicate_function_error() {
        let mut compiler = Compiler::new();
        
        // Program with duplicate function names
        let program = create_simple_program(vec![
            Item::Function(Function {
                name: "test".to_string(),
                params: vec![],
                return_type: None,
                body: vec![],
            }),
            Item::Function(Function {
                name: "test".to_string(), // Duplicate!
                params: vec![],
                return_type: None,
                body: vec![],
            }),
        ]);
        
        let result = compiler.compile(program);
        assert!(result.is_err());
        assert!(result.unwrap_err().msg.contains("Duplicate function"));
    }

    #[test]
    fn test_compiler_default() {
        let compiler = Compiler::default();
        assert!(compiler.function_names().is_empty());
    }
}
