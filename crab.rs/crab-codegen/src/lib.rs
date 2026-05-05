pub mod c_interop;
pub mod codegen;
pub mod null_safety;
pub mod type_mapper;

pub use c_interop::CInterop;
pub use codegen::CodeGenerator;
pub use null_safety::NullSafety;
pub use type_mapper::TypeMapper;
#[cfg(test)]
mod tests {
    use super::*;
    use crab_parser::Parser;
    #[test]
    fn test_async_function_codegen() {
        let code = r#"Future<int> getData() async => 42;"#;
        let mut parser = Parser::new(code).unwrap();
        let program = parser.parse().unwrap();
        let mut codegen = CodeGenerator::new();
        let rust = codegen.generate(&program);
        assert!(
            rust.contains("async fn getData() -> i64"),
            "Generated: {}",
            rust
        );
    }
    #[test]
    fn test_async_main_with_tokio_macro() {
        let code = r#"void main() async { print("test"); }"#;
        let mut parser = Parser::new(code).unwrap();
        let program = parser.parse().unwrap();
        let mut codegen = CodeGenerator::new();
        let rust = codegen.generate(&program);
        assert!(rust.contains("#[tokio::main]"), "Generated: {}", rust);
        assert!(rust.contains("async fn main()"), "Generated: {}", rust);
    }
    #[test]
    fn test_await_expression() {
        let code = r#"
Future<int> getData() async => 42;
void main() async {
    var result = await getData();
}
"#;
        let mut parser = Parser::new(code).unwrap();
        let program = parser.parse().unwrap();
        let mut codegen = CodeGenerator::new();
        let rust = codegen.generate(&program);
        assert!(rust.contains(".await"), "Generated: {}", rust);
    }
    #[test]
    fn test_method_chain() {
        let code = r#"
class Builder {
    Builder new() => Builder();
    Builder step1() => this;
}
void main() {
    var b = Builder.new().step1();
}
"#;
        let mut parser = Parser::new(code).unwrap();
        let program = parser.parse().unwrap();
        let mut codegen = CodeGenerator::new();
        let rust = codegen.generate(&program);
        println!("Generated Rust:\n{}", rust);
        assert!(
            rust.contains("Builder.new()"),
            "Missing 'Builder.new()' in: {}",
            rust
        );
        assert!(rust.contains(".step1()"), "Missing '.step1()' in: {}", rust);
        assert!(
            rust.contains("let mut b"),
            "Missing variable declaration in: {}",
            rust
        );
    }
    #[test]
    fn test_httpserver_method_chain() {
        let code = r#"
void main() {
    var server = HttpServer.new()
        .bind("127.0.0.1:8080")
        .workers(4);
}
"#;
        let mut parser = Parser::new(code).unwrap();
        let program = parser.parse().unwrap();
        let mut codegen = CodeGenerator::new();
        let rust = codegen.generate(&program);
        println!("Generated HttpServer Rust:\n{}", rust);
        assert!(
            rust.contains("let mut server"),
            "Missing variable declaration in: {}",
            rust
        );
        assert!(
            rust.contains("HttpServer"),
            "Missing 'HttpServer' in: {}",
            rust
        );
        assert!(rust.contains("bind"), "Missing 'bind' in: {}", rust);
        assert!(rust.contains("workers"), "Missing 'workers' in: {}", rust);
    }
    #[test]
    fn test_class_generation() {
        let code = r#"
class Cookie {
    String _id;
    String _name;
    Cookie(String id, String name) {
        _id = id;
        _name = name;
    }
    set name(String n) {
        _name = n;
    }
    void restock(int amount) { _stock += amount; }
}
"#;
        let mut parser = Parser::new(code).unwrap();
        let program = parser.parse().unwrap();
        println!("AST items count: {}", program.items.len());
        for item in &program.items {
            match item {
                crab_parser::TopLevelItem::ClassDecl(c) => {
                    println!("Class: {} with {} methods", c.name, c.methods.len())
                }
                crab_parser::TopLevelItem::FunctionDecl(f) => println!("Function: {}", f.name),
                _ => println!("Other item"),
            }
        }
        let mut codegen = CodeGenerator::new();
        let rust = codegen.generate(&program);
        println!("Generated Rust:\n{}", rust);
        assert!(
            rust.contains("struct Cookie"),
            "Missing struct definition in: {}",
            rust
        );
        assert!(
            rust.contains("impl Cookie"),
            "Missing impl block in: {}",
            rust
        );
        assert!(rust.contains("set_name"), "Missing setter in: {}", rust);
    }
    #[test]
    fn test_enum_generation() {
        let code = r#"
enum CookieType {
    ChocolateChip,
    OatmealRaisin
}
"#;
        let mut parser = Parser::new(code).unwrap();
        let program = parser.parse().unwrap();
        let mut codegen = CodeGenerator::new();
        let rust = codegen.generate(&program);
        assert!(
            rust.contains("pub enum CookieType"),
            "Missing enum in: {}",
            rust
        );
        assert!(
            rust.contains("ChocolateChip,"),
            "Missing variant in: {}",
            rust
        );
    }
    #[test]
    fn test_null_coalesce_assign_codegen() {
        let code = r#"void main() { var x; x ??= "test"; }"#;
        let mut parser = Parser::new(code).unwrap();
        let program = parser.parse().unwrap();
        println!("AST items: {}", program.items.len());
        if let crab_parser::TopLevelItem::FunctionDecl(func) = &program.items[0] {
            if let crab_parser::FunctionBody::Block(stmts) = &func.body {
                println!("Statements: {}", stmts.len());
                for (i, stmt) in stmts.iter().enumerate() {
                    match stmt {
                        crab_parser::Statement::VarDecl(v) => {
                            println!("Stmt {}: VarDecl({})", i, v.name);
                        }
                        crab_parser::Statement::Expression(expr) => {
                            let expr_ref: &crab_parser::Expr = expr;
                            match expr_ref {
                                crab_parser::Expr::NullCoalesceAssign { target, value } => {
                                    println!("Stmt {}: NullCoalesceAssign", i);
                                    println!("  target: {:?}", target);
                                    println!("  value: {:?}", value);
                                }
                                crab_parser::Expr::Identifier(name) => {
                                    println!("Stmt {}: Expression(Identifier({}))", i, name);
                                }
                                crab_parser::Expr::Literal(lit) => {
                                    println!("Stmt {}: Expression(Literal({:?}))", i, lit);
                                }
                                _ => {
                                    println!("Stmt {}: Expression(Other)", i);
                                }
                            }
                        }
                        _ => {
                            println!("Stmt {}: {:?}", i, std::mem::discriminant(stmt));
                        }
                    }
                }
            }
        }
        let mut codegen = CodeGenerator::new();
        let rust = codegen.generate(&program);
        println!("Generated Rust:\n{}", rust);
        assert!(
            rust.contains("Some(x.unwrap_or") || rust.contains("x = Some(x.unwrap_or"),
            "Expected Some() wrapping for ??=, got: {}",
            rust
        );
    }
}
