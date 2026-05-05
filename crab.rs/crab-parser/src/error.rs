use crab_lexer::LexError;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct CrabError {
    pub kind: ErrorKind,
    pub line: usize,
    pub column: usize,
    pub message: String,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    LexError,
    ParseError,
    TypeError,
    NameError,
    CodegenError,
}
impl CrabError {
    pub fn new(kind: ErrorKind, line: usize, column: usize, message: String) -> Self {
        CrabError {
            kind,
            line,
            column,
            message,
        }
    }
    pub fn lex_error(line: usize, column: usize, message: String) -> Self {
        CrabError::new(ErrorKind::LexError, line, column, message)
    }
    pub fn parse_error(line: usize, column: usize, message: String) -> Self {
        CrabError::new(ErrorKind::ParseError, line, column, message)
    }
    pub fn codegen_error(line: usize, column: usize, message: String) -> Self {
        CrabError::new(ErrorKind::CodegenError, line, column, message)
    }
}
impl fmt::Display for CrabError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {:?}: {}",
            self.line, self.column, self.kind, self.message
        )
    }
}
impl Error for CrabError {}

impl From<LexError> for CrabError {
    fn from(err: LexError) -> Self {
        CrabError::new(ErrorKind::LexError, err.line, err.column, err.message)
    }
}

pub type ParseResult<T> = Result<T, CrabError>;
