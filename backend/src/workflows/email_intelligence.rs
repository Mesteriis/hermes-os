mod categories;
mod errors;
mod heuristics;
mod models;
mod prompt;
mod service;

#[cfg(test)]
mod tests;

pub use categories::EmailCategory;
pub use errors::EmailIntelligenceError;
pub use models::EmailAnalysis;
pub use service::EmailIntelligenceService;
