#![forbid(unsafe_code)]

pub mod admission;
mod managed_runtime;
mod worker;

pub use admission::ollama_ai_module_descriptor_v1;
pub use managed_runtime::{
    OllamaAiManagedRuntimeErrorV1, OllamaAiManagedRuntimeV1, OllamaAiRuntimeAdmissionV1,
};

pub const PACKAGE: &str = "hermes-ollama-ai-runtime";
