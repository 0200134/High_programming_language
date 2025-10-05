use crate::data_structures::{
    Program, Statement, Expression, Value, TokenKind, Span,
};

pub struct Optimizer;

impl Optimizer {
    pub fn optimize(program: &mut Program) {
        for stmt in program.statements.iter_mut() {
            Self::optimize_statement(stmt);
        }
    }

    fn optimize_statement(stmt: &mut Box<Statement>) {
    match stmt.as_mut() {
        Statement::ExpressionStatement(expr) => {
            Self::optimize_expression(expr);
        }
        Statement::LetStatement { value, .. } => {
            Self::optimize_expression(value);
        }
        Statement::ReturnStatement(expr) => {
            Self::optimize_expression(expr);
        }
        Statement::IfStatement { condition, then_branch, else_branch } => {
            Self::optimize_expression(condition);
            Self::optimize_statement(then_branch);
            if let Some(else_stmt) = else_branch {
                Self::optimize_statement(else_stmt);
            }
        }
        Statement::BlockStatement { statements, .. } => {
            for s in statements.iter_mut() {
                Self::optimize_statement(s);
            }
        }
        Statement::ForStatement { initializer, condition, increment, body } => {
            if let Some(init) = initializer {
                Self::optimize_statement(init);
            }
            if let Some(cond) = condition {
                Self::optimize_expression(cond);
            }
            if let Some(inc) = increment {
                Self::optimize_expression(inc);
            }
            Self::optimize_statement(body);
        }
        Statement::WhileStatement { condition, body } => {
            Self::optimize_expression(condition);
            Self::optimize_statement(body);
        }
        Statement::MacroDefinition { .. } => {
            // 매크로 정의는 확장기에서 처리
        }
    }
}


    fn optimize_expression(expr: &mut Box<Expression>) {
        match expr.as_mut() {
            Expression::InfixOperation(span, op, left, right) => {
                Self::optimize_expression(left);
                Self::optimize_expression(right);

                if let (Expression::Literal(_, l), Expression::Literal(_, r)) = (&**left, &**right) {
                    if let Some(val) = Self::fold_constants(op, l, r) {
                        *expr = Box::new(Expression::Literal(*span, val));
                    }
                }
            }
            Expression::Grouped(span, inner) => {
                Self::optimize_expression(inner);
                if let Expression::Literal(_, val) = &**inner {
                    *expr = Box::new(Expression::Literal(*span, val.clone()));
                }
            }
            Expression::Ternary(_, cond, then_expr, else_expr) => {
                Self::optimize_expression(cond);
                Self::optimize_expression(then_expr);
                Self::optimize_expression(else_expr);

                if let Expression::Literal(_, Value::Boolean(b)) = &**cond {
                    *expr = if *b {
                        then_expr.clone()
                    } else {
                        else_expr.clone()
                    };
                }
            }
            Expression::Call(_, func, args) => {
                Self::optimize_expression(func);
                for arg in args.iter_mut() {
                    Self::optimize_expression(arg);
                }
            }
            Expression::Reflect(_, inner)
            | Expression::Eval(_, inner)
            | Expression::TypeOf(_, inner) => {
                Self::optimize_expression(inner);
            }
            Expression::MacroCall(_, _, args) => {
                for arg in args.iter_mut() {
                    Self::optimize_expression(arg);
                }
            }
            _ => {}
        }
    }

    fn fold_constants(op: &TokenKind, left: &Value, right: &Value) -> Option<Value> {
        match (op, left, right) {
            // ─── 산술 ─────────────────────────────
            (TokenKind::Plus, Value::Integer(a), Value::Integer(b)) => Some(Value::Integer(a + b)),
            (TokenKind::Minus, Value::Integer(a), Value::Integer(b)) => Some(Value::Integer(a - b)),
            (TokenKind::Asterisk, Value::Integer(a), Value::Integer(b)) => Some(Value::Integer(a * b)),
            (TokenKind::Slash, Value::Integer(a), Value::Integer(b)) if *b != 0 => Some(Value::Integer(a / b)),

            (TokenKind::Plus, Value::Float(a), Value::Float(b)) => Some(Value::Float(a + b)),
            (TokenKind::Minus, Value::Float(a), Value::Float(b)) => Some(Value::Float(a - b)),
            (TokenKind::Asterisk, Value::Float(a), Value::Float(b)) => Some(Value::Float(a * b)),
            (TokenKind::Slash, Value::Float(a), Value::Float(b)) if *b != 0.0 => Some(Value::Float(a / b)),

            // ─── 비교 ─────────────────────────────
            (TokenKind::Eq, Value::Integer(a), Value::Integer(b)) => Some(Value::Boolean(a == b)),
            (TokenKind::Neq, Value::Integer(a), Value::Integer(b)) => Some(Value::Boolean(a != b)),
            (TokenKind::Less, Value::Integer(a), Value::Integer(b)) => Some(Value::Boolean(a < b)),
            (TokenKind::Greater, Value::Integer(a), Value::Integer(b)) => Some(Value::Boolean(a > b)),
            (TokenKind::LessEqual, Value::Integer(a), Value::Integer(b)) => Some(Value::Boolean(a <= b)),
            (TokenKind::GreaterEqual, Value::Integer(a), Value::Integer(b)) => Some(Value::Boolean(a >= b)),

            (TokenKind::Eq, Value::Float(a), Value::Float(b)) => Some(Value::Boolean(a == b)),
            (TokenKind::Neq, Value::Float(a), Value::Float(b)) => Some(Value::Boolean(a != b)),
            (TokenKind::Less, Value::Float(a), Value::Float(b)) => Some(Value::Boolean(a < b)),
            (TokenKind::Greater, Value::Float(a), Value::Float(b)) => Some(Value::Boolean(a > b)),
            (TokenKind::LessEqual, Value::Float(a), Value::Float(b)) => Some(Value::Boolean(a <= b)),
            (TokenKind::GreaterEqual, Value::Float(a), Value::Float(b)) => Some(Value::Boolean(a >= b)),

            _ => None
        }
    }
}
