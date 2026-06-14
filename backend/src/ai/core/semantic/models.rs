use chrono::{DateTime, Utc};

use super::super::errors::AiError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticSourceKind {
    Message,
    Document,
    Project,
    Task,
    Person,
}

impl SemanticSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Document => "document",
            Self::Project => "project",
            Self::Task => "task",
            Self::Person => "person",
        }
    }

    pub(super) fn parse(value: &str) -> Result<Self, AiError> {
        match value {
            "message" => Ok(Self::Message),
            "document" => Ok(Self::Document),
            "project" => Ok(Self::Project),
            "task" => Ok(Self::Task),
            "contact" | "person" => Ok(Self::Person),
            _ => Err(AiError::InvalidSourceKind(value.to_owned())),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticEmbedding {
    pub semantic_embedding_id: String,
    pub source_kind: String,
    pub source_id: String,
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
    pub(super) title: String,
    pub(super) source_text: String,
    pub(super) graph_node_id: Option<String>,
}
