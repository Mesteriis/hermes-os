use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionEvidenceSourceKind {
    Observation,
    Communication,
    Document,
    Event,
    Memory,
    Knowledge,
    Decision,
    Obligation,
    Task,
    Relationship,
    Project,
    Organization,
    Persona,
}

impl DecisionEvidenceSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observation => "observation",
            Self::Communication => "communication",
            Self::Document => "document",
            Self::Event => "event",
            Self::Memory => "memory",
            Self::Knowledge => "knowledge",
            Self::Decision => "decision",
            Self::Obligation => "obligation",
            Self::Task => "task",
            Self::Relationship => "relationship",
            Self::Project => "project",
            Self::Organization => "organization",
            Self::Persona => "persona",
        }
    }
}
