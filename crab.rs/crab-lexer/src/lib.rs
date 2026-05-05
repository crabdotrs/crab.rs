pub mod error;
pub mod lexer;
pub mod token;

pub use error::{LexError, LexResult};
pub use lexer::Lexer;
pub use token::Token;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_interpolation_simple() {
        let code = r#""Hello $name""#;
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.len() > 0);
        match &tokens[0] {
            Token::StringLiteral(s) => assert!(s.contains("$name")),
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_string_interpolation_braced() {
        let code = r#""Count: ${items.length}""#;
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.len() > 0);
        match &tokens[0] {
            Token::StringLiteral(s) => assert!(s.contains("${items.length}")),
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_greet_function() {
        let code = r#"String greet() => "Hello, I'm $name";"#;
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.len() >= 6);
    }

    #[test]
    fn test_apostrophe_in_string() {
        let code = r#""Hello, I'm $name";"#;
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.len() >= 2);
    }
    #[test]
    fn test_question_question_equal() {
        use crate::token::Token;
        let code = "x ??= \"test\"";
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize().unwrap();
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::QuestionQuestionEqual))
        );
    }
}
