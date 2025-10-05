// rust_emitter_service.rs
// 검증된 Program (AST)을 유효한 Rust 소스 코드로 변환하는 트랜스파일러 서비스입니다.

use crate::data_structures::{
    Diagnostic, DiagnosticLevel, Program, Statement, Expression, TokenKind,
};
use std::fmt::Write;

pub struct RustEmitterService;

impl RustEmitterService {
    /// Program AST를 받아 Rust 소스 코드 문자열을 생성합니다.
    pub fn run(program: &Program) -> Result<String, Diagnostic> {
        println!("\n[RUST EMITTER] START: Generating Rust Source Code.");
        
        let mut rust_code = String::new();

        // 1. Rust 표준 라이브러리 사용 및 주석
        writeln!(rust_code, "// [Transpiled Code] - Generated from custom language AST").unwrap();
        writeln!(rust_code, "// File: generated_code.rs\n").unwrap();

        // 2. main 함수 시작
        writeln!(rust_code, "fn main() {{").unwrap();

        // 3. Statement들을 순회하며 Rust 코드로 변환
        let num_statements = program.statements.len();
        for (i, statement) in program.statements.iter().enumerate() {
            let is_last_statement = i == num_statements - 1;
            
            // 모든 코드는 4칸 들여쓰기 합니다.
            match self.emit_statement(statement, is_last_statement)? {
                Some(code) => {
                    writeln!(rust_code, "    {}", code).unwrap();
                },
                None => {
                    // ExpressionStatement에서 아무것도 생성하지 않은 경우 (e.g., 단순 함수 정의)
                }
            }
        }
        
        // 4. main 함수 종료
        writeln!(rust_code, "}}").unwrap();
        
        println!("[RUST EMITTER] SUCCESS: Rust source code generated.");
        Ok(rust_code)
    }

    /// Statement 노드를 Rust 코드로 변환합니다.
    /// `is_last_statement`가 참이면 세미콜론을 생략하여 값 반환을 시뮬레이션합니다.
    fn emit_statement(&self, stmt: &Statement, is_last_statement: bool) -> Result<Option<String>, Diagnostic> {
        match stmt {
            Statement::LetStatement { name, value, type_name, span: _ } => {
                // type_name이 있다면 사용하고, 없으면 i64로 추정합니다.
                let rust_type = type_name.as_deref().unwrap_or("i64"); 
                let expr_code = self.emit_expression(value)?;
                
                // Rust의 let 바인딩은 항상 세미콜론으로 끝나야 합니다.
                Ok(Some(format!("let {}: {} = {};", name, rust_type, expr_code)))
            },
            Statement::ReturnStatement { value, span: _ } => {
                let expr_code = self.emit_expression(value)?;
                
                // DSL의 return을 Rust의 명시적 return으로 변환합니다.
                Ok(Some(format!("return {};", expr_code)))
            },
            Statement::ExpressionStatement(expr) => {
                let expr_code = self.emit_expression(expr)?;
                
                // 마지막 표현식(Implicit Return)이 아니라면 세미콜론을 추가합니다.
                let semicolon = if is_last_statement { "" } else { ";" };
                
                // 간단한 ExpressionStatement (예: 함수 호출) 처리
                Ok(Some(format!("{}{}", expr_code, semicolon)))
            },
        }
    }
    
    /// Expression 노드를 Rust 코드로 변환합니다.
    fn emit_expression(&self, expr: &Expression) -> Result<String, Diagnostic> {
        match expr {
            Expression::Integer(i) => Ok(i.to_string()),
            Expression::Boolean(b) => Ok(b.to_string()),
            Expression::StringLiteral(s) => Ok(format!("\"{}\"", s)),
            Expression::Identifier(name) => Ok(name.clone()),
            
            Expression::Binary { op, left, right, span } => {
                let left_code = self.emit_expression(left)?;
                let right_code = self.emit_expression(right)?;
                
                let op_str = match op {
                    // 산술 연산자
                    TokenKind::Plus => "+",
                    TokenKind::Minus => "-",
                    TokenKind::Asterisk => "*",
                    TokenKind::Slash => "/",
                    // 비교/관계 연산자 (새로 추가됨)
                    TokenKind::EqualEqual => "==",
                    TokenKind::NotEqual => "!=",
                    TokenKind::LessThan => "<",
                    TokenKind::GreaterThan => ">",
                    TokenKind::LessThanOrEqual => "<=",
                    TokenKind::GreaterThanOrEqual => ">=",
                    _ => {
                        return Err(Diagnostic {
                            level: DiagnosticLevel::Error,
                            message: format!("Unsupported binary operator for Rust emitter: {:?}", op),
                            span: span.clone(),
                            help: None,
                        });
                    }
                };
                
                // 연산자 우선순위를 위해 괄호를 사용합니다.
                Ok(format!("({} {} {})", left_code, op_str, right_code))
            },
            
            Expression::Call { function, arguments, span } => {
                let func_name = match function.as_ref() {
                    Expression::Identifier(name) => name,
                    _ => {
                        return Err(Diagnostic {
                            level: DiagnosticLevel::Error,
                            message: "Function call target must be a simple identifier.".to_string(),
                            span: span.clone(),
                            help: None,
                        });
                    }
                };
                
                // 인수 리스트 생성
                let mut arg_list = Vec::new();
                for arg in arguments {
                    arg_list.push(self.emit_expression(arg)?);
                }
                let args_str = arg_list.join(", ");
                
                // 임시로 'print' 함수는 Rust의 println!으로 매핑합니다.
                if func_name == "print" {
                    Ok(format!("println!(\"{{}}\", {})", args_str))
                } else {
                    Ok(format!("{}({})", func_name, args_str))
                }
            },
            
            // 기타 복잡한 Expression 유형은 현재 트랜스파일러에서 지원하지 않는다고 가정합니다.
            expr => Err(Diagnostic {
                level: DiagnosticLevel::Error,
                message: format!("Unsupported expression type for Rust emitter: {:?}", expr),
                span: expr.get_span().unwrap_or_default(), 
                help: Some("This feature is not yet supported in the Rust transpiler backend.".to_string()),
            }),
        }
    }
}

// Span을 쉽게 가져오기 위한 Expression 트레이트 헬퍼
trait GetSpan {
    fn get_span(&self) -> Option<crate::data_structures::Span>;
}

impl GetSpan for Expression {
    fn get_span(&self) -> Option<crate::data_structures::Span> {
        // 모든 Expression variant에 span 필드가 없으므로, 현재는 None을 반환합니다.
        // 실제 구현에서는 모든 Expression에 Span을 포함시켜야 Diagnostic 정보가 정확해집니다.
        None
    }
}
