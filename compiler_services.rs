use tokio::time::Instant;
use crate::analyzer_service::{AnalyzerService, AnalysisResult};
use crate::executor_service::{ExecutorService, ExecutionRequest, ExecutionResult, ExecutionStatus};
use crate::blockchain::Blockchain;
use crate::lexer_service::LexerService;
use crate::parser_service::ParserService;
use crate::optimizer::Optimizer;
use crate::data_structures::{Program, Statement};
use crate::ir_generator::generate_ir;
use crate::native_codegen::{generate_native_binary, assemble_and_link};

pub struct CompilerService {
    analyzer: AnalyzerService,
    executor: ExecutorService,
    blockchain: Blockchain,
}

impl CompilerService {
    pub fn new() -> Self {
        Self {
            analyzer: AnalyzerService::new(),
            executor: ExecutorService::new(),
            blockchain: Blockchain::new(),
        }
    }

    pub async fn compile(&mut self, request: CompileRequest) -> CompileResult {
        let start_time = Instant::now();
        let mut errors = vec![];
        let mut success = true;

        let analysis_report = self.run_analysis(&request.source_code, &mut errors, &mut success).await;
        let mut program = self.run_parsing(&request.source_code, &mut errors, &mut success);

        if request.options.optimization_level > 0 {
            Optimizer::optimize(&mut program);
        }

        if !ends_with_return(&program) {
            success = false;
            errors.push("컴파일 실패: 실행 흐름이 균형을 이루지 않음 (return 누락 또는 위치 오류).".into());
        }

        let mut compiled_output = String::new();
        if success && request.options.emit_native {
            let ir = generate_ir(&program);
            let asm_path = "compiled.asm";

            #[cfg(target_os = "windows")]
            let bin_path = "compiled.exe";

            #[cfg(not(target_os = "windows"))]
            let bin_path = "compiled.out";

            match generate_native_binary(&ir, asm_path) {
                Ok(_) => match assemble_and_link(asm_path, bin_path) {
                    Ok(_) => {
                        compiled_output = format!("네이티브 실행 파일 생성 완료: {}", bin_path);
                    }
                    Err(e) => {
                        success = false;
                        errors.push(format!("링커 실패: {}", e));
                    }
                },
                Err(e) => {
                    success = false;
                    errors.push(format!("어셈블리 생성 실패: {}", e));
                }
            }
        }

        let execution_result = if success {
            let exec_request = ExecutionRequest {
                compiled_code_reference: compiled_output.clone(),
                input_data: if analysis_report.detected_sentiment == "Positive" {
                    Some("Success Data".into())
                } else {
                    None
                },
            };

            let result = self.executor.execute_code(exec_request).await;

            if matches!(result.status, ExecutionStatus::RuntimeError) {
                success = false;
                errors.push("실행 중 에러 발생: 런타임 오류".into());
            }

            result
        } else {
            ExecutionResult {
                output_log: vec!["[Executor] 실행되지 않음: 컴파일 에러.".into()],
                status: ExecutionStatus::Skipped,
                execution_time_ms: 0,
            }
        };

        let proof_hash = format!(
            "POCI_{}_{}_{:?}",
            request.source_code.len(),
            request.options.target_platform,
            execution_result.status
        );
        let new_block = self.blockchain.add_block(proof_hash);
        let total_time_ms = start_time.elapsed().as_millis();

        CompileResult {
            success,
            compiled_output,
            analysis_report,
            execution_log: execution_result.output_log,
            execution_status: execution_result.status,
            proof_block_index: new_block.index,
            errors,
            total_time_ms,
        }
    }

    async fn run_analysis(&self, source: &str, errors: &mut Vec<String>, success: &mut bool) -> AnalysisResult {
        match self.analyzer.analyze_text(source).await {
            Ok(report) => report,
            Err(e) => {
                errors.push(format!("분석 실패: {}", e));
                *success = false;
                AnalysisResult {
                    word_count: 0,
                    readability_score: 0.0,
                    detected_sentiment: "Error".into(),
                    processing_time_ms: 0,
                }
            }
        }
    }

    fn run_parsing(&self, source: &str, errors: &mut Vec<String>, success: &mut bool) -> Program {
        let lexer = LexerService::new(source);
        let mut parser = ParserService::new(lexer);
        parser.parse_program()
    }
}

// ─── 실행 흐름 검사 ─────────────────────────────

fn ends_with_return(program: &Program) -> bool {
    if let Some(last_stmt) = program.statements.last() {
        is_terminal(last_stmt)
    } else {
        false
    }
}

fn is_terminal(stmt: &Box<Statement>) -> bool {
    match stmt.as_ref() {
        Statement::ReturnStatement(_) => true,
        Statement::BlockStatement { statements, .. } => {
            if let Some(inner_last) = statements.last() {
                is_terminal(inner_last)
            } else {
                false
            }
        }
        Statement::IfStatement { then_branch, else_branch, .. } => {
            let then_term = is_terminal(then_branch);
            let else_term = else_branch.as_ref().map_or(false, is_terminal);
            then_term && else_term
        }
        _ => false,
    }
}

// ─── 요청 및 결과 구조체 ─────────────────────────────

#[derive(Debug)]
pub struct CompileRequest {
    pub source_code: String,
    pub options: CompileOptions,
}

#[derive(Debug)]
pub struct CompileOptions {
    pub target_platform: String,
    pub optimization_level: u8,
    pub emit_native: bool,
}

#[derive(Debug)]
pub struct CompileResult {
    pub success: bool,
    pub compiled_output: String,
    pub analysis_report: AnalysisResult,
    pub execution_log: Vec<String>,
    pub execution_status: ExecutionStatus,
    pub proof_block_index: u32,
    pub errors: Vec<String>,
    pub total_time_ms: u128,
}
