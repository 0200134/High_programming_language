use tokio::time::Instant;
use std::fs;
use std::io::{self, Write};

use High::compiler_services::{CompilerService, CompileRequest, CompileOptions};
use High::analyzer_service::AnalyzerService;
use High::executor_service::{ExecutorService, ExecutionRequest, ExecutionStatus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- High Programming Language Compiler Orchestrator ---");

    let mut compiler_service = CompilerService::new();
    let analyzer_service = AnalyzerService::new();
    let executor_service = ExecutorService::new();

    loop {
        println!("\n-------------------------------------------------------");
        println!("Type 'q' or 'quit' to exit.");
        print!("Enter file path to compile (e.g. main.high): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let file_path = input.trim();

        if file_path.eq_ignore_ascii_case("q") || file_path.eq_ignore_ascii_case("quit") {
            println!("Exiting.");
            break;
        }

        let source_code = match fs::read_to_string(file_path) {
            Ok(code) => code,
            Err(e) => {
                println!("❌ Failed to read file '{}': {}", file_path, e);
                continue;
            }
        };

        let start_time = Instant::now();

        println!("\n[Analyzer] Running preliminary code analysis...");
        let _ = match analyzer_service.analyze_text(&source_code).await {
            Ok(res) => {
                println!("[Analyzer] Analysis successful.");
                println!("  - Sentiment: {}", res.detected_sentiment);
                println!("  - Word Count: {}", res.word_count);
                println!("  - Readability Score: {:.2}", res.readability_score);
                res
            },
            Err(e) => {
                println!("[Analyzer] Analysis failed: {}", e);
                continue;
            }
        };

        let request = CompileRequest {
    source_code,
    options: CompileOptions {
        target_platform: "her_vm".into(),
        optimization_level: 2,
        emit_native: true, // ✅ 네이티브 바이너리 생성 여부
    },
};


        println!("\n[Compiler] Starting full compilation pipeline...");
        let result = compiler_service.compile(request).await;

        if result.success {
            println!("\n--- Compilation Successful ---");
            println!("Compiled Output: {}", result.compiled_output);

            println!("\n[Executor] Requesting code execution...");
            let execution_request = ExecutionRequest {
                compiled_code_reference: result.compiled_output.clone(),
                input_data: Some("1, 2, 3".into()),
            };

            let execution_result = executor_service.execute_code(execution_request).await;

            println!("--- Execution Result ---");
            match execution_result.status {
                ExecutionStatus::Success => println!("Status: Success"),
                ExecutionStatus::RuntimeError => println!("Status: Runtime Error"),
                ExecutionStatus::Skipped => println!("Status: Skipped"),
            }

            println!("Log:");
            for line in execution_result.output_log {
                println!("  {}", line);
            }
            println!("Execution Time: {}ms", execution_result.execution_time_ms);
            println!("Proof Block Index: {}", result.proof_block_index);
        } else {
            println!("\n--- Compilation Failed ---");
            for error in result.errors {
                println!("Error: {}", error);
            }
        }

        let total_elapsed = start_time.elapsed();
        println!("\nTotal Orchestration Time: {:.2}ms", total_elapsed.as_millis());
    }

    Ok(())
}
