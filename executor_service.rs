use tokio::time::{self, Duration};

/// 실행 상태를 나타내는 열거형
#[derive(Debug)]
pub enum ExecutionStatus {
    Success,
    RuntimeError,
    Skipped,
}

/// 코드 실행 요청 구조체
#[derive(Debug)]
pub struct ExecutionRequest {
    pub compiled_code_reference: String,
    pub input_data: Option<String>,
}

/// 실행 결과 구조체
#[derive(Debug)]
pub struct ExecutionResult {
    pub output_log: Vec<String>,
    pub status: ExecutionStatus,
    pub execution_time_ms: u128,
}

/// 실행기 서비스
pub struct ExecutorService {}

impl ExecutorService {
    pub fn new() -> Self {
        println!("[Executor] ExecutorService가 초기화되었습니다.");
        Self {}
    }

    pub async fn execute_code(&self, request: ExecutionRequest) -> ExecutionResult {
        let start_time = time::Instant::now();
        let mut output_log = vec![];
        let mut status = ExecutionStatus::Success;

        println!("[Executor] 코드 실행 시작...");
        time::sleep(Duration::from_millis(30)).await;
        output_log.push(">> [System] Runtime environment started.".into());

        let delay = (request.compiled_code_reference.len() * 2).max(50);
        time::sleep(Duration::from_millis(delay as u64)).await;

        if request.compiled_code_reference.contains("error") {
            status = ExecutionStatus::RuntimeError;
            let fault = request.compiled_code_reference.split(' ').last().unwrap_or("UNKNOWN");
            output_log.push(format!(">> [Error] Segmentation Fault at instruction: {}", fault));
        } else {
            output_log.push(Self::generate_output(&request));
        }

        let execution_time_ms = start_time.elapsed().as_millis();
        println!("[Executor] 실행 완료. 상태: {:?}, 소요 시간: {}ms", status, execution_time_ms);

        ExecutionResult {
            output_log,
            status,
            execution_time_ms,
        }
    }

    fn generate_output(request: &ExecutionRequest) -> String {
        let input = request.input_data.as_deref().unwrap_or("None");
        format!(">> [Code Output] Hello from the compiled code! Input data was: {}", input)
    }
}
