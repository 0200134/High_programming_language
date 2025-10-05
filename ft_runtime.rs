use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::data_structures::{
    Program, Value, Diagnostic, DiagnosticLevel, Statement, Expression, Span, ReflectionInfo,
};

use crate::lexer_service::LexerService;
use crate::parser_service::ParserService;

pub type ValueStore = HashMap<String, Value>;

#[derive(Debug, Clone)]
pub struct Environment {
    pub store: ValueStore,
    pub outer: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self { store: HashMap::new(), outer: None }
    }

    pub fn new_enclosed(outer: Rc<RefCell<Environment>>) -> Self {
        Self { store: HashMap::new(), outer: Some(outer) }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.store.get(name).cloned().or_else(|| {
            self.outer.as_ref()?.borrow().get(name)
        })
    }

    pub fn set(&mut self, name: String, val: Value) {
        self.store.insert(name, val);
    }
}

pub struct HighEnduranceRuntime {
    pub environment: Rc<RefCell<Environment>>,
    pub output: Vec<String>,
}

impl HighEnduranceRuntime {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new())),
            output: Vec::new(),
        }
    }

    pub fn execute_program(&mut self, program: Program) -> Diagnostic {
        let mut executed_count = 0;

        for statement in program.statements.iter() {
            match statement.as_ref() {
                Statement::ExpressionStatement(expr) => {
                    let val = self.evaluate_expression(expr);
                    self.output.push(format!("Expression result: {:?}", val));
                    executed_count += 1;
                }
                Statement::LetStatement { name, value, .. } => {
                    let val = self.evaluate_expression(value);
                    self.environment.borrow_mut().set(name.clone(), val);
                    self.output.push(format!("Variable '{}' bound", name));
                    executed_count += 1;
                }
                Statement::ReturnStatement(expr) => {
                    let val = self.evaluate_expression(expr);
                    self.output.push(format!("Return value: {:?}", val));
                    executed_count += 1;
                }
                Statement::BlockStatement { statements, .. } => {
                    self.output.push("Entering block scope.".to_string());
                    let enclosed = Rc::new(RefCell::new(Environment::new_enclosed(self.environment.clone())));
                    let mut block_rt = HighEnduranceRuntime {
                        environment: enclosed,
                        output: Vec::new(),
                    };
                    let block_prog = Program {
                        root_id: 0,
                        statements: statements.clone(),
                        span: program.span,
                    };
                    let diag = block_rt.execute_program(block_prog);
                    self.output.extend(block_rt.output);
                    executed_count += 1;

                    if matches!(diag.level, DiagnosticLevel::HerFatal | DiagnosticLevel::Error) {
                        return diag;
                    }
                }
                Statement::IfStatement { condition, then_branch, else_branch } => {
                    let cond_val = self.evaluate_expression(condition);
                    if matches!(cond_val, Value::Boolean(true)) {
                        let _ = self.execute_program(Program {
                            root_id: 0,
                            statements: vec![then_branch.clone()],
                            span: program.span,
                        });
                    } else if let Some(else_stmt) = else_branch {
                        let _ = self.execute_program(Program {
                            root_id: 0,
                            statements: vec![else_stmt.clone()],
                            span: program.span,
                        });
                    }
                    executed_count += 1;
                }
                Statement::WhileStatement { condition, body } => {
                    while matches!(self.evaluate_expression(condition), Value::Boolean(true)) {
                        let _ = self.execute_program(Program {
                            root_id: 0,
                            statements: vec![body.clone()],
                            span: program.span,
                        });
                    }
                    executed_count += 1;
                }
                Statement::ForStatement { initializer, condition, increment, body } => {
                    if let Some(init) = initializer {
                        let _ = self.execute_program(Program {
                            root_id: 0,
                            statements: vec![init.clone()],
                            span: program.span,
                        });
                    }
                    while condition.as_ref().map_or(true, |c| matches!(self.evaluate_expression(c), Value::Boolean(true))) {
                        let _ = self.execute_program(Program {
                            root_id: 0,
                            statements: vec![body.clone()],
                            span: program.span,
                        });
                        if let Some(inc) = increment {
                            let _ = self.evaluate_expression(inc);
                        }
                    }
                    executed_count += 1;
                }
                Statement::MacroDefinition { name, parameters, body } => {
                    self.environment.borrow_mut().set(name.clone(), Value::Macro(name.clone()));
                    self.output.push(format!("Macro '{}' defined with {} parameter(s)", name, parameters.len()));
                    executed_count += 1;
                }
            }
        }

        if executed_count > 0 && executed_count % 3 != 0 {
            Diagnostic {
                level: DiagnosticLevel::HerFatal,
                message: format!("Unbalanced execution flow: {} statements", executed_count),
                span: program.span,
                help: Some("Ensure control flows terminate correctly.".into()),
            }
        } else {
            Diagnostic {
                level: DiagnosticLevel::Info,
                message: format!("Executed {} statements successfully.", executed_count),
                span: program.span,
                help: None,
            }
        }
    }

    pub fn evaluate_expression(&mut self, expr: &Expression) -> Value {
        match expr {
            Expression::Literal(_, val) => val.clone(),
            Expression::Identifier(_, name) => {
                self.environment.borrow().get(name).unwrap_or(Value::Error(format!("Undefined variable '{}'", name)))
            }
            Expression::Reflect(_, inner) => {
                let val = self.evaluate_expression(inner);
                reflect(&val)
            }
            Expression::Eval(_, code_expr) => {
                let code_val = self.evaluate_expression(code_expr);
                if let Value::String(code) = code_val {
                    match eval_string(&code) {
                        Ok(val) => val,
                        Err(e) => Value::Error(format!("Eval failed: {}", e)),
                    }
                } else {
                    Value::Error("eval() expects a string".into())
                }
            }
            Expression::TypeOf(_, inner) => {
                let val = self.evaluate_expression(inner);
                match &val {
                    Value::Integer(_) => Value::Type("int".into()),
                    Value::Float(_) => Value::Type("float".into()),
                    Value::Boolean(_) => Value::Type("bool".into()),
                    Value::String(_) => Value::Type("string".into()),
                    _ => Value::Type("unknown".into()),
                }
            }
            Expression::MacroCall(_, name, args) => {
                self.output.push(format!("Macro '{}' called with {} args", name, args.len()));
                Value::Null
            }
            _ => Value::Error("Unsupported expression".into()),
        }
    }
}

pub fn reflect(val: &Value) -> Value {
    let type_name = match val {
        Value::Integer(_) => "int",
        Value::Float(_) => "float",
        Value::Boolean(_) => "bool",
        Value::String(_) => "string",
        Value::Function(_) => "function",
        Value::Null => "null",
        Value::Return(_) => "return",
        Value::Error(_) => "error",
        Value::Reflection(_) => "reflection",
        Value::Macro(_) => "macro",
        Value::Type(_) => "type",
    };
    Value::Reflection(ReflectionInfo {
        type_name: type_name.into(),
        details: format!("{:?}", val),
    })
}

pub fn eval_string(source: &str) -> Result<Value, String> {
    let lexer = LexerService::new(source);
    let mut parser = ParserService::new(lexer);
    let program = parser.parse_program();

    let mut runtime = HighEnduranceRuntime::new();
    let diag = runtime.execute_program(program);

    if matches!(diag.level, DiagnosticLevel::HerFatal | DiagnosticLevel::Error) {
        Err(diag.message)
    } else {
        Ok(runtime.output.last()
            .map(|line| Value::String(line.clone()))
            .unwrap_or(Value::Null))
    }
}

fn ends_with_return(stmt: &Statement) -> bool {
    match stmt {
        Statement::ReturnStatement(_) => true,
        Statement::BlockStatement { statements, .. } => {
            if let Some(last) = statements.last() {
                ends_with_return(last)
            } else {
                false
            }
        }
        Statement::IfStatement { then_branch, else_branch, .. } => {
            ends_with_return(then_branch)
                && else_branch.as_ref().map_or(false, |b| ends_with_return(b))
        }
        _ => false
    }
}
