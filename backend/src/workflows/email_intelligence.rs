mod categories;
mod errors;
mod heuristics;
mod models;
mod pipeline;
mod prompt;
mod service;

#[cfg(test)]
mod tests;

pub use categories::EmailCategory;
pub use errors::EmailIntelligenceError;
pub use models::{EmailAnalysis, EmailKnowledgeCandidate, EmailSummaryContract};
pub use pipeline::{MailAiPipelineReport, MailAiPipelineService};
pub use service::EmailIntelligenceService;
