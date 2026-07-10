use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonaPromise {
    pub id: String,
    #[serde(rename = "persona_id", alias = "person_id")]
    pub person_id: String,
    pub description: String,
    pub source_message_id: Option<String>,
    pub promised_at: DateTime<Utc>,
    pub due_at: Option<DateTime<Utc>>,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonaRisk {
    pub id: String,
    #[serde(rename = "persona_id", alias = "person_id")]
    pub person_id: String,
    pub risk_type: String,
    pub description: String,
    pub severity: String,
    pub source: String,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
}
