mod commands;
mod errors;
mod materialization;
mod models;
mod queries;
mod rows;
mod store;

pub use errors::PersonaEnrichmentError;
pub use models::EnrichedPersona;
pub use store::PersonaEnrichmentStore;

pub const PERSONA_TRUST_SCORE_CHANGED_EVENT_TYPE: &str = "persona.enrichment.trust_score_changed";
