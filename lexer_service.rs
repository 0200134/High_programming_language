use crate::data_structures::{Span, Token, TokenKind};

pub struct LexerService<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    position: usize,
    tokens: Vec<Token>,
    index: usize,
}

impl<'a> LexerService<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Self {
            source,
            chars: source.chars().peekable(),
            position: 0,
            tokens: vec![],
            index: 0,
        };
        lexer.tokens = lexer.tokenize();
        lexer
    }

    pub fn next_token(&mut self) -> Token {
        if self.index < self.tokens.len() {
            let tok = self.tokens[self.index].clone();
            self.index += 1;
            tok
        } else {
            Token {
                kind: TokenKind::Eof,
                span: Span { start: self.position, end: self.position },
            }
        }
    }

    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while self.peek().is_some() {
            self.skip_whitespace();
            let start = self.position;

            let current_char = match self.peek() {
                Some(&c) => c,
                None => break,
            };

            let token = match current_char {
                c if c.is_alphabetic() || c == '_' => self.read_identifier_or_keyword(start),
                c if c.is_digit(10) => self.read_number(start),
                c => self.read_symbol(start, c),
            };

            tokens.push(token);
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span { start: self.position, end: self.position },
        });

        tokens
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn advance(&mut self) -> Option<char> {
        let next_char = self.chars.next();
        if next_char.is_some() {
            self.position += 1;
        }
        next_char
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn read_identifier_or_keyword(&mut self, start: usize) -> Token {
        let mut literal = String::new();

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || *c == '_' {
                literal.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        let kind = match literal.as_str() {
            "fn" => TokenKind::Fn,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "return" => TokenKind::Return,
            "match" => TokenKind::Match,
            "macro" => TokenKind::Macro,
            "type_of" => TokenKind::TypeOf,
            "eval" => TokenKind::Eval,
            "reflect" => TokenKind::Reflect,
            "async" => TokenKind::Async,
            "await" => TokenKind::Await,
            "true" => TokenKind::BooleanLiteral(true),
            "false" => TokenKind::BooleanLiteral(false),
            "int" => TokenKind::Int,
            "float" => TokenKind::Float,
            "bool" => TokenKind::Bool,
            "string" => TokenKind::String,
            "void" => TokenKind::Void,
            "any" => TokenKind::Any,
            _ => TokenKind::Identifier(literal.clone()),
        };

        Token {
            kind,
            span: Span { start, end: self.position },
        }
    }

    fn read_number(&mut self, start: usize) -> Token {
        let mut literal = String::new();
        let mut is_float = false;

        while let Some(c) = self.peek() {
            if c.is_digit(10) {
                literal.push(self.advance().unwrap());
            } else if *c == '.' {
                is_float = true;
                literal.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        let kind = if is_float {
            TokenKind::FloatLiteral(literal.clone())
        } else {
            let value = literal.parse::<i64>().unwrap_or_default();
            TokenKind::IntegerLiteral(value)
        };

        Token {
            kind,
            span: Span { start, end: self.position },
        }
    }

    fn read_symbol(&mut self, start: usize, current_char: char) -> Token {
        let kind = match current_char {
            '=' => {
                self.advance();
                if self.peek() == Some(&'=') {
                    self.advance();
                    TokenKind::Eq
                } else {
                    TokenKind::Assign
                }
            }
            '+' => {
                self.advance();
                if self.peek() == Some(&'=') {
                    self.advance();
                    TokenKind::PlusAssign
                } else {
                    TokenKind::Plus
                }
            }
            '-' => {
                self.advance();
                if self.peek() == Some(&'=') {
                    self.advance();
                    TokenKind::MinusAssign
                } else {
                    TokenKind::Minus
                }
            }
            '*' => { self.advance(); TokenKind::Asterisk }
            '/' => { self.advance(); TokenKind::Slash }
            '%' => { self.advance(); TokenKind::Percent }
            '!' => {
                self.advance();
                if self.peek() == Some(&'=') {
                    self.advance();
                    TokenKind::Neq
                } else {
                    TokenKind::Bang
                }
            }
            '&' => {
                self.advance();
                if self.peek() == Some(&'&') {
                    self.advance();
                    TokenKind::And
                } else {
                    TokenKind::BitAnd
                }
            }
            '|' => {
                self.advance();
                if self.peek() == Some(&'|') {
                    self.advance();
                    TokenKind::Or
                } else {
                    TokenKind::BitOr
                }
            }
            '^' => { self.advance(); TokenKind::BitXor }
            '<' => {
                self.advance();
                if self.peek() == Some(&'<') {
                    self.advance();
                    TokenKind::ShiftLeft
                } else if self.peek() == Some(&'=') {
                    self.advance();
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                self.advance();
                if self.peek() == Some(&'>') {
                    self.advance();
                    TokenKind::ShiftRight
                } else if self.peek() == Some(&'=') {
                    self.advance();
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            }
            '?' => { self.advance(); TokenKind::Question }
            ':' => { self.advance(); TokenKind::Colon }
            '{' => { self.advance(); TokenKind::LBrace }
            '}' => { self.advance(); TokenKind::RBrace }
            '(' => { self.advance(); TokenKind::LParen }
            ')' => { self.advance(); TokenKind::RParen }
            '[' => { self.advance(); TokenKind::LBracket }
            ']' => { self.advance(); TokenKind::RBracket }
            ',' => { self.advance(); TokenKind::Comma }
            ';' => { self.advance(); TokenKind::Semicolon }
            '.' => { self.advance(); TokenKind::Dot }
            _ => {
                self.advance();
                TokenKind::Illegal(current_char)
            }
        };

        Token {
            kind,
            span: Span { start, end: self.position },
        }
    }
}
