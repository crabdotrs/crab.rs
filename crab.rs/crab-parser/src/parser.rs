use crate::ast::*;
use crate::error::{CrabError, ParseResult};
use crab_lexer::{Lexer, Token};
pub struct Parser {
    tokens: Vec<Token>,
    positions: Vec<usize>,
    current: usize,
    input: String,
}
impl Parser {
    pub fn new(input: &str) -> ParseResult<Self> {
        let mut lexer = Lexer::new(input);
        let tokens_with_pos = lexer.tokenize_with_positions()?;
        let (tokens, positions): (Vec<_>, Vec<_>) = tokens_with_pos.into_iter().unzip();
        Ok(Parser {
            tokens,
            positions,
            current: 0,
            input: input.to_string(),
        })
    }
    fn current_token_start_pos(&self) -> usize {
        self.positions
            .get(self.current)
            .copied()
            .unwrap_or(self.input.len())
    }
    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut items = Vec::new();
        while !self.is_at_end() {
            match self.parse_top_level_item() {
                Ok(item) => {
                    items.push(item);
                }
                Err(_e) => {
                    self.advance();
                }
            }
        }
        Ok(Program { items })
    }
    fn parse_top_level_item(&mut self) -> ParseResult<TopLevelItem> {
        match self.peek() {
            Token::Import => {
                self.advance();
                let path = match self.peek() {
                    Token::StringLiteral(s) => {
                        let s = s.clone();
                        self.advance();
                        s
                    }
                    _ => self.expect_identifier()?,
                };
                self.expect_optional(&Token::Semicolon);
                Ok(TopLevelItem::Import(ImportStmt { path }))
            }
            Token::Export => {
                self.advance();
                let path = match self.peek() {
                    Token::StringLiteral(s) => {
                        let s = s.clone();
                        self.advance();
                        s
                    }
                    _ => self.expect_identifier()?,
                };
                self.expect_optional(&Token::Semicolon);
                Ok(TopLevelItem::Export(ExportStmt { path }))
            }
            Token::CBlock => {
                self.advance();
                self.expect(&Token::LeftBrace)?;
                let start_pos = self.current_token_start_pos();
                let mut brace_count = 1;
                let mut end_pos = start_pos;
                while brace_count > 0 && !self.is_at_end() {
                    match self.peek() {
                        Token::LeftBrace => {
                            brace_count += 1;
                            self.advance();
                        }
                        Token::RightBrace => {
                            brace_count -= 1;
                            end_pos = self.current_token_start_pos();
                            self.advance();
                        }
                        _ => {
                            self.advance();
                        }
                    }
                }
                let c_code = self.input[start_pos..end_pos].to_string();
                Ok(TopLevelItem::CBlock(CBlockDecl { code: c_code }))
            }
            Token::Var | Token::Const => {
                let _is_const = matches!(self.peek(), Token::Const);
                self.advance();
                let name = self.expect_identifier()?;
                let _typ = if self.match_token(&Token::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.expect(&Token::Equal)?;
                let value = self.parse_expr()?;
                self.expect_optional(&Token::Semicolon);
                Ok(TopLevelItem::Const(ConstDecl {
                    name,
                    typ: None,
                    value,
                }))
            }
            Token::Final => {
                let checkpoint = self.current;
                self.advance();
                if self.check(&Token::Class) {
                    self.current = checkpoint;
                    Ok(TopLevelItem::ClassDecl(self.parse_class()?))
                } else {
                    let typ = self.parse_type()?;
                    let name = self.expect_identifier()?;
                    self.expect(&Token::Equal)?;
                    let value = self.parse_expr()?;
                    self.expect_optional(&Token::Semicolon);
                    Ok(TopLevelItem::VarDecl(TopLevelVarDecl {
                        name,
                        typ,
                        value,
                        is_final: true,
                    }))
                }
            }
            Token::Class | Token::Interface | Token::Abstract | Token::Sealed | Token::Base => {
                Ok(TopLevelItem::ClassDecl(self.parse_class()?))
            }
            Token::Mixin => Ok(TopLevelItem::MixinDecl(self.parse_mixin()?)),
            Token::Extension => Ok(TopLevelItem::ExtensionDecl(self.parse_extension()?)),
            Token::Enum => Ok(TopLevelItem::ClassDecl(self.parse_enum()?)),
            Token::Async => {
                let func = self.parse_function()?;
                Ok(TopLevelItem::FunctionDecl(func))
            }
            _ => {
                if self.check(&Token::Class)
                    || self.check(&Token::Abstract)
                    || self.check(&Token::Sealed)
                    || self.check(&Token::Interface)
                    || self.check(&Token::Base)
                {
                    return Ok(TopLevelItem::ClassDecl(self.parse_class()?));
                }
                let checkpoint = self.current;
                match self.parse_type() {
                    Ok(typ) => {
                        if let Ok(name) = self.expect_identifier() {
                            if self.check(&Token::Equal) {
                                self.advance();
                                let value = self.parse_expr()?;
                                self.expect_optional(&Token::Semicolon);
                                return Ok(TopLevelItem::VarDecl(TopLevelVarDecl {
                                    name,
                                    typ,
                                    value,
                                    is_final: false,
                                }));
                            }
                        }
                    }
                    Err(_) => {
                        self.current = checkpoint;
                    }
                }
                self.current = checkpoint;
                let func = self.parse_function()?;
                Ok(TopLevelItem::FunctionDecl(func))
            }
        }
    }
    fn parse_mixin(&mut self) -> ParseResult<MixinDecl> {
        self.expect(&Token::Mixin)?;
        let name = self.expect_identifier()?;
        let on_type = if self.match_token(&Token::On) {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(&Token::LeftBrace)?;
        let mut methods = Vec::new();
        let mut fields = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.check(&Token::Get) || self.check(&Token::Set) {
                let is_getter = self.match_token(&Token::Get);
                let is_setter = !is_getter && self.match_token(&Token::Set);
                let method = self.parse_simple_method(false, is_getter, is_setter)?;
                methods.push(method);
            } else {
                let checkpoint = self.current;
                if let Ok(typ) = self.parse_type() {
                    let fname = self.expect_identifier()?;
                    if fname == "get" && matches!(self.peek(), Token::Identifier(_)) {
                        let getter_name = self.expect_identifier()?;
                        self.expect(&Token::DoubleArrow)?;
                        let expr = self.parse_expr()?;
                        self.expect_optional(&Token::Semicolon);
                        methods.push(MethodDecl {
                            name: getter_name,
                            params: Vec::new(),
                            return_type: Some(typ),
                            body: FunctionBody::Expression(Box::new(expr)),
                            is_static: false,
                            is_getter: true,
                            is_setter: false,
                            is_override: false,
                        });
                    } else if self.check(&Token::LeftParen) {
                        self.current = checkpoint;
                        let method = self.parse_simple_method(false, false, false)?;
                        methods.push(method);
                    } else {
                        let initializer = if self.match_token(&Token::Equal) {
                            Some(Box::new(self.parse_expr()?))
                        } else {
                            None
                        };
                        self.expect_optional(&Token::Semicolon);
                        fields.push(Field {
                            name: fname,
                            typ,
                            is_final: false,
                            is_static: false,
                            is_private: false,
                            initializer,
                        });
                    }
                } else {
                    break;
                }
            }
        }
        self.expect(&Token::RightBrace)?;
        Ok(MixinDecl {
            name,
            on_type,
            methods,
            fields,
        })
    }
    fn parse_extension(&mut self) -> ParseResult<ExtensionDecl> {
        self.expect(&Token::Extension)?;
        let name = if self.check(&Token::On) {
            None
        } else {
            Some(self.expect_identifier()?)
        };
        self.expect(&Token::On)?;
        let on_type = self.parse_type()?;
        self.expect(&Token::LeftBrace)?;
        let mut methods = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.check(&Token::Get) || self.check(&Token::Set) {
                let is_getter = self.match_token(&Token::Get);
                let is_setter = !is_getter && self.match_token(&Token::Set);
                let method = self.parse_simple_method(false, is_getter, is_setter)?;
                methods.push(method);
            } else {
                let method = self.parse_simple_method(false, false, false)?;
                methods.push(method);
            }
        }
        self.expect(&Token::RightBrace)?;
        Ok(ExtensionDecl {
            name,
            on_type,
            methods,
        })
    }
    fn parse_enum(&mut self) -> ParseResult<ClassDecl> {
        self.expect(&Token::Enum)?;
        let name = self.expect_identifier()?;
        self.expect(&Token::LeftBrace)?;
        let mut fields = Vec::new();
        let mut constructors = Vec::new();
        let mut index = 0i64;
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let variant_name = self.expect_identifier()?;
            if self.check(&Token::LeftParen) {
                self.advance();
                let _value = self.parse_expr()?;
                self.expect(&Token::RightParen)?;
                fields.push(Field {
                    name: format!("_{}", variant_name.to_lowercase()),
                    typ: Type::Int,
                    is_final: true,
                    is_static: false,
                    is_private: true,
                    initializer: None,
                });
            }
            constructors.push(Constructor {
                name: Some(variant_name),
                params: Vec::new(),
                initializers: vec![Initializer {
                    field: "index".to_string(),
                    value: Expr::Literal(Literal::Int(index)),
                }],
                body: Vec::new(),
                has_super_delegation: false,
                super_args: Vec::new(),
            });
            index += 1;
            if self.check(&Token::Comma) {
                self.advance();
            }
        }
        self.expect(&Token::RightBrace)?;
        fields.insert(
            0,
            Field {
                name: "index".to_string(),
                typ: Type::Int,
                is_final: true,
                is_static: false,
                is_private: false,
                initializer: None,
            },
        );
        Ok(ClassDecl {
            name,
            parent: None,
            mixins: Vec::new(),
            implements: Vec::new(),
            fields,
            constructors,
            methods: Vec::new(),
            is_sealed: false,
            is_final: true,
            is_abstract: false,
            is_interface: false,
        })
    }
    fn parse_class(&mut self) -> ParseResult<ClassDecl> {
        let _is_sealed = self.match_token(&Token::Sealed);
        let _is_abstract = self.match_token(&Token::Abstract);
        let _is_interface = self.match_token(&Token::Interface);
        let _is_base = self.match_token(&Token::Base);
        let _is_final = self.match_token(&Token::Final);
        self.expect(&Token::Class)?;
        let base_name = self.expect_identifier()?;
        let (name, base_name_for_ctor) = if self.check(&Token::Less) {
            let generic_params = self.parse_generic_params()?;
            let full_name = format!("{}<{}>", base_name, generic_params.join(", "));
            (full_name, base_name)
        } else {
            (base_name.clone(), base_name)
        };
        let parent = if self.match_token(&Token::Extends) {
            let p = self.parse_type()?;
            Some(Box::new(p))
        } else {
            None
        };
        let mut implements = Vec::new();
        if self.match_token(&Token::Implements) {
            loop {
                implements.push(self.parse_type()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        let mut mixins = Vec::new();
        if self.match_token(&Token::With) {
            loop {
                mixins.push(self.parse_type()?);
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        self.expect(&Token::LeftBrace)?;
        let mut fields = Vec::new();
        let mut constructors: Vec<Constructor> = Vec::new();
        let mut methods = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            while self.check(&Token::At) {
                self.advance();
                let _annotation_name = match self.peek() {
                    Token::Identifier(name) => {
                        let n = name.clone();
                        self.advance();
                        n
                    }
                    Token::Override => {
                        self.advance();
                        "override".to_string()
                    }
                    _ => {
                        self.advance();
                        "unknown".to_string()
                    }
                };
                if self.check(&Token::LeftParen) {
                    self.advance();
                    let mut paren_count = 1;
                    while paren_count > 0 && !self.is_at_end() {
                        match self.peek() {
                            Token::LeftParen => paren_count += 1,
                            Token::RightParen => paren_count -= 1,
                            _ => {}
                        }
                        self.advance();
                    }
                }
            }
            if self.check(&Token::RightBrace) || self.is_at_end() {
                break;
            }
            let _is_static = self.match_token(&Token::Static);
            let is_final = self.match_token(&Token::Final);
            if self.check(&Token::Get) {
                self.advance();
                let method = self.parse_simple_method(false, true, false)?;
                methods.push(method);
            } else if self.check(&Token::Set) {
                self.advance();
                let method = self.parse_simple_method(false, false, true)?;
                methods.push(method);
            } else {
                let _checkpoint = self.current;
                let possible_type = self.parse_type()?;
                if self.check(&Token::LeftParen) {
                    let type_name = match &possible_type {
                        Type::Custom(n) => n.clone(),
                        _ => String::new(),
                    };
                    if type_name == base_name_for_ctor {
                        self.advance();
                        let ctor_params = self.parse_constructor_params()?;
                        self.expect(&Token::RightParen)?;
                        let (has_super_delegation, super_args) = if self.check(&Token::Colon) {
                            self.advance();
                            match self.peek() {
                                Token::Super => self.advance(),
                                Token::Identifier(name) if name == "super" => self.advance(),
                                _ => {
                                    return Err(CrabError::parse_error(
                                        0,
                                        0,
                                        "Expected 'super' in constructor delegation".to_string(),
                                    ));
                                }
                            };
                            self.expect(&Token::LeftParen)?;
                            let mut args = Vec::new();
                            while !self.check(&Token::RightParen) && !self.is_at_end() {
                                args.push(self.parse_expr()?);
                                if self.check(&Token::Comma) {
                                    self.advance();
                                }
                            }
                            self.expect(&Token::RightParen)?;
                            (true, args)
                        } else {
                            (false, Vec::new())
                        };
                        let body = if self.match_token(&Token::DoubleArrow) {
                            let _expr = self.parse_expr()?;
                            self.expect_optional(&Token::Semicolon);
                            Vec::new()
                        } else if self.check(&Token::LeftBrace) {
                            self.advance();
                            let stmts = self.parse_block()?;
                            self.expect(&Token::RightBrace)?;
                            stmts
                        } else {
                            self.expect_optional(&Token::Semicolon);
                            Vec::new()
                        };
                        let mut initializers: Vec<Initializer> = ctor_params
                            .iter()
                            .filter(|p| p.name.starts_with("this."))
                            .map(|p| {
                                let field_name = p.name[5..].to_string();
                                Initializer {
                                    field: field_name.clone(),
                                    value: Expr::Identifier(field_name.clone()),
                                }
                            })
                            .collect();
                        for stmt in &body {
                            if let Statement::Expression(Expr::Assign { target, value }) = stmt {
                                if let Expr::Identifier(field_name) = &**target {
                                    initializers.push(Initializer {
                                        field: field_name.clone(),
                                        value: (**value).clone(),
                                    });
                                }
                            }
                        }
                        constructors.push(Constructor {
                            name: None,
                            params: ctor_params,
                            initializers,
                            body,
                            has_super_delegation,
                            super_args,
                        });
                        continue;
                    }
                }
                let typ = possible_type;
                let fname = match self.peek() {
                    Token::Identifier(name) => {
                        let n = name.clone();
                        self.advance();
                        n
                    }
                    Token::Get => {
                        self.advance();
                        "get".to_string()
                    }
                    Token::Set => {
                        self.advance();
                        "set".to_string()
                    }
                    Token::New => {
                        self.advance();
                        "new".to_string()
                    }
                    Token::Override => {
                        self.advance();
                        "override".to_string()
                    }
                    _ => {
                        return Err(CrabError::parse_error(
                            0,
                            0,
                            format!(
                                "Expected identifier, 'get', 'set', or 'new', got {:?}",
                                self.peek()
                            ),
                        ));
                    }
                };
                if fname == "get" && matches!(self.peek(), Token::Identifier(_)) {
                    let getter_name = self.expect_identifier()?;
                    let body = if self.match_token(&Token::DoubleArrow) {
                        let expr = FunctionBody::Expression(Box::new(self.parse_expr()?));
                        self.expect_optional(&Token::Semicolon);
                        expr
                    } else if self.match_token(&Token::Semicolon) {
                        FunctionBody::Block(Vec::new())
                    } else {
                        self.expect(&Token::LeftBrace)?;
                        let stmts = self.parse_block()?;
                        self.expect(&Token::RightBrace)?;
                        FunctionBody::Block(stmts)
                    };
                    methods.push(MethodDecl {
                        name: getter_name,
                        params: Vec::new(),
                        return_type: Some(typ),
                        body,
                        is_static: false,
                        is_getter: true,
                        is_setter: false,
                        is_override: false,
                    });
                } else if self.check(&Token::LeftParen) {
                    self.advance();
                    let params = self.parse_parameters()?;
                    self.expect(&Token::RightParen)?;
                    let body = if self.match_token(&Token::DoubleArrow) {
                        let expr = FunctionBody::Expression(Box::new(self.parse_expr()?));
                        self.expect_optional(&Token::Semicolon);
                        expr
                    } else if self.check(&Token::Semicolon) {
                        // Abstract method - no body
                        self.advance();
                        FunctionBody::Block(Vec::new())
                    } else {
                        self.expect(&Token::LeftBrace)?;
                        let stmts = self.parse_block()?;
                        self.expect(&Token::RightBrace)?;
                        FunctionBody::Block(stmts)
                    };
                    methods.push(MethodDecl {
                        name: fname.clone(),
                        params,
                        return_type: Some(typ),
                        body,
                        is_static: false,
                        is_getter: false,
                        is_setter: false,
                        is_override: false,
                    });
                } else {
                    let initializer = if self.match_token(&Token::Equal) {
                        Some(Box::new(self.parse_expr()?))
                    } else {
                        None
                    };
                    self.expect_optional(&Token::Semicolon);
                    let is_private = fname.starts_with('_');
                    fields.push(Field {
                        name: fname.clone(),
                        typ,
                        is_final,
                        is_static: false,
                        is_private,
                        initializer,
                    });
                }
            }
        }
        self.expect(&Token::RightBrace)?;
        Ok(ClassDecl {
            name,
            parent,
            mixins,
            implements,
            fields,
            constructors,
            methods,
            is_sealed: _is_sealed,
            is_final: _is_final,
            is_abstract: _is_abstract,
            is_interface: _is_interface,
        })
    }
    fn parse_function(&mut self) -> ParseResult<FunctionDecl> {
        let return_type = if self.check(&Token::Void) || self.check_type_start() {
            Some(self.parse_type()?)
        } else {
            None
        };
        let name = self.expect_identifier()?;
        self.expect(&Token::LeftParen)?;
        let params = self.parse_parameters()?;
        self.expect(&Token::RightParen)?;
        let is_async = self.match_token(&Token::Async);
        let is_generator = if is_async {
            self.match_token(&Token::Sync)
        } else {
            self.match_token(&Token::Sync)
        };
        let body = if self.match_token(&Token::DoubleArrow) {
            let expr = FunctionBody::Expression(Box::new(self.parse_expr()?));
            self.expect_optional(&Token::Semicolon);
            expr
        } else {
            self.expect(&Token::LeftBrace)?;
            let stmts = self.parse_block()?;
            self.expect(&Token::RightBrace)?;
            FunctionBody::Block(stmts)
        };
        Ok(FunctionDecl {
            name,
            params,
            return_type,
            body,
            is_async,
            is_generator,
        })
    }
    fn parse_parameters(&mut self) -> ParseResult<Vec<Parameter>> {
        let mut params = Vec::new();
        while !self.check(&Token::RightParen) && !self.is_at_end() {
            let typ = self.parse_type()?;
            let name = self.expect_identifier()?;
            let (_is_required, default_value) = if self.match_token(&Token::Equal) {
                (false, Some(Box::new(self.parse_expr()?)))
            } else {
                (true, None)
            };
            params.push(Parameter {
                name,
                typ,
                is_required: true,
                default_value,
            });
            if !self.check(&Token::RightParen) {
                self.expect_optional(&Token::Comma);
            }
        }
        Ok(params)
    }
    fn parse_constructor_params(&mut self) -> ParseResult<Vec<Parameter>> {
        let mut params = Vec::new();
        while !self.check(&Token::RightParen) && !self.is_at_end() {
            if self.check(&Token::This) {
                self.advance();
                self.expect(&Token::Dot)?;
                let field_name = self.expect_identifier()?;
                params.push(Parameter {
                    name: format!("this.{}", field_name),
                    typ: Type::Dynamic,
                    is_required: true,
                    default_value: None,
                });
            } else {
                let typ = self.parse_type()?;
                let name = self.expect_identifier()?;
                params.push(Parameter {
                    name,
                    typ,
                    is_required: true,
                    default_value: None,
                });
            }
            if self.check(&Token::Comma) {
                self.advance();
            }
        }
        Ok(params)
    }
    fn parse_type(&mut self) -> ParseResult<Type> {
        let base_type = match self.peek() {
            Token::Int => {
                self.advance();
                Type::Int
            }
            Token::Double => {
                self.advance();
                Type::Double
            }
            Token::Bool => {
                self.advance();
                Type::Bool
            }
            Token::String => {
                self.advance();
                Type::String
            }
            Token::Void => {
                self.advance();
                Type::Void
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Type::Custom(name)
            }
            _ => {
                return Err(CrabError::parse_error(0, 0, "Expected type".to_string()));
            }
        };
        let typ = if self.check(&Token::Less) {
            let checkpoint = self.current;
            self.advance();
            let mut params = Vec::new();
            loop {
                if self.check(&Token::Greater) || self.is_at_end() {
                    break;
                }
                match self.parse_type() {
                    Ok(param_type) => {
                        params.push(param_type);
                        if self.check(&Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    Err(_) => {
                        self.current = checkpoint;
                        return Ok(base_type);
                    }
                }
            }
            if self.check(&Token::Greater) {
                self.advance();
                let base_name = match &base_type {
                    Type::Custom(name) => name.clone(),
                    _ => return Ok(base_type),
                };
                Type::Generic(base_name, params)
            } else {
                self.current = checkpoint;
                base_type
            }
        } else if self.match_token(&Token::Question) {
            Type::Nullable(Box::new(base_type))
        } else {
            base_type
        };
        Ok(typ)
    }
    fn parse_generic_params(&mut self) -> ParseResult<Vec<String>> {
        let mut params = Vec::new();
        self.expect(&Token::Less)?;
        loop {
            let param = self.expect_identifier()?;
            params.push(param);
            if self.check(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&Token::Greater)?;
        Ok(params)
    }
    fn parse_block(&mut self) -> ParseResult<Vec<Statement>> {
        let mut statements = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if let Ok(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                self.advance();
            }
        }
        Ok(statements)
    }
    fn parse_statement(&mut self) -> ParseResult<Statement> {
        let peek = self.peek();
        match peek {
            Token::Var => {
                self.advance();
                let name = self.expect_identifier()?;
                let typ = if self.match_token(&Token::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                let value = if self.match_token(&Token::Equal) {
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                self.expect_optional(&Token::Semicolon);
                Ok(Statement::VarDecl(VarDecl { name, typ, value }))
            }
            Token::Final => {
                self.advance();
                let name = self.expect_identifier()?;
                let typ = if self.match_token(&Token::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                let value = if self.match_token(&Token::Equal) {
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                self.expect_optional(&Token::Semicolon);
                Ok(Statement::FinalDecl(VarDecl { name, typ, value }))
            }
            Token::Const => {
                self.advance();
                let (name, typ) = if self.peek_type_keyword() {
                    let t = self.parse_type()?;
                    let n = self.expect_identifier()?;
                    (n, Some(t))
                } else {
                    let n = self.expect_identifier()?;
                    let t = if self.match_token(&Token::Colon) {
                        Some(self.parse_type()?)
                    } else {
                        None
                    };
                    (n, t)
                };
                self.expect(&Token::Equal)?;
                let value = self.parse_expr()?;
                self.expect_optional(&Token::Semicolon);
                Ok(Statement::ConstDecl(ConstDecl { name, typ, value }))
            }
            Token::Int | Token::Double | Token::Bool | Token::String | Token::Void => {
                let typ = self.parse_type()?;
                let name = self.expect_identifier()?;
                let value = if self.match_token(&Token::Equal) {
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                self.expect_optional(&Token::Semicolon);
                Ok(Statement::VarDecl(VarDecl {
                    name,
                    typ: Some(typ),
                    value,
                }))
            }
            Token::Identifier(_) => {
                let checkpoint = self.current;
                if let Token::Identifier(name) = self.peek() {
                    let label_name = name.clone();
                    self.advance();
                    if self.check(&Token::Colon) {
                        self.advance();
                        match self.peek() {
                            Token::While => {
                                let mut while_stmt = self.parse_while_statement()?;
                                if let Statement::While(ref mut ws) = while_stmt {
                                    ws.label = Some(label_name);
                                }
                                return Ok(while_stmt);
                            }
                            Token::Do => {
                                let mut do_while_stmt = self.parse_do_while_statement()?;
                                if let Statement::DoWhile(ref mut dws) = do_while_stmt {
                                    dws.label = Some(label_name);
                                }
                                return Ok(do_while_stmt);
                            }
                            Token::For => {
                                let mut for_stmt = self.parse_for_statement()?;
                                if let Statement::For(ref mut fs) = for_stmt {
                                    fs.label = Some(label_name);
                                }
                                return Ok(for_stmt);
                            }
                            _ => {
                                self.current = checkpoint;
                            }
                        }
                    } else {
                        self.current = checkpoint;
                    }
                }
                if let Ok(typ) = self.parse_type() {
                    if self.check(&Token::Identifier("".to_string()))
                        || matches!(self.peek(), Token::Identifier(_))
                    {
                        let name = self.expect_identifier()?;
                        let value = if self.match_token(&Token::Equal) {
                            Some(Box::new(self.parse_expr()?))
                        } else {
                            None
                        };
                        self.expect_optional(&Token::Semicolon);
                        return Ok(Statement::VarDecl(VarDecl {
                            name,
                            typ: Some(typ),
                            value,
                        }));
                    }
                }
                self.current = checkpoint;
                let expr = self.parse_expr()?;
                self.expect_optional(&Token::Semicolon);
                Ok(Statement::Expression(expr))
            }
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_statement(),
            Token::Do => self.parse_do_while_statement(),
            Token::For => self.parse_for_statement(),
            Token::Switch => self.parse_switch_statement(),
            Token::Try => self.parse_try_statement(),
            Token::Throw => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect_optional(&Token::Semicolon);
                Ok(Statement::ThrowStmt(expr))
            }
            Token::Return => {
                self.advance();
                let value = if self.check(&Token::Semicolon) {
                    None
                } else {
                    Some(Box::new(self.parse_expr()?))
                };
                self.expect_optional(&Token::Semicolon);
                Ok(Statement::Return(value))
            }
            Token::Break => {
                self.advance();
                let label = if matches!(self.peek(), Token::Identifier(_)) {
                    if let Token::Identifier(name) = self.peek() {
                        let label_name = name.clone();
                        self.advance();
                        Some(label_name)
                    } else {
                        None
                    }
                } else {
                    None
                };
                self.expect_optional(&Token::Semicolon);
                Ok(Statement::Break(label))
            }
            Token::Continue => {
                self.advance();
                let label = if matches!(self.peek(), Token::Identifier(_)) {
                    if let Token::Identifier(name) = self.peek() {
                        let label_name = name.clone();
                        self.advance();
                        Some(label_name)
                    } else {
                        None
                    }
                } else {
                    None
                };
                self.expect_optional(&Token::Semicolon);
                Ok(Statement::Continue(label))
            }
            Token::LeftBrace => {
                self.advance();
                let stmts = self.parse_block()?;
                self.expect(&Token::RightBrace)?;
                Ok(Statement::Block(stmts))
            }
            _ => {
                let expr = self.parse_expr()?;
                self.expect_optional(&Token::Semicolon);
                Ok(Statement::Expression(expr))
            }
        }
    }
    fn parse_if_statement(&mut self) -> ParseResult<Statement> {
        self.expect(&Token::If)?;
        self.expect(&Token::LeftParen)?;
        let condition = Box::new(self.parse_expr()?);
        self.expect(&Token::RightParen)?;
        let then_block = if self.check(&Token::LeftBrace) {
            self.advance();
            let stmts = self.parse_block()?;
            self.expect(&Token::RightBrace)?;
            stmts
        } else {
            vec![self.parse_statement()?]
        };
        let else_block = if self.match_token(&Token::Else) {
            if self.check(&Token::LeftBrace) {
                self.advance();
                let stmts = self.parse_block()?;
                self.expect(&Token::RightBrace)?;
                Some(stmts)
            } else {
                Some(vec![self.parse_statement()?])
            }
        } else {
            None
        };
        Ok(Statement::If(IfStmt {
            condition,
            then_block,
            else_if_blocks: Vec::new(),
            else_block,
        }))
    }
    fn parse_while_statement(&mut self) -> ParseResult<Statement> {
        self.expect(&Token::While)?;
        self.expect(&Token::LeftParen)?;
        let condition = Box::new(self.parse_expr()?);
        self.expect(&Token::RightParen)?;
        let body = if self.check(&Token::LeftBrace) {
            self.advance();
            let stmts = self.parse_block()?;
            self.expect(&Token::RightBrace)?;
            stmts
        } else {
            vec![self.parse_statement()?]
        };
        Ok(Statement::While(WhileStmt {
            label: None,
            condition,
            body,
        }))
    }
    fn parse_do_while_statement(&mut self) -> ParseResult<Statement> {
        self.expect(&Token::Do)?;
        let body = if self.check(&Token::LeftBrace) {
            self.advance();
            let stmts = self.parse_block()?;
            self.expect(&Token::RightBrace)?;
            stmts
        } else {
            vec![self.parse_statement()?]
        };
        self.expect(&Token::While)?;
        self.expect(&Token::LeftParen)?;
        let condition = Box::new(self.parse_expr()?);
        self.expect(&Token::RightParen)?;
        self.expect_optional(&Token::Semicolon);
        Ok(Statement::DoWhile(DoWhileStmt {
            label: None,
            body,
            condition,
        }))
    }
    fn parse_for_statement(&mut self) -> ParseResult<Statement> {
        self.expect(&Token::For)?;
        self.expect(&Token::LeftParen)?;
        let checkpoint = self.current;
        let is_for_in = self.peek_type_keyword() || self.check(&Token::Var);
        if is_for_in {
            let _checkpoint = self.current;
            if self.check(&Token::Var) {
                self.advance();
            } else if self.peek_type_keyword() {
                let _ = self.parse_type();
            }
            if let Ok(var_name) = self.expect_identifier() {
                if self.check(&Token::In) {
                    self.advance();
                    let iterable = Box::new(self.parse_expr()?);
                    self.expect(&Token::RightParen)?;
                    let body = if self.check(&Token::LeftBrace) {
                        self.advance();
                        let stmts = self.parse_block()?;
                        self.expect(&Token::RightBrace)?;
                        stmts
                    } else {
                        vec![self.parse_statement()?]
                    };
                    return Ok(Statement::ForIn(ForInStmt {
                        label: None,
                        variable: var_name,
                        iterable,
                        body,
                    }));
                }
            }
            self.current = checkpoint;
        }
        let (init_var, init_expr) = if self.check(&Token::Semicolon) {
            (None, None)
        } else {
            if self.peek_type_keyword() {
                let typ = Some(self.parse_type()?);
                let var_name = self.expect_identifier()?;
                let init = if self.check(&Token::Equal) {
                    self.advance();
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                (Some((var_name, typ)), init)
            } else {
                (None, Some(Box::new(self.parse_expr()?)))
            }
        };
        self.expect(&Token::Semicolon)?;
        let condition = if self.check(&Token::Semicolon) {
            None
        } else {
            Some(Box::new(self.parse_expr()?))
        };
        self.expect(&Token::Semicolon)?;
        let update = if self.check(&Token::RightParen) {
            None
        } else {
            Some(Box::new(self.parse_expr()?))
        };
        self.expect(&Token::RightParen)?;
        let body = if self.check(&Token::LeftBrace) {
            self.advance();
            let stmts = self.parse_block()?;
            self.expect(&Token::RightBrace)?;
            stmts
        } else {
            vec![self.parse_statement()?]
        };
        Ok(Statement::For(ForStmt {
            label: None,
            init_var,
            init_expr,
            condition,
            update,
            body,
        }))
    }
    fn parse_switch_statement(&mut self) -> ParseResult<Statement> {
        self.expect(&Token::Switch)?;
        self.expect(&Token::LeftParen)?;
        let expr = Box::new(self.parse_expr()?);
        self.expect(&Token::RightParen)?;
        self.expect(&Token::LeftBrace)?;
        let mut cases = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if self.match_token(&Token::Case) {
                // Parse pattern - could be Literal (5, "str"), Destructure (Type _, Type x), or Type.Variant
                let pattern = if let Token::Identifier(class_name) = self.peek() {
                    let name = class_name.clone();
                    self.advance();
                    // Check for Type.Variant pattern (enum variant matching)
                    if self.check(&Token::Dot) {
                        self.advance(); // consume dot
                        if let Token::Identifier(variant_name) = self.peek() {
                            let variant = variant_name.clone();
                            self.advance();
                            SwitchPattern::Literal(Expr::Identifier(variant))
                        } else {
                            SwitchPattern::Literal(Expr::Identifier(name))
                        }
                    } else if self.check(&Token::Underscore) {
                        self.advance();
                        SwitchPattern::Destructure(name, vec!["_".to_string()])
                    } else if let Token::Identifier(var_name) = self.peek() {
                        let vname = var_name.clone();
                        self.advance();
                        SwitchPattern::Destructure(name, vec![vname])
                    } else {
                        // Just the type name - treat as literal match
                        SwitchPattern::Literal(Expr::Identifier(name))
                    }
                } else {
                    // Regular literal pattern
                    let pattern_expr = self.parse_expr()?;
                    SwitchPattern::Literal(pattern_expr)
                };
                self.expect(&Token::DoubleArrow)?;
                let result = if self.check(&Token::LeftBrace) {
                    self.advance();
                    let stmts = self.parse_block()?;
                    self.expect(&Token::RightBrace)?;
                    Expr::Block(stmts)
                } else if self.check(&Token::Return) {
                    // Handle return statements in switch cases
                    let stmt = self.parse_statement()?;
                    Expr::Block(vec![stmt])
                } else {
                    self.parse_expr()?
                };
                self.expect_optional(&Token::Comma);
                cases.push(SwitchCase {
                    pattern,
                    guard: None,
                    result,
                });
            } else if self.match_token(&Token::Default) {
                self.expect(&Token::DoubleArrow)?;
                let result = if self.check(&Token::LeftBrace) {
                    self.advance();
                    let stmts = self.parse_block()?;
                    self.expect(&Token::RightBrace)?;
                    Expr::Block(stmts)
                } else if self.check(&Token::Return) {
                    // Handle return statements in switch cases
                    let stmt = self.parse_statement()?;
                    Expr::Block(vec![stmt])
                } else {
                    self.parse_expr()?
                };
                self.expect_optional(&Token::Comma);
                cases.push(SwitchCase {
                    pattern: SwitchPattern::Default,
                    guard: None,
                    result,
                });
            } else {
                break;
            }
        }
        self.expect(&Token::RightBrace)?;
        Ok(Statement::Switch(SwitchStmt { expr, cases }))
    }
    fn parse_try_statement(&mut self) -> ParseResult<Statement> {
        use crate::ast::{CatchBlock, TryStmt};
        self.expect(&Token::Try)?;
        self.expect(&Token::LeftBrace)?;
        let body = self.parse_block()?;
        self.expect(&Token::RightBrace)?;
        let mut catch_blocks = Vec::new();
        while self.match_token(&Token::Catch) {
            self.expect(&Token::LeftParen)?;
            let exception_type = if !self.check(&Token::RightParen) {
                Some(self.parse_type()?)
            } else {
                None
            };
            let exception_var = if !self.check(&Token::RightParen) {
                if self.match_token(&Token::Identifier("".to_string()))
                    || matches!(self.peek(), Token::Identifier(_))
                {
                    Some(self.expect_identifier()?)
                } else {
                    None
                }
            } else {
                None
            };
            self.expect(&Token::RightParen)?;
            self.expect(&Token::LeftBrace)?;
            let catch_body = self.parse_block()?;
            self.expect(&Token::RightBrace)?;
            catch_blocks.push(CatchBlock {
                exception_type,
                exception_var,
                body: catch_body,
            });
        }
        let finally_block = if self.match_token(&Token::Finally) {
            self.expect(&Token::LeftBrace)?;
            let stmts = self.parse_block()?;
            self.expect(&Token::RightBrace)?;
            Some(stmts)
        } else {
            None
        };
        Ok(Statement::Try(TryStmt {
            body,
            catch_blocks,
            finally_block,
        }))
    }
    pub fn parse_expr(&mut self) -> ParseResult<Expr> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> ParseResult<Expr> {
        let expr = self.parse_ternary()?;
        if self.check(&Token::Equal) {
            self.advance();
            let value = Box::new(self.parse_assignment()?);
            return Ok(Expr::Assign {
                target: Box::new(expr),
                value,
            });
        }
        if let Some(op) = self.match_tokens(&[
            Token::PlusEqual,
            Token::MinusEqual,
            Token::StarEqual,
            Token::SlashEqual,
            Token::PercentEqual,
        ]) {
            let binary_op = match op {
                Token::PlusEqual => BinaryOp::Add,
                Token::MinusEqual => BinaryOp::Sub,
                Token::StarEqual => BinaryOp::Mul,
                Token::SlashEqual => BinaryOp::Div,
                Token::PercentEqual => BinaryOp::Mod,
                _ => unreachable!(),
            };
            let value = Box::new(self.parse_assignment()?);
            return Ok(Expr::CompoundAssign {
                target: Box::new(expr),
                op: binary_op,
                value,
            });
        }
        if self.check(&Token::QuestionQuestionEqual) {
            self.advance();
            let value = Box::new(self.parse_assignment()?);
            return Ok(Expr::NullCoalesceAssign {
                target: Box::new(expr),
                value,
            });
        }
        Ok(expr)
    }
    fn parse_ternary(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_null_coalesce()?;
        if self.match_token(&Token::Question) {
            // Try to parse as ternary first - parse the then expression
            let checkpoint = self.current;
            match self.parse_expr() {
                Ok(then_expr) => {
                    // Check if there's a colon after the then expression
                    if self.check(&Token::Colon) {
                        self.advance(); // consume :
                        let else_expr = Box::new(self.parse_expr()?);
                        expr = Expr::Ternary {
                            condition: Box::new(expr),
                            then_expr: Box::new(then_expr),
                            else_expr,
                        };
                    } else {
                        // No colon, this was a try operator, not a ternary
                        // Reset and treat ? as propagate
                        self.current = checkpoint;
                        expr = Expr::Propagate(Box::new(expr));
                    }
                }
                Err(_) => {
                    // Failed to parse then expression, treat as try operator
                    self.current = checkpoint;
                    expr = Expr::Propagate(Box::new(expr));
                }
            }
        }
        Ok(expr)
    }
    fn parse_null_coalesce(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_logical_or()?;
        while !self.check(&Token::QuestionQuestionEqual)
            && self.match_token(&Token::QuestionQuestion)
        {
            let right = Box::new(self.parse_unary()?);
            expr = Expr::NullCoalesce {
                left: Box::new(expr),
                right,
            };
        }
        Ok(expr)
    }
    fn parse_logical_or(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_logical_and()?;
        while self.match_token(&Token::PipePipe) {
            let right = Box::new(self.parse_logical_and()?);
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: BinaryOp::Or,
                right,
            };
        }
        Ok(expr)
    }
    fn parse_logical_and(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_equality()?;
        while self.match_token(&Token::AmpersandAmpersand) {
            let right = Box::new(self.parse_equality()?);
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: BinaryOp::And,
                right,
            };
        }
        Ok(expr)
    }
    fn parse_equality(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_comparison()?;
        while let Some(op) = self.match_tokens(&[Token::EqualEqual, Token::BangEqual]) {
            let right = Box::new(self.parse_comparison()?);
            let binary_op = match op {
                Token::EqualEqual => BinaryOp::Equal,
                Token::BangEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: binary_op,
                right,
            };
        }
        Ok(expr)
    }
    fn parse_comparison(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_additive()?;
        while let Some(op) = self.match_tokens(&[
            Token::Less,
            Token::LessEqual,
            Token::Greater,
            Token::GreaterEqual,
        ]) {
            let right = Box::new(self.parse_additive()?);
            let binary_op = match op {
                Token::Less => BinaryOp::Less,
                Token::LessEqual => BinaryOp::LessEqual,
                Token::Greater => BinaryOp::Greater,
                Token::GreaterEqual => BinaryOp::GreaterEqual,
                _ => unreachable!(),
            };
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: binary_op,
                right,
            };
        }
        if self.match_token(&Token::Is) {
            let typ = self.parse_type()?;
            expr = Expr::Is {
                expr: Box::new(expr),
                typ,
                negated: false,
            };
        } else if self.match_token(&Token::As) {
            let typ = self.parse_type()?;
            expr = Expr::Cast {
                expr: Box::new(expr),
                typ,
            };
        }
        Ok(expr)
    }
    fn parse_additive(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_multiplicative()?;
        while let Some(op) = self.match_tokens(&[Token::Plus, Token::Minus]) {
            let right = Box::new(self.parse_multiplicative()?);
            let binary_op = match op {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: binary_op,
                right,
            };
        }
        Ok(expr)
    }
    fn parse_multiplicative(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_unary()?;
        while let Some(op) = self.match_tokens(&[Token::Star, Token::Slash, Token::Percent]) {
            let right = Box::new(self.parse_unary()?);
            let binary_op = match op {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Percent => BinaryOp::Mod,
                _ => unreachable!(),
            };
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                op: binary_op,
                right,
            };
        }
        Ok(expr)
    }
    fn parse_unary(&mut self) -> ParseResult<Expr> {
        if self.match_token(&Token::Await) {
            let expr = Box::new(self.parse_unary()?);
            return Ok(Expr::Await(expr));
        }
        if let Some(op) = self.match_tokens(&[Token::Bang, Token::Minus, Token::Tilde]) {
            let unary_op = match op {
                Token::Bang => UnaryOp::Not,
                Token::Minus => UnaryOp::Neg,
                Token::Tilde => UnaryOp::BitNot,
                _ => unreachable!(),
            };
            let operand = Box::new(self.parse_unary()?);
            return Ok(Expr::UnaryOp {
                op: unary_op,
                operand,
            });
        }
        self.parse_postfix()
    }
    fn parse_postfix(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.match_token(&Token::LeftParen) {
                let args = self.parse_arguments()?;
                self.expect(&Token::RightParen)?;
                expr = Expr::Call {
                    func: Box::new(expr),
                    args,
                };
            } else if self.match_token(&Token::Dot) {
                let property = self.expect_identifier()?;
                if self.check(&Token::LeftParen) {
                    self.advance();
                    let args = self.parse_arguments()?;
                    self.expect(&Token::RightParen)?;
                    expr = Expr::MethodCall {
                        object: Box::new(expr),
                        method: property,
                        args,
                    };
                } else {
                    expr = Expr::PropertyAccess {
                        object: Box::new(expr),
                        property,
                    };
                }
            } else if self.match_token(&Token::QuestionDot) {
                let property = self.expect_identifier()?;
                let args = if self.check(&Token::LeftParen) {
                    self.advance();
                    let parsed_args = self.parse_arguments()?;
                    self.expect(&Token::RightParen)?;
                    Some(parsed_args)
                } else {
                    None
                };
                expr = Expr::NullAware {
                    object: Box::new(expr),
                    property,
                    args,
                };
            } else if self.match_token(&Token::LeftBracket) {
                let index = Box::new(self.parse_expr()?);
                self.expect(&Token::RightBracket)?;
                expr = Expr::Index {
                    object: Box::new(expr),
                    index,
                };
            } else if self.match_token(&Token::Bang) {
                expr = Expr::NullAssertion(Box::new(expr));
            } else {
                break;
            }
        }
        Ok(expr)
    }
    fn parse_primary(&mut self) -> ParseResult<Expr> {
        match self.peek() {
            Token::IntLiteral(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::Literal(Literal::Int(n)))
            }
            Token::DoubleLiteral(f) => {
                let f = *f;
                self.advance();
                Ok(Expr::Literal(Literal::Double(f)))
            }
            Token::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                if s.contains('$') {
                    Ok(self.parse_string_interpolation(&s)?)
                } else {
                    Ok(Expr::Literal(Literal::String(s)))
                }
            }
            Token::BoolLiteral(b) => {
                let b = *b;
                self.advance();
                Ok(Expr::Literal(Literal::Bool(b)))
            }
            Token::NullLiteral => {
                self.advance();
                Ok(Expr::Literal(Literal::Null))
            }
            // Allow type keywords to be used as identifiers (e.g., int.parse(x))
            Token::Int => {
                self.advance();
                Ok(Expr::Identifier("int".to_string()))
            }
            Token::Double => {
                self.advance();
                Ok(Expr::Identifier("double".to_string()))
            }
            Token::String => {
                self.advance();
                Ok(Expr::Identifier("String".to_string()))
            }
            Token::Bool => {
                self.advance();
                Ok(Expr::Identifier("bool".to_string()))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                if self.check(&Token::LeftParen) {
                    match name.as_str() {
                        "Ok" => {
                            self.advance();
                            let args = self.parse_arguments()?;
                            self.expect(&Token::RightParen)?;
                            if let Some(arg) = args.into_iter().next() {
                                return Ok(Expr::ResultConstructor {
                                    variant: "Ok".to_string(),
                                    value: Box::new(arg),
                                });
                            }
                            return Ok(Expr::ResultConstructor {
                                variant: "Ok".to_string(),
                                value: Box::new(Expr::Literal(Literal::Null)),
                            });
                        }
                        "Err" => {
                            self.advance();
                            let args = self.parse_arguments()?;
                            self.expect(&Token::RightParen)?;
                            if let Some(arg) = args.into_iter().next() {
                                return Ok(Expr::ResultConstructor {
                                    variant: "Err".to_string(),
                                    value: Box::new(arg),
                                });
                            }
                            return Ok(Expr::ResultConstructor {
                                variant: "Err".to_string(),
                                value: Box::new(Expr::Literal(Literal::Null)),
                            });
                        }
                        "Some" => {
                            self.advance();
                            let args = self.parse_arguments()?;
                            self.expect(&Token::RightParen)?;
                            if let Some(arg) = args.into_iter().next() {
                                return Ok(Expr::OptionConstructor {
                                    variant: "Some".to_string(),
                                    value: Some(Box::new(arg)),
                                });
                            }
                            return Ok(Expr::OptionConstructor {
                                variant: "Some".to_string(),
                                value: None,
                            });
                        }
                        "None" => {
                            self.advance();
                            let _args = self.parse_arguments()?;
                            self.expect(&Token::RightParen)?;
                            return Ok(Expr::OptionConstructor {
                                variant: "None".to_string(),
                                value: None,
                            });
                        }
                        _ => {}
                    }
                }
                Ok(Expr::Identifier(name))
            }
            Token::LeftParen => {
                let checkpoint = self.current;
                self.advance();
                if let Ok(param_names) = self.try_parse_lambda_params() {
                    if self.check(&Token::DoubleArrow) {
                        self.advance();
                        let body = Box::new(self.parse_expr()?);
                        let params = param_names
                            .into_iter()
                            .map(|name| Parameter {
                                name,
                                typ: Type::Dynamic,
                                is_required: true,
                                default_value: None,
                            })
                            .collect();
                        return Ok(Expr::Lambda { params, body });
                    }
                }
                self.current = checkpoint;
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RightParen)?;
                Ok(expr)
            }
            Token::LeftBracket => {
                self.advance();
                let items = self.parse_list_items()?;
                self.expect(&Token::RightBracket)?;
                Ok(Expr::ListLiteral(items))
            }
            Token::LeftBrace => {
                self.advance();
                if self.check(&Token::RightBrace) {
                    self.advance();
                    Ok(Expr::MapLiteral(Vec::new()))
                } else {
                    let mut pairs = Vec::new();
                    loop {
                        let key = self.parse_expr()?;
                        self.expect(&Token::Colon)?;
                        let value = self.parse_expr()?;
                        pairs.push((key, value));
                        if !self.check(&Token::RightBrace) {
                            self.expect_optional(&Token::Comma);
                        } else {
                            break;
                        }
                    }
                    self.expect(&Token::RightBrace)?;
                    Ok(Expr::MapLiteral(pairs))
                }
            }
            Token::New => {
                self.advance();
                let class_name = self.expect_identifier()?;
                self.expect(&Token::LeftParen)?;
                let args = self.parse_arguments()?;
                self.expect(&Token::RightParen)?;
                Ok(Expr::New {
                    class_name,
                    constructor: None,
                    args,
                })
            }
            Token::This => {
                self.advance();
                Ok(Expr::This)
            }
            _ => Err(CrabError::parse_error(
                0,
                0,
                format!("Unexpected token in expression: {:?}", self.peek()),
            )),
        }
    }
    fn parse_arguments(&mut self) -> ParseResult<Vec<Expr>> {
        let mut args = Vec::new();
        while !self.check(&Token::RightParen) && !self.is_at_end() {
            args.push(self.parse_expr()?);
            if !self.check(&Token::RightParen) {
                self.expect_optional(&Token::Comma);
            }
        }
        Ok(args)
    }
    fn parse_list_items(&mut self) -> ParseResult<Vec<Expr>> {
        let mut items = Vec::new();
        while !self.check(&Token::RightBracket) && !self.is_at_end() {
            items.push(self.parse_expr()?);
            if !self.check(&Token::RightBracket) {
                self.expect_optional(&Token::Comma);
            }
        }
        Ok(items)
    }
    fn parse_string_interpolation(&self, s: &str) -> ParseResult<Expr> {
        use crate::ast::StringPart;
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut chars = s.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '$' {
                if let Some(&next_ch) = chars.peek() {
                    if next_ch.is_alphabetic() || next_ch == '_' {
                        if !current.is_empty() {
                            parts.push(StringPart::Static(current.clone()));
                            current.clear();
                        }
                        let mut var_name = String::new();
                        while let Some(&c) = chars.peek() {
                            if c.is_alphanumeric() || c == '_' {
                                var_name.push(c);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        let expr = Expr::Identifier(var_name);
                        parts.push(StringPart::Interpolation(Box::new(expr)));
                    } else if next_ch == '{' {
                        if !current.is_empty() {
                            parts.push(StringPart::Static(current.clone()));
                            current.clear();
                        }
                        chars.next();
                        let mut expr_str = String::new();
                        let mut brace_count = 1;
                        while let Some(c) = chars.next() {
                            if c == '{' {
                                brace_count += 1;
                                expr_str.push(c);
                            } else if c == '}' {
                                brace_count -= 1;
                                if brace_count == 0 {
                                    break;
                                }
                                expr_str.push(c);
                            } else {
                                expr_str.push(c);
                            }
                        }
                        let mut expr_parser = Parser::new(&expr_str)?;
                        let expr = expr_parser.parse_expr()?;
                        parts.push(StringPart::Interpolation(Box::new(expr)));
                    } else {
                        current.push(ch);
                    }
                } else {
                    current.push(ch);
                }
            } else {
                current.push(ch);
            }
        }
        if !current.is_empty() {
            parts.push(StringPart::Static(current));
        }
        Ok(Expr::StringInterpolation(parts))
    }
    fn parse_simple_method(
        &mut self,
        _is_static: bool,
        is_getter: bool,
        is_setter: bool,
    ) -> ParseResult<MethodDecl> {
        let return_type = if is_getter {
            Some(self.parse_type()?)
        } else {
            None
        };
        let name = self.expect_identifier()?;
        let params = if self.check(&Token::LeftParen) {
            self.advance();
            let p = self.parse_parameters()?;
            self.expect(&Token::RightParen)?;
            p
        } else {
            Vec::new()
        };
        let body = if self.match_token(&Token::DoubleArrow) {
            FunctionBody::Expression(Box::new(self.parse_expr()?))
        } else {
            self.expect(&Token::LeftBrace)?;
            let stmts = self.parse_block()?;
            self.expect(&Token::RightBrace)?;
            FunctionBody::Block(stmts)
        };
        Ok(MethodDecl {
            name,
            params,
            return_type,
            body,
            is_static: false,
            is_getter,
            is_setter,
            is_override: false,
        })
    }
    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or(&Token::Eof)
    }
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }
    fn check(&self, token: &Token) -> bool {
        matches!(
            (self.peek(), token),
            (Token::LeftBrace, Token::LeftBrace)
                | (Token::RightBrace, Token::RightBrace)
                | (Token::LeftParen, Token::LeftParen)
                | (Token::RightParen, Token::RightParen)
                | (Token::LeftBracket, Token::LeftBracket)
                | (Token::RightBracket, Token::RightBracket)
                | (Token::Semicolon, Token::Semicolon)
                | (Token::Comma, Token::Comma)
                | (Token::Colon, Token::Colon)
                | (Token::Dot, Token::Dot)
                | (Token::At, Token::At)
                | (Token::Equal, Token::Equal)
                | (Token::Less, Token::Less)
                | (Token::Greater, Token::Greater)
                | (Token::Plus, Token::Plus)
                | (Token::Minus, Token::Minus)
                | (Token::Star, Token::Star)
                | (Token::Slash, Token::Slash)
                | (Token::Percent, Token::Percent)
                | (Token::Bang, Token::Bang)
                | (Token::Question, Token::Question)
                | (Token::Ampersand, Token::Ampersand)
                | (Token::Pipe, Token::Pipe)
                | (Token::Caret, Token::Caret)
                | (Token::Tilde, Token::Tilde)
                | (Token::EqualEqual, Token::EqualEqual)
                | (Token::BangEqual, Token::BangEqual)
                | (Token::LessEqual, Token::LessEqual)
                | (Token::GreaterEqual, Token::GreaterEqual)
                | (Token::AmpersandAmpersand, Token::AmpersandAmpersand)
                | (Token::PipePipe, Token::PipePipe)
                | (Token::QuestionDot, Token::QuestionDot)
                | (Token::QuestionQuestion, Token::QuestionQuestion)
                | (Token::DoubleArrow, Token::DoubleArrow)
                | (Token::Arrow, Token::Arrow)
                | (Token::Eof, Token::Eof)
                | (Token::Class, Token::Class)
                | (Token::Void, Token::Void)
                | (Token::Int, Token::Int)
                | (Token::Double, Token::Double)
                | (Token::Bool, Token::Bool)
                | (Token::String, Token::String)
                | (Token::Var, Token::Var)
                | (Token::Const, Token::Const)
                | (Token::If, Token::If)
                | (Token::Else, Token::Else)
                | (Token::While, Token::While)
                | (Token::Do, Token::Do)
                | (Token::For, Token::For)
                | (Token::Switch, Token::Switch)
                | (Token::Enum, Token::Enum)
                | (Token::Case, Token::Case)
                | (Token::Default, Token::Default)
                | (Token::Return, Token::Return)
                | (Token::Break, Token::Break)
                | (Token::Continue, Token::Continue)
                | (Token::Null, Token::Null)
                | (Token::True, Token::True)
                | (Token::False, Token::False)
                | (Token::Try, Token::Try)
                | (Token::Catch, Token::Catch)
                | (Token::Finally, Token::Finally)
                | (Token::Throw, Token::Throw)
                | (Token::Import, Token::Import)
                | (Token::Export, Token::Export)
                | (Token::Async, Token::Async)
                | (Token::Await, Token::Await)
                | (Token::Sealed, Token::Sealed)
                | (Token::Abstract, Token::Abstract)
                | (Token::Interface, Token::Interface)
                | (Token::Base, Token::Base)
                | (Token::Final, Token::Final)
                | (Token::Mixin, Token::Mixin)
                | (Token::Extension, Token::Extension)
                | (Token::On, Token::On)
                | (Token::Extends, Token::Extends)
                | (Token::Implements, Token::Implements)
                | (Token::With, Token::With)
                | (Token::Static, Token::Static)
                | (Token::Get, Token::Get)
                | (Token::Set, Token::Set)
                | (Token::New, Token::New)
                | (Token::This, Token::This)
                | (Token::In, Token::In)
                | (Token::Is, Token::Is)
                | (Token::As, Token::As)
                | (Token::Underscore, Token::Underscore)
                | (Token::SlashEqual, Token::SlashEqual)
                | (Token::PercentEqual, Token::PercentEqual)
                | (Token::QuestionQuestionEqual, Token::QuestionQuestionEqual)
        )
    }
    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }
    fn match_tokens(&mut self, tokens: &[Token]) -> Option<Token> {
        for token in tokens {
            if self.check(token) {
                let result = self.peek().clone();
                self.advance();
                return Some(result);
            }
        }
        None
    }
    fn expect(&mut self, token: &Token) -> ParseResult<()> {
        if self.check(token) {
            self.advance();
            Ok(())
        } else {
            Err(CrabError::parse_error(
                0,
                0,
                format!("Expected {:?}, got {:?}", token, self.peek()),
            ))
        }
    }
    fn expect_optional(&mut self, token: &Token) {
        let _ = self.match_token(token);
    }
    fn expect_identifier(&mut self) -> ParseResult<String> {
        match self.peek() {
            Token::Identifier(name) => {
                let result = name.clone();
                self.advance();
                Ok(result)
            }
            Token::New => {
                self.advance();
                Ok("new".to_string())
            }
            Token::Get => {
                self.advance();
                Ok("get".to_string())
            }
            Token::Set => {
                self.advance();
                Ok("set".to_string())
            }
            Token::On => {
                self.advance();
                Ok("on".to_string())
            }
            Token::Show => {
                self.advance();
                Ok("show".to_string())
            }
            Token::Hide => {
                self.advance();
                Ok("hide".to_string())
            }
            Token::In => {
                self.advance();
                Ok("in".to_string())
            }
            Token::Is => {
                self.advance();
                Ok("is".to_string())
            }
            Token::As => {
                self.advance();
                Ok("as".to_string())
            }
            _ => Err(CrabError::parse_error(
                0,
                0,
                format!("Expected identifier, got {:?}", self.peek()),
            )),
        }
    }
    fn check_type_start(&self) -> bool {
        matches!(
            self.peek(),
            Token::Int
                | Token::Double
                | Token::Bool
                | Token::String
                | Token::Void
                | Token::Identifier(_)
        )
    }
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.check(&Token::Eof)
    }
    fn peek_type_keyword(&self) -> bool {
        matches!(
            self.peek(),
            Token::Int | Token::Double | Token::String | Token::Bool | Token::Void | Token::Var
        )
    }
    fn try_parse_lambda_params(&mut self) -> ParseResult<Vec<String>> {
        let mut params = Vec::new();
        if self.check(&Token::RightParen) {
            self.advance();
            return Ok(params);
        }
        match self.peek() {
            Token::Identifier(name) => {
                params.push(name.clone());
                self.advance();
            }
            _ => {
                return Err(CrabError::parse_error(
                    0,
                    0,
                    format!(
                        "Expected identifier in lambda params, got {:?}",
                        self.peek()
                    ),
                ));
            }
        }
        while self.check(&Token::Comma) {
            self.advance();
            match self.peek() {
                Token::Identifier(name) => {
                    params.push(name.clone());
                    self.advance();
                }
                _ => {
                    return Err(CrabError::parse_error(
                        0,
                        0,
                        format!(
                            "Expected identifier after comma in lambda params, got {:?}",
                            self.peek()
                        ),
                    ));
                }
            }
        }
        if self.check(&Token::RightParen) {
            self.advance();
            Ok(params)
        } else {
            Err(CrabError::parse_error(
                0,
                0,
                format!("Expected ) in lambda params, got {:?}", self.peek()),
            ))
        }
    }
}
