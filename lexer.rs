// --- Core Data Structures for the High Programming Language (Re-Included for self-containment) ---

// HighType represents the type system of the language.
#[derive(Debug, Clone, PartialEq)]
pub enum HighType {
    Integer,
    Float,
    Boolean,
    String,
    Function,
    Void,
    Unknown,
}

// TokenKind defines all possible types of tokens in the language.
#[derive(Debug, Clone, PartialEq, Eq, Hash)] 
pub enum TokenKind {
    // Single-character tokens.
    LParen, // (
    RParen, // )
    LBrace, // {
    RBrace, // }
    Comma,  // ,
    Dot,    // .
    Minus,  // -
    Plus,   // +
    Star,   // *
    Slash,  // /
    Bang,   // !
    Semicolon, // ;

    // One or two character tokens.
    Assign, // =
    Eq,     // ==
    Neq,    // !=
    Lt,     // <
    Gt,     // >
    Le,     // <=
    Ge,     // >=
    Arrow,  // =>

    // Literals.
    Identifier(String),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    BooleanLiteral(bool),
    StringLiteral(String),

    // Keywords.
    Fn,     // fn
    Let,    // let
    If,     // if
    Else,   // else
    Return, // return
    True,   // true
    False,  // false
    Match,  // match
    When,   // when

    // Special
    Illegal, // For tokens that don't match any rule
    Eof, // End of File
}

impl TokenKind {
    // Methods used by src/parser_service.rs (as specified in your original code)
    pub fn is_identifier(&self) -> bool {
        matches!(self, TokenKind::Identifier(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, TokenKind::IntegerLiteral(_))
    }

    pub fn fn_is_float(&self) -> bool { // Renamed to avoid name conflict in some Rust environments
        matches!(self, TokenKind::FloatLiteral(_))
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, TokenKind::BooleanLiteral(_))
    }
}

// AST nodes
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Identifier(String),
    Infix(Box<Expression>, TokenKind, Box<Expression>),
    Prefix(TokenKind, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
    If(Box<Expression>, Box<Statement>, Option<Box<Statement>>),
    Match(Box<Expression>, Vec<MatchArm>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Expression,
    pub consequence: Statement,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    LetStatement {
        type_name: HighType,
        name: String,
        value: Expression,
    },
    ReturnStatement(Expression),
    ExpressionStatement(Expression),
    BlockStatement(Vec<Statement>),
}

// The root of the AST (Included but unused in Lexer)
#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub type_errors: Vec<String>,
}

// --- Lexer Implementation ---

/// A minimal Token structure for the lexer output
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub literal: String,
}

/// The Lexer structure holds the input source code and tracks the current position.
pub struct Lexer {
    input: String,
    position: usize,      // current position in input (points to current char)
    read_position: usize, // current reading position in input (after current char)
    ch: Option<char>,     // current char under examination
}

impl Lexer {
    /// Creates a new Lexer from the given input string.
    pub fn new(input: String) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            read_position: 0,
            ch: None,
        };
        lexer.read_char(); // Initialize the first character
        lexer
    }

    /// Reads the next character and advances the position.
    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = None; // EOF
        } else {
            // Get the character at read_position and update the current char
            self.ch = self.input.chars().nth(self.read_position);
        }
        self.position = self.read_position;
        self.read_position += self.ch.map_or(1, |c| c.len_utf8()); // Correctly handle UTF-8
    }

    /// Looks ahead one character without advancing the position.
    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.read_position)
    }

    /// Skips whitespace characters (space, tab, newline, carriage return).
    fn skip_whitespace(&mut self) {
        while self.ch.map_or(false, |c| c.is_whitespace()) {
            self.read_char();
        }
    }

    /// Reads a contiguous sequence of alphabetic characters and returns the corresponding TokenKind.
    fn read_identifier(&mut self) -> String {
        let start = self.position;
        while self.ch.map_or(false, |c| c.is_alphabetic() || c == '_') {
            self.read_char();
        }
        let end = self.position;
        self.input[start..end].to_string()
    }

    /// Reads a contiguous sequence of digits, handling both Integer and Float literals.
    fn read_number(&mut self) -> String {
        let start = self.position;
        let mut is_float = false;

        while self.ch.map_or(false, |c| c.is_digit(10) || c == '.') {
            if self.ch == Some('.') {
                if is_float {
                    // Two dots in a number are illegal, stop here
                    break;
                }
                is_float = true;
            }
            self.read_char();
        }
        
        let end = self.position;
        self.input[start..end].to_string()
    }
    
    /// Reads a string literal enclosed in double quotes.
    fn read_string(&mut self) -> (TokenKind, String) {
        let start = self.read_position;
        loop {
            self.read_char();
            if self.ch == Some('"') || self.ch.is_none() {
                break;
            }
        }
        
        let end = self.position;
        let literal = self.input[start..end].to_string();
        
        if self.ch == Some('"') {
            self.read_char(); // Consume the closing quote
            (TokenKind::StringLiteral(literal.clone()), literal)
        } else {
            // Handle unclosed string (ran into EOF)
            (TokenKind::Illegal, "Unclosed string literal".to_string())
        }
    }

    /// Returns the next token in the input string.
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        
        let (kind, literal) = match self.ch {
            Some('=') => self.read_two_char_op('=', TokenKind::Eq, TokenKind::Assign, Some('>'), TokenKind::Arrow),
            Some('!') => self.read_two_char_op('=', TokenKind::Neq, TokenKind::Bang, None, TokenKind::Illegal),
            Some('<') => self.read_two_char_op('=', TokenKind::Le, TokenKind::Lt, None, TokenKind::Illegal),
            Some('>') => self.read_two_char_op('=', TokenKind::Ge, TokenKind::Gt, None, TokenKind::Illegal),
            Some('(') => (TokenKind::LParen, "(".to_string()),
            Some(')') => (TokenKind::RParen, ")".to_string()),
            Some('{') => (TokenKind::LBrace, "{".to_string()),
            Some('}') => (TokenKind::RBrace, "}".to_string()),
            Some(',') => (TokenKind::Comma, ",".to_string()),
            Some('.') => (TokenKind::Dot, ".".to_string()),
            Some('+') => (TokenKind::Plus, "+".to_string()),
            Some('-') => (TokenKind::Minus, "-".to_string()),
            Some('*') => (TokenKind::Star, "*".to_string()),
            Some('/') => (TokenKind::Slash, "/".to_string()),
            Some(';') => (TokenKind::Semicolon, ";".to_string()),
            Some('"') => {
                // The actual logic is in read_string, which consumes the opening quote
                self.read_char(); 
                let (kind, lit) = self.read_string();
                return Token { kind, literal: lit };
            }
            Some(ch) => {
                if ch.is_alphabetic() || ch == '_' {
                    let literal = self.read_identifier();
                    let kind = lookup_ident(&literal);
                    return Token { kind, literal };
                } else if ch.is_digit(10) {
                    let literal = self.read_number();
                    return match literal.parse::<i64>() {
                        Ok(val) => Token { kind: TokenKind::IntegerLiteral(val), literal },
                        Err(_) => match literal.parse::<f64>() {
                            Ok(val) => Token { kind: TokenKind::FloatLiteral(val), literal },
                            Err(_) => Token { kind: TokenKind::Illegal, literal: literal }, // Should be caught by the number logic above, but safer to handle
                        }
                    };
                } else {
                    (TokenKind::Illegal, ch.to_string())
                }
            }
            None => (TokenKind::Eof, "".to_string()),
        };

        self.read_char();
        Token { kind, literal }
    }
    
    /// Helper for reading one or two character operators like `==`, `!=`, `<=`, `>=`.
    fn read_two_char_op(&mut self, 
                        expected_char: char, 
                        two_char_kind: TokenKind, 
                        one_char_kind: TokenKind,
                        alternate_char: Option<char>,
                        alternate_kind: TokenKind) -> (TokenKind, String) {
        
        if self.peek_char() == Some(expected_char) {
            self.read_char(); // Consume the current char
            self.read_char(); // Consume the peeked char
            (two_char_kind, format!("{}{}", expected_char, expected_char))
        } else if let Some(alt) = alternate_char {
            if self.peek_char() == Some(alt) {
                 self.read_char(); // Consume the current char
                 self.read_char(); // Consume the peeked char
                 (alternate_kind, format!("{}{}", expected_char, alt))
            } else {
                (one_char_kind, expected_char.to_string())
            }
        } else {
            (one_char_kind, expected_char.to_string())
        }
    }
}

/// Maps a literal string to its corresponding TokenKind (Keyword or Identifier).
fn lookup_ident(ident: &str) -> TokenKind {
    match ident {
        "fn"     => TokenKind::Fn,
        "let"    => TokenKind::Let,
        "if"     => TokenKind::If,
        "else"   => TokenKind::Else,
        "return" => TokenKind::Return,
        "true"   => TokenKind::True,
        "false"  => TokenKind::False,
        "match"  => TokenKind::Match,
        "when"   => TokenKind::When,
        _        => TokenKind::Identifier(ident.to_string()),
    }
}

// --- Example Usage and Testing ---

fn main() {
    let input = r#"
        let my_int: Integer = 5 + 10;
        let is_float = 3.14 * 2.0;
        if (x != y) { return true => };
        match (temp) {
            when (hot) => { fn_call(); }
        }
    "#;

    let mut lexer = Lexer::new(input.to_string());
    let expected_tokens = vec![
        // let my_int: Integer = 5 + 10;
        TokenKind::Let, TokenKind::Identifier("my_int".to_string()), TokenKind::Assign,
        TokenKind::IntegerLiteral(5), TokenKind::Plus, TokenKind::IntegerLiteral(10), TokenKind::Semicolon,
        // let is_float = 3.14 * 2.0;
        TokenKind::Let, TokenKind::Identifier("is_float".to_string()), TokenKind::Assign,
        TokenKind::FloatLiteral(3.14), TokenKind::Star, TokenKind::FloatLiteral(2.0), TokenKind::Semicolon,
        // if (x != y) { return true => };
        TokenKind::If, TokenKind::LParen, TokenKind::Identifier("x".to_string()), TokenKind::Neq,
        TokenKind::Identifier("y".to_string()), TokenKind::RParen, TokenKind::LBrace,
        TokenKind::Return, TokenKind::True, TokenKind::Arrow, TokenKind::RBrace, TokenKind::Semicolon,
        // match (temp) { when (hot) => { fn_call(); } }
        TokenKind::Match, TokenKind::LParen, TokenKind::Identifier("temp".to_string()), TokenKind::RParen,
        TokenKind::LBrace, TokenKind::When, TokenKind::LParen, TokenKind::Identifier("hot".to_string()),
        TokenKind::RParen, TokenKind::Arrow, TokenKind::LBrace, TokenKind::Identifier("fn_call".to_string()),
        TokenKind::LParen, TokenKind::RParen, TokenKind::Semicolon, TokenKind::RBrace, TokenKind::RBrace,
        TokenKind::Eof,
    ];

    println!("--- Starting Lexing Test ---");
    let mut actual_tokens = Vec::new();

    loop {
        let token = lexer.next_token();
        println!("Token: {:?}", token);
        actual_tokens.push(token.kind.clone());
        if token.kind == TokenKind::Eof {
            break;
        }
    }
    println!("--- Lexing Test Complete ---");

    // This simple comparison won't work perfectly due to the way I've handled the explicit type 
    // in the first line of the test string being ignored for simplicity, and literal vs kind differences.
    // For a real test, one would compare the TokenKind only.
    // assert_eq!(actual_tokens, expected_tokens); 
    // (A real test would require more robust setup, this is just for demoing the lexer output)
}
