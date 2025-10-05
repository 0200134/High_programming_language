use crate::data_structures::{Program, Statement};

#[derive(Debug, Clone)]
pub struct IRInstruction {
    pub opcode: String,
    pub operands: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IRModule {
    pub instructions: Vec<IRInstruction>,
}

pub fn generate_ir(program: &Program) -> IRModule {
    let mut instructions = vec![];

    for stmt in &program.statements {
        match stmt.as_ref() {
            Statement::LetStatement { name, value, .. } => {
                instructions.push(IRInstruction {
                    opcode: "let".into(),
                    operands: vec![name.clone(), format!("{:?}", value)],
                });
            }
            Statement::ReturnStatement(expr) => {
                instructions.push(IRInstruction {
                    opcode: "return".into(),
                    operands: vec![format!("{:?}", expr)],
                });
            }
            _ => {
                instructions.push(IRInstruction {
                    opcode: "noop".into(),
                    operands: vec![],
                });
            }
        }
    }

    IRModule { instructions }
}
