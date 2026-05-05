pub mod ast;
pub mod error;
pub mod parser;
pub use ast::*;
pub use crab_lexer::{LexError, LexResult, Lexer, Token};
pub use error::*;
pub use parser::*;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lexer_simple() {
        let code = "var x = 10;";
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.len() > 0);
        assert!(matches!(tokens[0], Token::Var));
    }
    #[test]
    fn test_parser_var_decl() {
        let code = "void main() { var x = 10; }";
        let mut parser = Parser::new(code).unwrap();
        let program = parser.parse().unwrap();
        assert!(program.items.len() > 0);
    }
    #[test]
    fn test_parser_simple_expr() {
        let code = "void printNum() { var x = 10; }";
        let mut parser = Parser::new(code).unwrap();
        let program = parser.parse().unwrap();
        assert!(program.items.len() > 0);
    }
    #[test]
    fn test_null_safety_type() {
        let code = "void main() { String? name = null; }";
        let mut parser = Parser::new(code).unwrap();
        let program = parser.parse().unwrap();
        assert!(program.items.len() > 0);
    }
    #[test]
    fn test_null_coalesce_assign() {
        let code = "void main() { var x; x ??= \"test\"; }";
        let mut parser = Parser::new(code).unwrap();
        let program = parser.parse().unwrap();
        assert!(program.items.len() > 0);
        if let TopLevelItem::FunctionDecl(func) = &program.items[0] {
            if let FunctionBody::Block(stmts) = &func.body {
                println!("Statements count: {}", stmts.len());
                for (i, stmt) in stmts.iter().enumerate() {
                    match stmt {
                        Statement::VarDecl(v) => println!("Stmt {}: VarDecl({})", i, v.name),
                        Statement::Expression(expr) => {
                            let e: &Expr = expr;
                            match e {
                                Expr::NullCoalesceAssign { .. } => {
                                    println!("Stmt {}: NullCoalesceAssign", i)
                                }
                                Expr::Identifier(name) => {
                                    println!("Stmt {}: Identifier({})", i, name)
                                }
                                Expr::Literal(lit) => println!("Stmt {}: Literal({:?})", i, lit),
                                _ => println!("Stmt {}: Other expression", i),
                            }
                        }
                        _ => println!("Stmt {}: Other statement type", i),
                    }
                }
                assert_eq!(
                    stmts.len(),
                    2,
                    "Expected 2 statements: var decl and null coalesce assign"
                );
                match &stmts[1] {
                    Statement::Expression(expr) => {
                        let expr_ref: &Expr = expr;
                        match expr_ref {
                            Expr::NullCoalesceAssign { .. } => {}
                            other => panic!("Expected NullCoalesceAssign, got: {:?}", other),
                        }
                    }
                    other => panic!("Expected Expression statement, got: {:?}", other),
                }
            } else {
                panic!("Expected block body");
            }
        } else {
            panic!("Expected function decl");
        }
    }
    #[test]
    fn test_switch_statement() {
        let code = r#"void main() {
            var x = 1;
            switch (x) {
                case 1 => print("one"),
                case 2 => print("two"),
                default => print("other")
            }
        }"#;
        let mut parser = Parser::new(code).unwrap();
        let program = parser.parse().unwrap();
        assert!(program.items.len() > 0);
    }
    #[test]
    fn test_switch_with_blocks() {
        let code = r#"void main() {
            var x = 1;
            switch (x) {
                case "add" => {
                    print("adding");
                }
                case "list" => {
                    print("listing");
                }
                default => {
                    print("other");
                }
            }
        }"#;
        let mut parser = Parser::new(code).unwrap();
        let program = parser.parse().unwrap();
        assert!(program.items.len() > 0);
    }
    #[test]
    fn test_switch_string_patterns() {
        // First test: simple switch in void function
        let code1 = r#"void main() {
            var command = "test";
            switch (command) {
                case "add" => {
                    print("add");
                }
                case "list" => {
                    print("list");
                }
                default => {
                    print("other");
                }
            }
        }"#;
        // Debug: print tokens
        let mut lexer1 = Lexer::new(code1);
        let tokens1 = lexer1.tokenize().unwrap();
        println!("Tokens:");
        for (i, t) in tokens1.iter().enumerate() {
            println!("{}: {:?}", i, t);
        }

        let mut parser1 = Parser::new(code1).unwrap();
        let result1 = parser1.parse();
        assert!(result1.is_ok(), "Simple switch failed: {:?}", result1);

        // Check the AST
        let program1 = result1.unwrap();
        if let TopLevelItem::FunctionDecl(func) = &program1.items[0] {
            if let FunctionBody::Block(stmts) = &func.body {
                println!("Number of statements: {}", stmts.len());
                for (i, stmt) in stmts.iter().enumerate() {
                    println!("Statement {}: {:?}", i, std::mem::discriminant(stmt));
                }
                match &stmts[1] {
                    Statement::Switch(_) => println!("Test 1 PASSED: Found Statement::Switch"),
                    other => panic!(
                        "Test 1 FAILED: Expected Statement::Switch, got: {:?}",
                        other
                    ),
                }
            }
        }

        // Second test: nullable return type function with switch
        let code2 = r#"Command? parseArguments(List<String> args) {
            var command = args[0];
            switch (command) {
                case "add" => { return null; }
                default => { return null; }
            }
        }"#;
        let mut parser2 = Parser::new(code2).unwrap();
        let result2 = parser2.parse();
        if let Err(ref e) = result2 {
            println!("Test 2 parse error: {:?}", e);
        }
        assert!(result2.is_ok(), "Nullable return type switch failed");

        let program2 = result2.unwrap();
        if let TopLevelItem::FunctionDecl(func) = &program2.items[0] {
            if let FunctionBody::Block(stmts) = &func.body {
                match &stmts[1] {
                    Statement::Switch(_) => println!("Test 2 PASSED: Found Statement::Switch"),
                    other => panic!(
                        "Test 2 FAILED: Expected Statement::Switch, got: {:?}",
                        other
                    ),
                }
            }
        }
    }
    #[test]
    fn test_class_with_override_method() {
        let code = r#"sealed class Command {
            void execute(TaskRepository repo);
        }
        
        class AddCommand extends Command {
            String title;
            int priority;
            
            AddCommand(this.title, this.priority);
            
            @override
            void execute(TaskRepository repo) {
                print("Executing");
            }
        }"#;
        let mut parser = Parser::new(code).unwrap();
        let result = parser.parse();
        if let Err(ref e) = result {
            println!("Parse error: {:?}", e);
        }
        assert!(result.is_ok());
        let program = result.unwrap();

        println!("Number of items: {}", program.items.len());
        for (i, item) in program.items.iter().enumerate() {
            match item {
                TopLevelItem::ClassDecl(c) => println!("Item {}: ClassDecl({})", i, c.name),
                TopLevelItem::FunctionDecl(f) => println!("Item {}: FunctionDecl({})", i, f.name),
                _ => println!("Item {}: Other", i),
            }
        }

        assert_eq!(
            program.items.len(),
            2,
            "Expected 2 items: Command trait and AddCommand class"
        );

        match &program.items[1] {
            TopLevelItem::ClassDecl(class) => {
                assert_eq!(class.name, "AddCommand");
                assert_eq!(
                    class.methods.len(),
                    1,
                    "AddCommand should have 1 method (execute)"
                );
                assert_eq!(class.methods[0].name, "execute");
            }
            other => panic!("Expected ClassDecl for AddCommand, got: {:?}", other),
        }
    }
    #[test]
    fn test_parse_enum() {
        let code = r#"enum CookieType {
    ChocolateChip,
    OatmealRaisin
}"#;
        let mut parser = Parser::new(code).unwrap();
        let program = parser.parse().unwrap();
        assert_eq!(program.items.len(), 1, "Should parse 1 enum");
    }
}
