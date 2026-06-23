mod commands;
mod errors;
mod materialization;
mod models;
mod queries;
mod rows;
mod store;

pub use errors::PersonEnrichmentError;
pub use models::EnrichedPerson;
pub use store::PersonEnrichmentStore;

pub const PERSON_TRUST_SCORE_CHANGED_EVENT_TYPE: &str = "person.enrichment.trust_score_changed";
