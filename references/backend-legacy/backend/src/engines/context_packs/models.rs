use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::errors::ContextPackStoreError;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextPackKind {
    Persona,
    Meeting,
    Task,
    Calendar,
    Project,
    Review,
}

impl ContextPackKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Persona => "persona",
            Self::Meeting => "meeting",
            Self::Task => "task",
            Self::Calendar => "calendar",
            Self::Project => "project",
            Self::Review => "review",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ContextPackStoreError> {
        match value.as_ref() {
            "persona" => Ok(Self::Persona),
            "meeting" => Ok(Self::Meeting),
            "task" => Ok(Self::Task),
            "calendar" => Ok(Self::Calendar),
            "project" => Ok(Self::Project),
            "review" => Ok(Self::Review),
            unknown => Err(ContextPackStoreError::UnknownContextPackKind(
                unknown.to_owned(),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextPackSourceKind {
    Observation,
    DomainEntity,
    Knowledge,
    Relationship,
    Decision,
    Task,
    Obligation,
    Document,
    CalendarEvent,
    Project,
    ReviewItem,
}

impl ContextPackSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observation => "observation",
            Self::DomainEntity => "domain_entity",
            Self::Knowledge => "knowledge",
            Self::Relationship => "relationship",
            Self::Decision => "decision",
            Self::Task => "task",
            Self::Obligation => "obligation",
            Self::Document => "document",
            Self::CalendarEvent => "calendar_event",
            Self::Project => "project",
            Self::ReviewItem => "review_item",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ContextPackStoreError> {
        match value.as_ref() {
            "observation" => Ok(Self::Observation),
            "domain_entity" => Ok(Self::DomainEntity),
            "knowledge" => Ok(Self::Knowledge),
            "relationship" => Ok(Self::Relationship),
            "decision" => Ok(Self::Decision),
            "task" => Ok(Self::Task),
            "obligation" => Ok(Self::Obligation),
            "document" => Ok(Self::Document),
            "calendar_event" => Ok(Self::CalendarEvent),
            "project" => Ok(Self::Project),
            "review_item" => Ok(Self::ReviewItem),
            unknown => Err(ContextPackStoreError::UnknownContextPackSourceKind(
                unknown.to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ContextPack {
    pub context_pack_id: String,
    pub kind: ContextPackKind,
    pub subject_id: String,
    pub content: Value,
    pub metadata: Value,
    pub rebuildable: bool,
    pub built_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewContextPack {
    pub kind: ContextPackKind,
    pub subject_id: String,
    pub content: Value,
    pub metadata: Value,
    pub rebuildable: bool,
}

impl NewContextPack {
    pub fn new(kind: ContextPackKind, subject_id: impl Into<String>, content: Value) -> Self {
        Self {
            kind,
            subject_id: subject_id.into(),
            content,
            metadata: json!({}),
            rebuildable: true,
        }
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn rebuildable(mut self, rebuildable: bool) -> Self {
        self.rebuildable = rebuildable;
        self
    }

    pub fn validate(&self) -> Result<(), ContextPackStoreError> {
        validate_non_empty("subject_id", &self.subject_id)?;
        validate_json_object("content", &self.content)?;
        validate_json_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ContextPackSource {
    pub context_pack_id: String,
    pub source_kind: ContextPackSourceKind,
    pub source_id: String,
    pub role: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewContextPackSource {
    pub source_kind: ContextPackSourceKind,
    pub source_id: String,
    pub role: String,
    pub metadata: Value,
}

impl NewContextPackSource {
    pub fn new(source_kind: ContextPackSourceKind, source_id: impl Into<String>) -> Self {
        Self {
            source_kind,
            source_id: source_id.into(),
            role: "source".to_owned(),
            metadata: json!({}),
        }
    }

    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.role = role.into();
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn validate(&self) -> Result<(), ContextPackStoreError> {
        validate_non_empty("source_id", &self.source_id)?;
        validate_non_empty("role", &self.role)?;
        validate_json_object("metadata", &self.metadata)?;
        Ok(())
    }
}

pub(super) fn validate_context_pack_with_sources(
    pack: &NewContextPack,
    sources: &[NewContextPackSource],
) -> Result<(), ContextPackStoreError> {
    pack.validate()?;
    if sources.is_empty() {
        return Err(ContextPackStoreError::MissingSources);
    }
    for source in sources {
        source.validate()?;
    }
    Ok(())
}

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), ContextPackStoreError> {
    if value.trim().is_empty() {
        return Err(ContextPackStoreError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_json_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), ContextPackStoreError> {
    if !value.is_object() {
        return Err(ContextPackStoreError::InvalidJsonObject(field_name));
    }

    Ok(())
}
