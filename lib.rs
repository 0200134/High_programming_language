// src/lib.rs
// High Programming Language Compiler/Interpreter의 루트 모듈 정의입니다.

pub mod data_structures;
pub mod lexer_service;
pub mod parser_service;
pub mod ft_runtime;
pub mod analyzer_service; 
pub mod executor_service; 
pub mod blockchain; // Hargo-Chain 모듈 추가
pub mod compiler_services;
pub mod optimizer;

pub mod ir_generator;      // ✅ IR 생성기 모듈
pub mod native_codegen;    // ✅ 네이티브 코드 생성기 모듈


// 자주 사용되는 타입들을 루트 모듈에서 직접 사용할 수 있도록 export 합니다.
pub use data_structures::{Diagnostic, DiagnosticLevel, Program, Value};
pub use blockchain::{Block, Blockchain};
pub use analyzer_service::{AnalysisResult, AnalysisError, AnalyzerService};
pub use executor_service::{ExecutionRequest, ExecutionResult, ExecutorService};
pub use compiler_services::{CompileRequest, CompileOptions, CompileResult, CompilerService};
