use crate::error::{LexError, LexResult};
use crate::token::Token;

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> LexResult<Vec<Token>> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            self.skip_whitespace_and_comments();
            if self.is_at_end() {
                break;
            }
            tokens.push(self.next_token()?);
        }
        tokens.push(Token::Eof);
        Ok(tokens)
    }

    pub fn tokenize_with_positions(&mut self) -> LexResult<Vec<(Token, usize)>> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            self.skip_whitespace_and_comments();
            if self.is_at_end() {
                break;
            }
            let pos = self.position;
            let token = self.next_token()?;
            tokens.push((token, pos));
        }
        tokens.push((Token::Eof, self.position));
        Ok(tokens)
    }

    fn next_token(&mut self) -> LexResult<Token> {
        let ch = self.current_char();
        let (line, column) = (self.line, self.column);
        match ch {
            '+' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Ok(Token::PlusEqual)
                } else {
                    Ok(Token::Plus)
                }
            }
            '-' => {
                self.advance();
                if self.current_char() == '>' {
                    self.advance();
                    Ok(Token::Arrow)
                } else if self.current_char() == '=' {
                    self.advance();
                    Ok(Token::MinusEqual)
                } else {
                    Ok(Token::Minus)
                }
            }
            '*' => {
                self.advance();
                if self.current_char() == '*' {
                    self.advance();
                    Ok(Token::DoubleStar)
                } else if self.current_char() == '=' {
                    self.advance();
                    Ok(Token::StarEqual)
                } else {
                    Ok(Token::Star)
                }
            }
            '/' => {
                self.advance();
                if self.current_char() == '/' {
                    self.advance();
                    Ok(Token::DoubleSlash)
                } else if self.current_char() == '=' {
                    self.advance();
                    Ok(Token::SlashEqual)
                } else {
                    Ok(Token::Slash)
                }
            }
            '%' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Ok(Token::PercentEqual)
                } else {
                    Ok(Token::Percent)
                }
            }
            '~' => {
                self.advance();
                Ok(Token::Tilde)
            }
            '&' => {
                self.advance();
                if self.current_char() == '&' {
                    self.advance();
                    Ok(Token::AmpersandAmpersand)
                } else {
                    Ok(Token::Ampersand)
                }
            }
            '|' => {
                self.advance();
                if self.current_char() == '|' {
                    self.advance();
                    Ok(Token::PipePipe)
                } else {
                    Ok(Token::Pipe)
                }
            }
            '^' => {
                self.advance();
                Ok(Token::Caret)
            }
            '=' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Ok(Token::EqualEqual)
                } else if self.current_char() == '>' {
                    self.advance();
                    Ok(Token::DoubleArrow)
                } else {
                    Ok(Token::Equal)
                }
            }
            '!' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Ok(Token::BangEqual)
                } else {
                    Ok(Token::Bang)
                }
            }
            '<' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Ok(Token::LessEqual)
                } else if self.current_char() == '<' {
                    self.advance();
                    Ok(Token::LeftShift)
                } else {
                    Ok(Token::Less)
                }
            }
            '>' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    Ok(Token::GreaterEqual)
                } else if self.current_char() == '>' {
                    self.advance();
                    Ok(Token::RightShift)
                } else {
                    Ok(Token::Greater)
                }
            }
            '?' => {
                self.advance();
                if self.current_char() == '.' {
                    self.advance();
                    Ok(Token::QuestionDot)
                } else if self.current_char() == '?' {
                    self.advance();
                    if self.current_char() == '=' {
                        self.advance();
                        Ok(Token::QuestionQuestionEqual)
                    } else {
                        Ok(Token::QuestionQuestion)
                    }
                } else {
                    Ok(Token::Question)
                }
            }
            '(' => {
                self.advance();
                Ok(Token::LeftParen)
            }
            ')' => {
                self.advance();
                Ok(Token::RightParen)
            }
            '{' => {
                self.advance();
                Ok(Token::LeftBrace)
            }
            '}' => {
                self.advance();
                Ok(Token::RightBrace)
            }
            '[' => {
                self.advance();
                Ok(Token::LeftBracket)
            }
            ']' => {
                self.advance();
                Ok(Token::RightBracket)
            }
            ';' => {
                self.advance();
                Ok(Token::Semicolon)
            }
            ',' => {
                self.advance();
                Ok(Token::Comma)
            }
            '.' => {
                self.advance();
                if self.current_char() == '.' && self.peek_char() == '.' {
                    self.advance();
                    self.advance();
                    Ok(Token::Ellipsis)
                } else {
                    Ok(Token::Dot)
                }
            }
            ':' => {
                self.advance();
                Ok(Token::Colon)
            }
            '@' => {
                self.advance();
                Ok(Token::At)
            }
            '"' => self.read_string('"'),
            '\'' => self.read_string('\''),
            '0'..='9' => self.read_number(),
            'a'..='z' | 'A'..='Z' => self.read_identifier(),
            '#' => {
                self.advance();
                Ok(Token::Hash)
            }
            '_' => {
                self.advance();
                if self.current_char().is_alphanumeric() || self.current_char() == '_' {
                    self.read_identifier_after_underscore()
                } else {
                    Ok(Token::Underscore)
                }
            }
            _ => Err(LexError::new(
                line,
                column,
                format!("Unexpected character: '{}'", ch),
            )),
        }
    }

    fn read_string(&mut self, delimiter: char) -> LexResult<Token> {
        self.advance();
        let mut value = String::new();
        while !self.is_at_end() && self.current_char() != delimiter {
            if self.current_char() == '\\' {
                self.advance();
                if !self.is_at_end() {
                    value.push(match self.current_char() {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        '\'' => '\'',
                        '$' => '$',
                        _ => self.current_char(),
                    });
                    self.advance();
                }
            } else if self.current_char() == '$' {
                value.push('$');
                self.advance();
                if self.current_char() == '{' {
                    value.push('{');
                    self.advance();
                    while !self.is_at_end() && self.current_char() != '}' {
                        value.push(self.current_char());
                        self.advance();
                    }
                    if self.current_char() == '}' {
                        value.push('}');
                        self.advance();
                    }
                }
            } else {
                value.push(self.current_char());
                self.advance();
            }
        }
        if self.is_at_end() {
            return Err(LexError::new(self.line, self.column, "Unterminated string"));
        }
        self.advance();
        Ok(Token::StringLiteral(value))
    }

    fn read_number(&mut self) -> LexResult<Token> {
        let mut number = String::new();
        let mut is_double = false;
        while !self.is_at_end() && (self.current_char().is_numeric() || self.current_char() == '.')
        {
            if self.current_char() == '.' {
                if is_double {
                    break;
                }
                is_double = true;
            }
            number.push(self.current_char());
            self.advance();
        }
        if is_double {
            Ok(Token::DoubleLiteral(
                number.parse().expect("Invalid double literal"),
            ))
        } else {
            Ok(Token::IntLiteral(
                number.parse().expect("Invalid integer literal"),
            ))
        }
    }

    fn read_identifier(&mut self) -> LexResult<Token> {
        let mut ident = String::new();
        while !self.is_at_end()
            && (self.current_char().is_alphanumeric() || self.current_char() == '_')
        {
            ident.push(self.current_char());
            self.advance();
        }
        self.tokenize_identifier(ident)
    }

    fn read_identifier_after_underscore(&mut self) -> LexResult<Token> {
        let mut ident = String::from('_');
        while !self.is_at_end()
            && (self.current_char().is_alphanumeric() || self.current_char() == '_')
        {
            ident.push(self.current_char());
            self.advance();
        }
        self.tokenize_identifier(ident)
    }

    fn tokenize_identifier(&self, ident: String) -> LexResult<Token> {
        let token = match ident.as_str() {
            "var" => Token::Var,
            "final" => Token::Final,
            "const" => Token::Const,
            "int" => Token::Int,
            "double" => Token::Double,
            "bool" => Token::Bool,
            "String" => Token::String,
            "void" => Token::Void,
            "class" => Token::Class,
            "interface" => Token::Interface,
            "abstract" => Token::Abstract,
            "sealed" => Token::Sealed,
            "base" => Token::Base,
            "extends" => Token::Extends,
            "if" => Token::If,
            "else" => Token::Else,
            "for" => Token::For,
            "while" => Token::While,
            "do" => Token::Do,
            "switch" => Token::Switch,
            "case" => Token::Case,
            "default" => Token::Default,
            "return" => Token::Return,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "try" => Token::Try,
            "catch" => Token::Catch,
            "finally" => Token::Finally,
            "throw" => Token::Throw,
            "fn" => Token::Function,
            "async" => Token::Async,
            "await" => Token::Await,
            "yield" => Token::Yield,
            "is" => Token::Is,
            "as" => Token::As,
            "new" => Token::New,
            "super" => Token::Super,
            "this" => Token::This,
            "static" => Token::Static,
            "get" => Token::Get,
            "set" => Token::Set,
            "typedef" => Token::Typedef,
            "true" => Token::BoolLiteral(true),
            "false" => Token::BoolLiteral(false),
            "null" => Token::NullLiteral,
            "required" => Token::Required,
            "factory" => Token::Factory,
            "override" => Token::Override,
            "in" => Token::In,
            "import" => Token::Import,
            "export" => Token::Export,
            "cblock" | "CBlock" => Token::CBlock,
            "unsafe" => Token::Unsafe,
            "mixin" => Token::Mixin,
            "extension" => Token::Extension,
            "enum" => Token::Enum,
            "on" => Token::On,
            "with" => Token::With,
            "part" => Token::Part,
            "library" => Token::Library,
            "show" => Token::Show,
            "hide" => Token::Hide,
            "deferred" => Token::Deferred,
            "implements" => Token::Implements,
            _ => Token::Identifier(ident),
        };
        Ok(token)
    }

    fn skip_whitespace_and_comments(&mut self) {
        while !self.is_at_end() {
            match self.current_char() {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                    self.position += 1;
                }
                '/' if self.peek_char() == '/' => {
                    while !self.is_at_end() && self.current_char() != '\n' {
                        self.advance();
                    }
                }
                '/' if self.peek_char() == '*' => {
                    self.advance();
                    self.advance();
                    while !self.is_at_end() {
                        if self.current_char() == '*' && self.peek_char() == '/' {
                            self.advance();
                            self.advance();
                            break;
                        }
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn current_char(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.position]
        }
    }

    fn peek_char(&self) -> char {
        if self.position + 1 >= self.input.len() {
            '\0'
        } else {
            self.input[self.position + 1]
        }
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            if self.current_char() == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}
