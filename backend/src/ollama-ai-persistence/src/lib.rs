#![forbid(unsafe_code)]

mod model;
mod repository;
pub mod schema;
mod summary_model;
mod summary_repository;
mod translation_model;
mod translation_repository;

pub use model::{
    OllamaAiPersistenceErrorV1, OllamaAiPersistenceOutcomeV1, OllamaAiTransitionV1,
    PersistedOllamaAiRunV1,
};
pub use repository::OllamaAiPersistenceV1;
pub use summary_model::{
    OllamaSummaryPersistenceOutcomeV1, OllamaSummaryTransitionV1, PersistedOllamaSummaryRunV1,
};
pub use translation_model::{
    OllamaTranslationPersistenceOutcomeV1, OllamaTranslationTransitionV1,
    PersistedOllamaTranslationRunV1,
};

pub const PACKAGE: &str = "hermes-ollama-ai-persistence";
