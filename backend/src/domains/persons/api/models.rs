use chrono::{DateTime, Utc};
use serde::Serialize;

use super::errors::PersonProjectionError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonaType {
    Human,
    AiAgent,
    OrganizationProxy,
    System,
}

impl PersonaType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::AiAgent => "ai_agent",
            Self::OrganizationProxy => "organization_proxy",
            Self::System => "system",
        }
    }
}

impl TryFrom<&str> for PersonaType {
    type Error = PersonProjectionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "human" => Ok(Self::Human),
            "ai_agent" => Ok(Self::AiAgent),
            "organization_proxy" => Ok(Self::OrganizationProxy),
            "system" => Ok(Self::System),
            _ => Err(PersonProjectionError::InvalidPersonaType(value.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Person {
    pub person_id: String,
    pub display_name: String,
    pub email_address: String,
    pub persona_type: PersonaType,
    pub is_self: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
