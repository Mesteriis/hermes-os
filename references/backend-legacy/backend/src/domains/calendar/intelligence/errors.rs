use thiserror::Error;

#[derive(Debug, Error)]
pub enum CalendarIntelligenceError {
    #[error("analysis failed: {0}")]
    AnalysisFailed(String),
}
