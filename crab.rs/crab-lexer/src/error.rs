use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lex error at {}:{}: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for LexError {}

impl LexError {
    pub fn new(line: usize, column: usize, message: impl Into<String>) -> Self {
        LexError {
            line,
            column,
            message: message.into(),
        }
    }
}

pub type LexResult<T> = Result<T, LexError>;
