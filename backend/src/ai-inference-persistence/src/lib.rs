#![forbid(unsafe_code)]

mod model;
mod repository;
pub mod schema;
mod summary_model;
mod summary_repository;

pub use model::{
    AI_INFERENCE_RECOVERY_LIMIT_V1, AiInferencePersistenceErrorV1, AiInferencePersistenceOutcomeV1,
    AiInferenceTransitionV1, PersistedAiInferenceRunV1,
};
pub use repository::AiInferencePersistenceV1;
pub use summary_model::{
    AiSummaryPersistenceOutcomeV1, AiSummaryTransitionV1, PersistedAiSummaryRunV1,
};

pub const PACKAGE: &str = "hermes-ai-inference-persistence";
