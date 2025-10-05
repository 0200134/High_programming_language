// src/type_checker.rs

use crate::data_structures::{
    Expression, HighType, Statement, TokenKind, Program,
};
use std::collections::HashMap;

/// 타입 검사 중 변수의 타입을 저장하는 환경입니다.
#[derive(Debug, Clone)]
pub struct TypeEnv {
    store: HashMap<String, HighType>,
}

impl TypeEnv {
    pub fn new() -> Self {
        TypeEnv { store: HashMap::new() }
    }

    /// 변수 이름을 환경에 추가하고 해당 타입을 저장합니다.
    pub fn set(&mut self, name: String, t: HighType) {
        self.store.insert(name, t);
    }

    /// 환경에서 변수의 타입을 조회합니다.
    pub fn get(&self, name: &str) -> Option<&HighType> {
        self.store.get(name)
    }
}

/// AST의 타입 검사 및 타입 추론을 담당하는 서비스입니다.
pub struct TypeChecker {
    env: TypeEnv,
    errors: Vec<String>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            env: TypeEnv::new(),
            errors: Vec::new(),
        }
    }

    /// 프로그램 전체의 타입 검사를 시작하고, AST에 추론된 타입을 채우며 오류를 반환합니다.
    pub fn check_program(&mut self, program: &mut Program) {
        for stmt in program.statements.iter_mut() {
            match self.check_statement(stmt) {
                Ok(_) => {},
                Err(e) => self.errors.push(e),
            }
        }
        program.type_errors.append(&mut self.errors);
    }

    /// 문장의 타입을 검사하고 환경을 업데이트합니다.
    fn check_statement(&mut self, stmt: &mut Statement) -> Result<(), String> {
        match stmt {
            Statement::LetStatement { name, value, final_type, .. } => {
                // 1. 값 표현식의 타입을 추론합니다.
                let inferred_type = self.check_expression(value)?;

                // 2. 환경에 변수 이름과 추론된 타입을 등록합니다.
                self.env.set(name.clone(), inferred_type.clone());
                
                // 3. AST의 final_type 필드를 업데이트합니다.
                *final_type = inferred_type;
                
                Ok(())
            },
            Statement::ExpressionStatement(expr) => {
                // 표현식 문장의 타입은 Unit이 될 수 있지만, 여기서는 그냥 검사만 수행합니다.
                self.check_expression(expr)?;
                Ok(())
            }
        }
    }

    /// 표현식의 타입을 재귀적으로 검사하고 추론된 타입을 반환합니다.
    fn check_expression(&mut self, expr: &mut Expression) -> Result<HighType, String> {
        let result_type = match expr {
            Expression::Identifier(name) => {
                match self.env.get(name) {
                    Some(t) => t.clone(),
                    None => return Err(format!("미정의 변수: '{}'", name)),
                }
            }
            Expression::Integer(_) => HighType::Int,
            Expression::Float(_) => HighType::Float,
            Expression::Boolean(_) => HighType::Bool,

            // Prefix/Binary 연산자
            Expression::PrefixOp { operator, right, inferred_type } => {
                let right_t = self.check_expression(right)?;
                let op = operator.clone();
                let t = match op {
                    // ! (NOT)은 Bool에만 적용되고 Bool을 반환해야 합니다.
                    TokenKind::Bang if right_t == HighType::Bool => HighType::Bool,
                    // - (단항 마이너스)는 Int 또는 Float에 적용되고 같은 타입을 반환해야 합니다.
                    TokenKind::Minus if right_t == HighType::Int => HighType::Int,
                    TokenKind::Minus if right_t == HighType::Float => HighType::Float,
                    _ => return Err(format!("{} 연산자를 타입 {:?}에 사용할 수 없습니다.", op.to_string(), right_t)),
                };
                *inferred_type = t.clone();
                t
            }
            Expression::BinaryOp { operator, left, right, inferred_type } => {
                let left_t = self.check_expression(left)?;
                let right_t = self.check_expression(right)?;
                let op = operator.clone();
                
                let t = match op {
                    // 비교 연산자 (==, !=, <, >)는 Bool을 반환해야 합니다.
                    _ if op.is_comparison_op() => {
                        if left_t != right_t {
                            return Err(format!("비교 연산자 {:?}에서 타입 불일치: {:?}와 {:?}", op, left_t, right_t));
                        }
                        // Int == Int -> Bool, Float == Float -> Bool
                        if left_t == HighType::Int || left_t == HighType::Float {
                            HighType::Bool
                        } else {
                            return Err(format!("비교 연산자 {:?}는 타입 {:?}에 적용할 수 없습니다.", op, left_t));
                        }
                    }
                    // 산술 연산자 (+, -, *, /)는 피연산자와 같은 타입을 반환해야 합니다.
                    _ if op.is_arithmetic_op() => {
                        if left_t != right_t {
                            return Err(format!("산술 연산자 {:?}에서 타입 불일치: {:?}와 {:?}", op, left_t, right_t));
                        }
                        // Int + Int -> Int, Float + Float -> Float
                        if left_t == HighType::Int || left_t == HighType::Float {
                            left_t
                        } else {
                            return Err(format!("산술 연산자 {:?}는 타입 {:?}에 적용할 수 없습니다.", op, left_t));
                        }
                    }
                    _ => HighType::Error,
                };
                *inferred_type = t.clone();
                t
            }
            
            // If/Else 표현식
            Expression::IfExpression { condition, consequence, alternative, inferred_type, .. } => {
                // 1. 조건은 Bool 타입이어야 합니다.
                let condition_t = self.check_expression(condition)?;
                if condition_t != HighType::Bool {
                    return Err(format!("If 조건에 Bool 대신 {:?} 타입이 사용되었습니다.", condition_t));
                }

                // 2. 결과와 대체 블록의 타입을 추론합니다.
                let consequence_t = self.check_expression(consequence)?;
                
                let t = if let Some(alt) = alternative {
                    let alt_t = self.check_expression(alt)?;
                    
                    // 3. 두 브랜치의 타입이 일치해야 합니다. (If/Else는 표현식이므로)
                    if consequence_t != alt_t {
                        return Err(format!("If/Else 브랜치의 타입 불일치: 참 브랜치 {:?}, 거짓 브랜치 {:?}", consequence_t, alt_t));
                    }
                    consequence_t
                } else {
                    // 4. else가 없으면 Unit 타입으로 추론됩니다.
                    HighType::Unit
                };

                *inferred_type = t.clone();
                t
            }

            // 블록 표현식
            Expression::BlockExpression(statements, inferred_type) => {
                // 블록 내부에 새로운 스코프를 만들어야 하지만, 여기서는 단순화를 위해 전역 환경을 사용합니다.
                // 블록의 타입은 마지막 문장의 타입으로 결정됩니다.
                let mut last_type = HighType::Unit;
                
                // 마지막 문장을 제외한 모든 문장은 Unit 타입으로 간주됩니다.
                for stmt in statements.iter_mut() {
                    self.check_statement(stmt)?;
                    // 마지막 문장의 타입은 ExpressionStatement 내부의 표현식 타입입니다.
                    if let Statement::ExpressionStatement(expr) = stmt {
                        last_type = self.check_expression(expr)?;
                    } else if let Statement::LetStatement { final_type, .. } = stmt {
                        last_type = final_type.clone(); // let 바인딩 자체는 Unit으로 간주 가능하나, 여기선 추적된 타입을 사용
                    }
                }
                
                // 블록 내부의 모든 문장이 끝나고 마지막 표현식의 타입이 블록의 타입이 됩니다.
                // 마지막 문장이 ExpressionStatement가 아니면 (예: let x = 5;) Unit을 반환합니다.
                // 이 예시에서는 모든 문장을 처리했으므로, last_type이 이미 마지막 문장의 결과 타입입니다.

                *inferred_type = last_type.clone();
                last_type
            }

            // 함수 리터럴
            Expression::FunctionLiteral { parameters, body, inferred_type, .. } => {
                // 단순화를 위해 파라미터는 모두 Int 타입으로 가정하고 바디를 검사합니다.
                // 실제로는 타입을 AST에서 읽어와야 하지만, 현재 문법에는 타입 명시가 없습니다.
                
                let param_types: Vec<HighType> = parameters.iter().map(|_| HighType::Int).collect();
                
                // 바디 검사를 위해 임시 환경 생성 (스코핑)
                let mut body_env = self.env.clone(); 
                for (name, t) in parameters.iter().zip(param_types.iter()) {
                    body_env.set(name.clone(), t.clone());
                }

                // 바디의 타입을 추론합니다.
                let body_t = self.check_expression(body)?;
                
                // 함수 타입을 구성합니다.
                let func_t = HighType::Function(param_types, Box::new(body_t.clone()));
                *inferred_type = func_t.clone();
                
                func_t
            }

            // 함수 호출
            Expression::CallExpression { function, arguments, inferred_type, .. } => {
                // 1. 함수 표현식의 타입을 추론합니다.
                let func_t = self.check_expression(function)?;

                let (expected_param_ts, return_t) = match func_t {
                    HighType::Function(param_ts, ret_t) => (param_ts, ret_t),
                    _ => return Err(format!("호출할 수 없는 타입 {:?}이(가) 사용되었습니다.", func_t)),
                };

                // 2. 인자의 개수를 확인합니다.
                if arguments.len() != expected_param_ts.len() {
                    return Err(format!("기대 인자 수: {}, 제공된 인자 수: {}", expected_param_ts.len(), arguments.len()));
                }

                // 3. 인자의 타입을 검사하고 기대 타입과 일치하는지 확인합니다.
                for (i, (arg_expr, expected_t)) in arguments.iter_mut().zip(expected_param_ts.iter()).enumerate() {
                    let actual_t = self.check_expression(arg_expr)?;
                    if &actual_t != expected_t {
                        return Err(format!("{}.번째 인자 타입 불일치: 기대 {:?}, 실제 {:?}", i, expected_t, actual_t));
                    }
                }
                
                // 4. 호출 결과는 함수의 반환 타입입니다.
                *inferred_type = (*return_t).clone();
                (*return_t).clone()
            }
            
            // MatchExpression은 복잡하므로 단순화를 위해 여기서는 Unhandled로 처리
            Expression::MatchExpression { .. } => HighType::Unknown,
            Expression::NoOp => HighType::Unit,
        };

        Ok(result_type)
    }
}
