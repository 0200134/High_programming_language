use std::fmt;

//
// ─── 런타임 값 ────────────────────────────────────────────────────────────────
//

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Function(Box<FunctionValue>),
    Null,
    Return(Box<Value>),
    Error(String),
    Reflection(ReflectionInfo),
    Macro(String), // 매크로 이름 또는 본문
    Type(String),  // 런타임 타입 표현
}

#[derive(Debug, Clone)]
pub struct FunctionValue {
    pub parameters: Vec<String>,
    pub body: Statement,
}

#[derive(Debug, Clone)]
pub struct ReflectionInfo {
    pub type_name: String,
    pub details: String,
}

//
// ─── 타입 시스템 ─────────────────────────────────────────────────────────────
//

#[derive(Debug, Clone)]
pub enum TypeAnnotation {
    Int,
    Float,
    Bool,
    String,
    Void,
    Any,
    Custom(String),
    Infer,
}

//
// ─── 토큰 ─────────────────────────────────────────────────────────────────────
//

#[derive(Debug, Clone)]


pub enum TokenKind {
    // ─── 리터럴 ─────────────────────────────
    IntegerLiteral(i64),
    FloatLiteral(String),
    StringLiteral(String),
    BooleanLiteral(bool),

    // ─── 식별자 ─────────────────────────────
    Identifier(String),

    // ─── 키워드 ─────────────────────────────
    Fn,
    Let,
    Mut,
    If,
    Else,
    While,
    For,
    Return,
    Match,
    Macro,
    TypeOf,
    Eval,
    Reflect,
    Async,
    Await,
    True,
    False,

    // ─── 타입 키워드 ────────────────────────
    Int,
    Float,
    Bool,
    String,
    Void,
    Any,

    // ─── 산술 연산자 ────────────────────────
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,

    // ─── 비교 연산자 ────────────────────────
    Eq,
    Neq,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,

    // ─── 논리 연산자 ────────────────────────
    And,
    Or,
    Bang,

    // ─── 비트 연산자 ────────────────────────
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,

    // ─── 대입 연산자 ────────────────────────
    Assign,
    PlusAssign,
    MinusAssign,

    // ─── 삼항 연산자 ────────────────────────
    Question,
    Colon,

    // ─── 구문 기호 ──────────────────────────
    Comma,
    Semicolon,
    Dot,
    Arrow,

    // ─── 괄호 ───────────────────────────────
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    // ─── 기타 ───────────────────────────────
    Eof,
    Illegal(char),
}


#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

//
// ─── 표현식 ───────────────────────────────────────────────────────────────────
//

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Span, Value),
    Identifier(Span, String),
    PrefixOperation(Span, TokenKind, Box<Expression>),
    InfixOperation(Span, TokenKind, Box<Expression>, Box<Expression>),
    Ternary(Span, Box<Expression>, Box<Expression>, Box<Expression>),
    Function(Span, Vec<String>, Box<Statement>),
    Call(Span, Box<Expression>, Vec<Box<Expression>>),
    Grouped(Span, Box<Expression>),
    Reflect(Span, Box<Expression>),
    Eval(Span, Box<Expression>),
    TypeOf(Span, Box<Expression>),
    MacroCall(Span, String, Vec<Box<Expression>>),
}

//
// ─── 문장 ─────────────────────────────────────────────────────────────────────
//

#[derive(Debug, Clone)]
pub enum Statement {
    ExpressionStatement(Box<Expression>),
    LetStatement {
        name: String,
        value: Box<Expression>,
        type_annotation: Option<TypeAnnotation>,
        is_mutable: bool,
    },
    ReturnStatement(Box<Expression>),
    BlockStatement {
        statements: Vec<Box<Statement>>,
        span: Span,
    },
    IfStatement {
        condition: Box<Expression>,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    WhileStatement {
        condition: Box<Expression>,
        body: Box<Statement>,
    },
    ForStatement {
        initializer: Option<Box<Statement>>,
        condition: Option<Box<Expression>>,
        increment: Option<Box<Expression>>,
        body: Box<Statement>,
    },
    MacroDefinition {
        name: String,
        parameters: Vec<String>,
        body: Box<Statement>,
    },
}

//
// ─── 프로그램 ─────────────────────────────────────────────────────────────────
//

#[derive(Debug, Clone)]
pub struct Program {
    pub root_id: usize,
    pub statements: Vec<Box<Statement>>,
    pub span: Span,
}

//
// ─── 진단 ─────────────────────────────────────────────────────────────────────
//

#[derive(Debug, Clone)]
pub enum DiagnosticLevel {
    Info,
    Warning,
    Error,
    HerFatal,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub span: Span,
    pub help: Option<String>,
}
