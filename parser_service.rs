use crate::data_structures::*;
use crate::lexer_service::LexerService;

pub struct ParserService<'a> {
    lexer: LexerService<'a>,
    current: Token,
    peek: Token,
}

impl<'a> ParserService<'a> {
    pub fn new(mut lexer: LexerService<'a>) -> Self {
        let mut parser = Self {
            lexer,
            current: Token { kind: TokenKind::Eof, span: Span { start: 0, end: 0 } },
            peek: Token { kind: TokenKind::Eof, span: Span { start: 0, end: 0 } },
        };
        parser.advance();
        parser.advance();
        parser
    }

    fn advance(&mut self) {
        let next = self.lexer.next_token();
        self.current = std::mem::replace(&mut self.peek, next);
    }

    pub fn parse_program(&mut self) -> Program {
        let mut statements = vec![];
        while !matches!(self.current.kind, TokenKind::Eof) {
            if let Some(stmt) = self.parse_statement() {
                statements.push(Box::new(stmt));
            } else {
                self.advance();
            }
        }
        Program {
            root_id: 0,
            statements,
            span: Span { start: 0, end: 0 },
        }
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.current.kind {
            TokenKind::Let => self.parse_let_statement(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::For => self.parse_for_statement(),
            TokenKind::Macro => self.parse_macro_definition(),
            TokenKind::LBrace => self.parse_block_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Option<Statement> {
        self.advance(); // consume 'let'
        let is_mutable = if matches!(self.current.kind, TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        };

        let name = if let TokenKind::Identifier(id) = &self.current.kind {
            id.clone()
        } else {
            return None;
        };
        self.advance();

        let type_annotation = if matches!(self.current.kind, TokenKind::Colon) {
            self.advance();
            self.parse_type_annotation()
        } else {
            None
        };

        if !matches!(self.current.kind, TokenKind::Assign) {
            return None;
        }
        self.advance();

        let value = self.parse_expression()?;
        Some(Statement::LetStatement {
            name,
            value: Box::new(value),
            type_annotation,
            is_mutable,
        })
    }

    fn parse_return_statement(&mut self) -> Option<Statement> {
        self.advance(); // consume 'return'
        let expr = self.parse_expression()?;
        Some(Statement::ReturnStatement(Box::new(expr)))
    }

    fn parse_if_statement(&mut self) -> Option<Statement> {
        self.advance(); // consume 'if'
        let condition = self.parse_expression()?;
        let then_branch = self.parse_statement()?;
        let else_branch = if matches!(self.current.kind, TokenKind::Else) {
            self.advance();
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        Some(Statement::IfStatement {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn parse_for_statement(&mut self) -> Option<Statement> {
        self.advance(); // consume 'for'
        let initializer = if !matches!(self.current.kind, TokenKind::Semicolon) {
            Some(Box::new(self.parse_statement()?))
        } else {
            self.advance();
            None
        };

        let condition = if !matches!(self.current.kind, TokenKind::Semicolon) {
            Some(Box::new(self.parse_expression()?))
        } else {
            self.advance();
            None
        };

        let increment = if !matches!(self.current.kind, TokenKind::LBrace) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        let body = self.parse_statement()?;
        Some(Statement::ForStatement {
            initializer,
            condition,
            increment,
            body: Box::new(body),
        })
    }

    fn parse_macro_definition(&mut self) -> Option<Statement> {
        self.advance(); // consume 'macro'
        let name = if let TokenKind::Identifier(id) = &self.current.kind {
            id.clone()
        } else {
            return None;
        };
        self.advance();

        let mut params = vec![];
        if matches!(self.current.kind, TokenKind::LParen) {
            self.advance();
            while !matches!(self.current.kind, TokenKind::RParen) {
                if let TokenKind::Identifier(id) = &self.current.kind {
                    params.push(id.clone());
                    self.advance();
                    if matches!(self.current.kind, TokenKind::Comma) {
                        self.advance();
                    }
                } else {
                    break;
                }
            }
            self.advance(); // consume ')'
        }

        let body = self.parse_block_statement()?;
        Some(Statement::MacroDefinition {
            name,
            parameters: params,
            body: Box::new(body),
        })
    }

    fn parse_block_statement(&mut self) -> Option<Statement> {
        self.advance(); // consume '{'
        let mut statements = vec![];
        while !matches!(self.current.kind, TokenKind::RBrace) {
            if let Some(stmt) = self.parse_statement() {
                statements.push(Box::new(stmt));
            } else {
                self.advance();
            }
        }
        self.advance(); // consume '}'
        Some(Statement::BlockStatement {
            statements,
            span: Span { start: 0, end: 0 },
        })
    }

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let expr = self.parse_expression()?;
        Some(Statement::ExpressionStatement(Box::new(expr)))
    }

    fn parse_expression(&mut self) -> Option<Expression> {
        let start = self.current.span.start;

        match &self.current.kind {
            TokenKind::Eval => {
                self.advance();
                let inner = self.parse_expression()?;
                Some(Expression::Eval(Span { start, end: self.current.span.end }, Box::new(inner)))
            }
            TokenKind::Reflect => {
                self.advance();
                let inner = self.parse_expression()?;
                Some(Expression::Reflect(Span { start, end: self.current.span.end }, Box::new(inner)))
            }
            TokenKind::TypeOf => {
                self.advance();
                let inner = self.parse_expression()?;
                Some(Expression::TypeOf(Span { start, end: self.current.span.end }, Box::new(inner)))
            }
            TokenKind::Identifier(name) => {
                let id = name.clone();
                self.advance();
                if matches!(self.current.kind, TokenKind::LParen) {
                    self.advance();
                    let mut args = vec![];
                    while !matches!(self.current.kind, TokenKind::RParen) {
                        let arg = self.parse_expression()?;
                        args.push(Box::new(arg));
                        if matches!(self.current.kind, TokenKind::Comma) {
                            self.advance();
                        }
                    }
                    self.advance(); // consume ')'
                    Some(Expression::MacroCall(Span { start, end: self.current.span.end }, id, args))
                } else {
                    Some(Expression::Identifier(Span { start, end: self.current.span.end }, id))
                }
            }
            TokenKind::IntegerLiteral(val) => {
                let v = Value::Integer(*val);
                self.advance();
                Some(Expression::Literal(Span { start, end: self.current.span.end }, v))
            }
            TokenKind::FloatLiteral(s) => {
                let v = Value::Float(s.parse().unwrap_or(0.0));
                self.advance();
                Some(Expression::Literal(Span { start, end: self.current.span.end }, v))
            }
            TokenKind::BooleanLiteral(b) => {
                let v = Value::Boolean(*b);
                self.advance();
                Some(Expression::Literal(Span { start, end: self.current.span.end }, v))
            }
            TokenKind::LParen => {
                self.advance();
                let inner = self.parse_expression()?;
                if matches!(self.current.kind, TokenKind::RParen) {
                    self.advance();
                    Some(Expression::Grouped(Span { start, end: self.current.span.end }, Box::new(inner)))
                } else {
                    None
                }
            }
            _ => None
        }
    }

        fn parse_type_annotation(&mut self) -> Option<TypeAnnotation> {
        match &self.current.kind {
            TokenKind::Identifier(name) => Some(TypeAnnotation::Custom(name.clone())),
            TokenKind::Int => Some(TypeAnnotation::Int),
            TokenKind::Float => Some(TypeAnnotation::Float),
            TokenKind::Bool => Some(TypeAnnotation::Bool),
            TokenKind::String => Some(TypeAnnotation::String),
            TokenKind::Void => Some(TypeAnnotation::Void),
            TokenKind::Any => Some(TypeAnnotation::Any),
            _ => None,
        }
    }
}
