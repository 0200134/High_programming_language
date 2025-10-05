use crate::data_structures::{Expression, Statement, TokenKind, Program, Value};
use std::collections::HashMap;

/// 런타임 변수 저장소 및 스코프 관리
#[derive(Debug, Clone)]
pub struct Environment {
    store: HashMap<String, Value>,
    outer: Option<Box<Environment>>, // 클로저 구현을 위한 외부 스코프
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
            outer: None,
        }
    }

    /// 함수 호출 시 사용할 외부 환경을 가진 새 환경을 생성합니다.
    /// (Closure의 캡처된 환경을 외부 스코프로 사용)
    pub fn new_enclosed(outer: Environment) -> Self {
        Environment {
            store: HashMap::new(),
            outer: Some(Box::new(outer)),
        }
    }

    /// 환경에 값을 바인딩합니다.
    pub fn set(&mut self, name: String, val: Value) {
        self.store.insert(name, val);
    }

    /// 환경에서 값을 찾습니다. 현재 스코프에 없으면 외부 스코프를 확인합니다.
    pub fn get(&self, name: &str) -> Option<&Value> {
        if let Some(val) = self.store.get(name) {
            Some(val)
        } else if let Some(outer) = &self.outer {
            outer.get(name)
        } else {
            None
        }
    }
}

/// AST를 순회하며 코드를 실행하는 인터프리터입니다.
pub struct Evaluator {
    env: Environment, // 최상위(글로벌) 환경
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator { env: Environment::new() }
    }

    /// 프로그램 전체를 실행하고 마지막 결과 값을 반환합니다.
    pub fn eval_program(&mut self, program: &Program) -> Value {
        let mut last_result = Value::Unit;

        // 최상위 스코프(self.env)에서 문장들을 실행하며 바인딩을 업데이트합니다.
        for stmt in program.statements.iter() {
            match self.eval_statement(stmt) {
                val @ Value::Error(_) => return val, // 오류 발생 시 즉시 중단
                val => last_result = val,
            }
        }
        last_result
    }

    /// 문장을 평가합니다.
    /// 이 함수는 최상위 레벨에서 호출되며, `self.env`에 바인딩합니다.
    fn eval_statement(&mut self, stmt: &Statement) -> Value {
        match stmt {
            Statement::LetStatement { name, value, .. } => {
                // Let 문장의 값은 현재 환경(self.env)을 읽기 전용으로 사용하여 평가됩니다.
                let val = self.eval_expression_with_env(value, &self.env); 
                
                match val {
                    Value::Error(_) => val,
                    _ => {
                        // 평가가 성공하면 self.env에 값을 바인딩합니다. (가변성 보장)
                        self.env.set(name.clone(), val); 
                        Value::Unit
                    }
                }
            }
            Statement::ExpressionStatement(expr) => {
                // 표현식은 현재 환경(self.env)을 읽기 전용으로 사용하여 평가됩니다.
                self.eval_expression_with_env(expr, &self.env)
            }
        }
    }

    /// 표현식을 평가합니다. (환경은 읽기 전용 `&Environment`로 전달)
    fn eval_expression_with_env(&mut self, expr: &Expression, env: &Environment) -> Value {
        match expr {
            Expression::Identifier(name) => {
                match env.get(name) {
                    Some(val) => val.clone(),
                    None => Value::Error(format!("미정의 식별자: {}", name)),
                }
            }
            Expression::Integer(v) => Value::Integer(*v),
            Expression::Boolean(v) => Value::Boolean(*v),
            
            Expression::PrefixOp { operator, right, .. } => {
                let right_val = self.eval_expression_with_env(right, env);
                self.eval_prefix_op(operator.clone(), right_val)
            }
            
            Expression::BinaryOp { operator, left, right, .. } => {
                let left_val = self.eval_expression_with_env(left, env);
                let right_val = self.eval_expression_with_env(right, env);
                self.eval_binary_op(operator.clone(), left_val, right_val)
            }

            Expression::IfExpression { condition, consequence, alternative, .. } => {
                let condition_val = self.eval_expression_with_env(condition, env);
                
                match condition_val {
                    Value::Boolean(true) => self.eval_block_statements(consequence, env),
                    Value::Boolean(false) => {
                        if let Some(alt) = alternative {
                            self.eval_block_statements(alt, env)
                        } else {
                            Value::Unit
                        }
                    }
                    val => Value::Error(format!("조건이 Bool 타입이 아닙니다: {:?}", val)),
                }
            }
            
            Expression::BlockExpression(statements, _) => {
                // BlockExpression은 자체 스코프를 생성하여 평가됩니다.
                self.eval_block_statements(statements, env)
            }
            
            // 함수 정의: 클로저를 생성합니다. (현재 환경을 캡처합니다.)
            Expression::FunctionLiteral { parameters, body, .. } => {
                Value::Function {
                    parameters: parameters.clone(),
                    body: body.clone(),
                    env: env.clone(), // 현재 스코프(클로저) 캡처
                }
            }
            
            // 함수 호출
            Expression::CallExpression { function, arguments, .. } => {
                // 1. 함수 자체를 평가하여 Value::Function을 얻습니다.
                let function_val = self.eval_expression_with_env(function, env);
                
                match function_val {
                    Value::Function { parameters, body, env: fn_env } => {
                        // 2. 인자들을 평가합니다.
                        let args = self.eval_expressions(arguments, env);
                        if args.iter().any(|v| matches!(v, Value::Error(_))) {
                            return args.into_iter().find(|v| matches!(v, Value::Error(_))).unwrap().clone();
                        }

                        // 3. 인자 개수 확인
                        if parameters.len() != args.len() {
                            return Value::Error(format!("함수 호출 인자 개수 불일치: {}개 예상, {}개 제공", parameters.len(), args.len()));
                        }

                        // 4. 함수 실행을 위한 새 환경을 설정하고 파라미터와 인자를 바인딩합니다.
                        let mut call_env = Environment::new_enclosed(fn_env); 
                        
                        for (param_name, arg_val) in parameters.iter().zip(args.into_iter()) {
                            call_env.set(param_name.clone(), arg_val);
                        }

                        // 5. 함수 바디(BlockExpression)를 실행합니다.
                        self.eval_block_statements(&body, &call_env)
                    }
                    val => Value::Error(format!("호출할 수 없는 값입니다: {:?}", val)),
                }
            }

            Expression::NoOp => Value::Unit,
            _ => Value::Error(format!("지원되지 않는 표현식입니다: {:?}", expr)),
        }
    }

    /// 문장 목록을 실행하고 마지막 문장의 결과 값을 반환합니다. (블록 스코프 생성)
    fn eval_block_statements(&mut self, statements: &Vec<Statement>, env: &Environment) -> Value {
        // 현재 환경을 외부 스코프로 하는 새로운 스코프를 생성합니다. (블록 스코프)
        let mut current_env = Environment::new_enclosed(env.clone()); 
        let mut last_result = Value::Unit;

        // 블록 내부에서 LetStatement는 current_env에만 바인딩됩니다.
        for stmt in statements.iter() {
            let stmt_result = match stmt {
                Statement::LetStatement { name, value, .. } => {
                    // 값은 현재 블록 스코프(current_env)를 읽기 전용으로 사용하여 평가됩니다.
                    let val = self.eval_expression_with_env(value, &current_env);
                    match val {
                        Value::Error(_) => val,
                        _ => {
                            current_env.set(name.clone(), val);
                            Value::Unit
                        }
                    }
                }
                Statement::ExpressionStatement(expr) => {
                    self.eval_expression_with_env(expr, &current_env)
                }
            };

            match stmt_result {
                val @ Value::Error(_) => return val, // 오류 발생 시 즉시 중단
                val => last_result = val,
            }
        }
        
        last_result
    }

    /// 표현식 리스트를 평가하고 값 리스트를 반환합니다. (함수 인자 평가용)
    fn eval_expressions(&mut self, expressions: &Vec<Expression>, env: &Environment) -> Vec<Value> {
        let mut result = Vec::new();
        for expr in expressions {
            // AST는 &Expression으로 전달됩니다.
            result.push(self.eval_expression_with_env(expr, env)); 
        }
        result
    }
    
    /// 전위 연산자 실행
    fn eval_prefix_op(&self, op: TokenKind, right: Value) -> Value {
        match op {
            TokenKind::Bang => match right {
                Value::Boolean(b) => Value::Boolean(!b),
                Value::Error(e) => Value::Error(e),
                _ => Value::Error(format!("! 연산자는 Bool에만 적용 가능합니다. {:?}", right)),
            },
            TokenKind::Minus => match right {
                Value::Integer(i) => Value::Integer(-i),
                Value::Error(e) => Value::Error(e),
                _ => Value::Error(format!("- 연산자는 Int에만 적용 가능합니다. {:?}", right)),
            },
            _ => Value::Error(format!("알 수 없는 전위 연산자입니다: {:?}", op)),
        }
    }

    /// 중위 연산자 실행
    fn eval_binary_op(&self, op: TokenKind, left: Value, right: Value) -> Value {
        match (&left, &right) {
            (Value::Integer(l), Value::Integer(r)) => self.eval_integer_binary_op(op, *l, *r),
            (Value::Boolean(l), Value::Boolean(r)) => self.eval_boolean_binary_op(op, *l, *r),
            (Value::Error(e), _) => Value::Error(e.clone()),
            (_, Value::Error(e)) => Value::Error(e.clone()),
            _ => Value::Error(format!("지원되지 않는 이진 연산: {:?} {:?} {:?}", left, op, right)),
        }
    }

    /// 정수 간의 이진 연산 실행
    fn eval_integer_binary_op(&self, op: TokenKind, left: i64, right: i64) -> Value {
        match op {
            // 산술 연산
            TokenKind::Plus => Value::Integer(left + right),
            TokenKind::Minus => Value::Integer(left - right),
            TokenKind::Asterisk => Value::Integer(left * right),
            TokenKind::Slash => {
                if right == 0 {
                    Value::Error("0으로 나눌 수 없습니다.".to_string())
                } else {
                    Value::Integer(left / right)
                }
            }
            // 비교 연산
            TokenKind::Eq => Value::Boolean(left == right),
            TokenKind::Neq => Value::Boolean(left != right),
            TokenKind::Lt => Value::Boolean(left < right),
            TokenKind::Gt => Value::Boolean(left > right),
            _ => Value::Error(format!("알 수 없는 정수 연산자: {:?}", op)),
        }
    }
    
    /// 부울 간의 이진 연산 실행 (==, !=만 지원)
    fn eval_boolean_binary_op(&self, op: TokenKind, left: bool, right: bool) -> Value {
        match op {
            TokenKind::Eq => Value::Boolean(left == right),
            TokenKind::Neq => Value::Boolean(left != right),
            _ => Value::Error(format!("알 수 없는 부울 연산자: {:?}", op)),
        }
    }
}
