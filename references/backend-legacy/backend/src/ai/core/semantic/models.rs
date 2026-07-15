use chrono::{DateTime, Utc};

use super::super::errors::AiError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticSourceKind {
    Message,
    Document,
    Project,
    Task,
    Persona,
}

impl SemanticSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Document => "document",
            Self::Project => "project",
            Self::Task => "task",
            Self::Persona => "persona",
        }
    }

    pub(super) fn parse(value: &str) -> Result<Self, AiError> {
        match value {
            "message" => Ok(Self::Message),
            "document" => Ok(Self::Document),
            "project" => Ok(Self::Project),
            "task" => Ok(Self::Task),
            "contact" | "person" | "persona" => Ok(Self::Persona),
            _ => Err(AiError::InvalidSourceKind(value.to_owned())),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticEmbedding {
    pub semantic_embedding_id: String,
    pub source_kind: String,
    pub source_id: String,
    pub observation_id: Option<String>,
    pub title: String,
    pub source_text: String,
    pub content_hash: String,
    pub embedding_model: String,
    pub embedding_dimension: i32,
    pub graph_node_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug)]
pub struct NewSemanticEmbedding<'a> {
    pub source_kind: SemanticSourceKind,
    pub source_id: &'a str,
    pub observation_id: Option<&'a str>,
    pub title: &'a str,
    pub source_text: &'a str,
    pub embedding_model: &'a str,
    pub embedding: &'a [f32],
    pub graph_node_id: Option<&'a str>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticSearchResult {
    pub source_kind: String,
    pub source_id: String,
    pub observation_id: Option<String>,
    pub title: String,
    pub source_text: String,
    pub graph_node_id: Option<String>,
    pub score: f64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SemanticIndexReport {
    pub sources_seen: usize,
    pub sources_indexed: usize,
    pub sources_skipped: usize,
}

pub(super) struct SemanticSource {
    pub(super) source_kind: SemanticSourceKind,
    pub(super) source_id: String,
    pub(super) observation_id: Option<String>,
    pub(super) title: String,
    pub(super) source_text: String,
    pub(super) graph_node_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::SemanticSourceKind;

    #[test]
    fn persona_source_kind_writes_persona_and_reads_legacy_person_or_contact() {
        assert_eq!(SemanticSourceKind::Persona.as_str(), "persona");
        assert_eq!(
            SemanticSourceKind::parse("persona").expect("persona source kind"),
            SemanticSourceKind::Persona
        );
        assert_eq!(
            SemanticSourceKind::parse("person").expect("person source kind"),
            SemanticSourceKind::Persona
        );
        assert_eq!(
            SemanticSourceKind::parse("contact").expect("legacy contact source kind"),
            SemanticSourceKind::Persona
        );
    }
}
