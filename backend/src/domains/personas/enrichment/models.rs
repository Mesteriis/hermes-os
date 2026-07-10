use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrichedPersona {
    #[serde(rename = "persona_id", alias = "person_id")]
    pub person_id: String,
    pub display_name: String,
    pub email_address: Option<String>,
    pub language: Option<String>,
    pub tone: Option<String>,
    pub trust_score: Option<i16>,
    pub avg_response_hours: Option<f64>,
    pub preferred_channel: Option<String>,
    pub last_interaction_at: Option<DateTime<Utc>>,
    pub interaction_count: i32,
    pub frequent_topics: Vec<String>,
    pub writing_style: Option<String>,
    pub persona_metadata: Value,
    pub is_favorite: bool,
    pub is_address_book: bool,
    pub notes: Option<String>,
    pub linked_projects: Vec<String>,
    pub linked_documents: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
