use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::InvestigatorError;
use crate::domains::personas::enrichment::EnrichedPersona;
use crate::domains::personas::memory::{PersonaFact, PersonaMemoryCard, RelationshipEvent};

#[derive(Clone, Debug, Serialize)]
pub struct DossierSectionItem {
    pub label: String,
    pub value: String,
    pub source_refs: Vec<String>,
    pub confidence: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PersonaDossier {
    pub persona: EnrichedPersona,
    pub facts: Vec<PersonaFact>,
    pub memory_cards: Vec<PersonaMemoryCard>,
    pub timeline: Vec<RelationshipEvent>,
    pub identities: Vec<Value>,
    pub expertise: Vec<Value>,
    pub promises: Vec<Value>,
    pub risks: Vec<Value>,
    pub summary: String,
    pub interests: Vec<DossierSectionItem>,
    pub projects: Vec<DossierSectionItem>,
    pub organizations: Vec<DossierSectionItem>,
    pub skills: Vec<DossierSectionItem>,
    pub communication_patterns: Vec<DossierSectionItem>,
    pub ai_observations: Vec<DossierSectionItem>,
    pub source_refs: Vec<String>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DossierReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl DossierReviewState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub fn parse(value: &str) -> Result<Self, InvestigatorError> {
        match value {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(InvestigatorError::InvalidDossierReviewState),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DossierSnapshot {
    pub dossier_snapshot_id: String,
    pub persona_id: String,
    pub dossier: Value,
    pub source_refs: Value,
    pub review_state: DossierReviewState,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub generated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MeetingPrep {
    #[serde(rename = "persona_id")]
    pub persona_id: String,
    pub display_name: String,
    pub last_interaction_days: Option<i64>,
    pub open_promises: i64,
    pub open_risks: i64,
    pub recent_topics: Vec<String>,
    pub communication_tips: Vec<String>,
    pub shared_projects: Vec<String>,
}
