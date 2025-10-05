use tokio::time::{self, Duration, Instant};
use std::error::Error;
use std::fmt;

/// 텍스트 분석 결과를 담는 구조체
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub word_count: usize,
    pub readability_score: f64,
    pub detected_sentiment: String,
    pub processing_time_ms: u128,
}

/// 사용자 정의 에러 타입
#[derive(Debug)]
pub struct AnalysisError(pub String);

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Analysis failed: {}", self.0)
    }
}

impl Error for AnalysisError {}

/// 텍스트 분석 서비스 구조체
pub struct AnalyzerService;

impl AnalyzerService {
    pub fn new() -> Self {
        println!("[Analyzer] AnalyzerService가 초기화되었습니다.");
        Self {}
    }

    /// 텍스트 분석을 비동기적으로 수행합니다.
    pub async fn analyze_text(&self, source_code: &str) -> Result<AnalysisResult, AnalysisError> {
        let start_time = Instant::now();
        time::sleep(Duration::from_millis(30)).await;

        let word_count = source_code.split_whitespace().count();
        if word_count == 0 {
            return Err(AnalysisError("분석할 텍스트가 비어 있거나 공백만 포함합니다.".into()));
        }

        let sentiment = Self::detect_sentiment(source_code);
        let readability_score = Self::calculate_readability(word_count);
        let processing_time_ms = start_time.elapsed().as_millis();

        Ok(AnalysisResult {
            word_count,
            readability_score,
            detected_sentiment: sentiment,
            processing_time_ms,
        })
    }

    /// 감정 분석 로직 (키워드 기반)
    fn detect_sentiment(text: &str) -> String {
        let positive_keywords = ["hello", "success", "great", "awesome", "good"];
        let negative_keywords = ["error", "fail", "panic", "bad", "crash"];

        let lower = text.to_lowercase();

        if negative_keywords.iter().any(|kw| lower.contains(kw)) {
            "Negative".to_string()
        } else if positive_keywords.iter().any(|kw| lower.contains(kw)) {
            "Positive".to_string()
        } else {
            "Neutral".to_string()
        }
    }

    /// 가독성 점수 계산 (단순 모델)
    fn calculate_readability(word_count: usize) -> f64 {
        let score = word_count as f64 / 10.0;
        score.min(1.0)
    }
}
