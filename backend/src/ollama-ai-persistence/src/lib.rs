#![forbid(unsafe_code)]

mod model;
mod repository;
pub mod schema;

pub use model::{
    OllamaAiPersistenceErrorV1, OllamaAiPersistenceOutcomeV1, OllamaAiTransitionV1,
    PersistedOllamaAiRunV1,
};
pub use repository::OllamaAiPersistenceV1;

pub const PACKAGE: &str = "hermes-ollama-ai-persistence";
