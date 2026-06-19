use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::errors::ObservationStoreError;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationOriginKind {
    VaultSource,
    Manual,
    BrowserCapture,
    VoiceMemo,
    FileImport,
    LocalRuntime,
    TestFixture,
}

impl ObservationOriginKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::VaultSource => "vault_source",
            Self::Manual => "manual",
            Self::BrowserCapture => "browser_capture",
            Self::VoiceMemo => "voice_memo",
            Self::FileImport => "file_import",
            Self::LocalRuntime => "local_runtime",
            Self::TestFixture => "test_fixture",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ObservationStoreError> {
        match value.as_ref() {
            "vault_source" => Ok(Self::VaultSource),
            "manual" => Ok(Self::Manual),
            "browser_capture" => Ok(Self::BrowserCapture),
            "voice_memo" => Ok(Self::VoiceMemo),
            "file_import" => Ok(Self::FileImport),
            "local_runtime" => Ok(Self::LocalRuntime),
            "test_fixture" => Ok(Self::TestFixture),
            unknown => Err(ObservationStoreError::UnknownOriginKind(unknown.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ObservationKindDefinition {
    pub kind_definition_id: String,
    pub code: String,
    pub name: String,
    pub version: i32,
    pub category: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Observation {
    pub observation_id: String,
    pub kind_definition_id: String,
    pub kind_code: String,
    pub origin_kind: ObservationOriginKind,
    pub vault_source_id: Option<String>,
    pub observed_at: DateTime<Utc>,
    pub captured_at: DateTime<Utc>,
    pub payload: Value,
    pub confidence: f64,
    pub content_hash: String,
    pub source_ref: String,
    pub provenance: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewObservation {
    pub kind_code: String,
    pub origin_kind: ObservationOriginKind,
    pub vault_source_id: Option<String>,
    pub observed_at: DateTime<Utc>,
    pub payload: Value,
    pub confidence: f64,
    pub source_ref: String,
    pub provenance: Value,
}

impl NewObservation {
    pub fn new(
        kind_code: impl Into<String>,
        origin_kind: ObservationOriginKind,
        observed_at: DateTime<Utc>,
        payload: Value,
        source_ref: impl Into<String>,
    ) -> Self {
        Self {
            kind_code: kind_code.into(),
            origin_kind,
            vault_source_id: None,
            observed_at,
            payload,
            confidence: 1.0,
            source_ref: source_ref.into(),
            provenance: json!({}),
        }
    }

    pub fn vault_source_id(mut self, vault_source_id: impl Into<String>) -> Self {
        self.vault_source_id = Some(vault_source_id.into());
        self
    }

    pub fn confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn provenance(mut self, provenance: Value) -> Self {
        self.provenance = provenance;
        self
    }

    pub fn validate(&self) -> Result<(), ObservationStoreError> {
        validate_non_empty("kind_code", &self.kind_code)?;
        validate_non_empty("source_ref", &self.source_ref)?;
        if let Some(vault_source_id) = &self.vault_source_id {
            validate_non_empty("vault_source_id", vault_source_id)?;
        }
        validate_json_object("payload", &self.payload)?;
        validate_json_object("provenance", &self.provenance)?;
        validate_score("confidence", self.confidence)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ObservationLink {
    pub observation_id: String,
    pub domain: String,
    pub entity_kind: String,
    pub entity_id: String,
    pub relationship_kind: String,
    pub confidence: f64,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewObservationLink {
    pub observation_id: String,
    pub domain: String,
    pub entity_kind: String,
    pub entity_id: String,
    pub relationship_kind: String,
    pub confidence: f64,
    pub metadata: Value,
}

impl NewObservationLink {
    pub fn new(
        observation_id: impl Into<String>,
        domain: impl Into<String>,
        entity_kind: impl Into<String>,
        entity_id: impl Into<String>,
    ) -> Self {
        Self {
            observation_id: observation_id.into(),
            domain: domain.into(),
            entity_kind: entity_kind.into(),
            entity_id: entity_id.into(),
            relationship_kind: "evidence_for".to_owned(),
            confidence: 1.0,
            metadata: json!({}),
        }
    }

    pub fn relationship_kind(mut self, relationship_kind: impl Into<String>) -> Self {
        self.relationship_kind = relationship_kind.into();
        self
    }

    pub fn confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn validate(&self) -> Result<(), ObservationStoreError> {
        validate_non_empty("observation_id", &self.observation_id)?;
        validate_non_empty("domain", &self.domain)?;
        validate_non_empty("entity_kind", &self.entity_kind)?;
        validate_non_empty("entity_id", &self.entity_id)?;
        validate_non_empty("relationship_kind", &self.relationship_kind)?;
        validate_json_object("metadata", &self.metadata)?;
        validate_score("confidence", self.confidence)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationIngestionRunStatus {
    Running,
    Succeeded,
    Failed,
    Skipped,
}

impl ObservationIngestionRunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ObservationStoreError> {
        match value.as_ref() {
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "skipped" => Ok(Self::Skipped),
            unknown => Err(ObservationStoreError::UnknownIngestionRunStatus(
                unknown.to_owned(),
            )),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ObservationIngestionRun {
    pub ingestion_run_id: String,
    pub observation_id: String,
    pub pipeline: String,
    pub status: ObservationIngestionRunStatus,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub output: Value,
    pub error_message: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewObservationIngestionRun {
    pub ingestion_run_id: String,
    pub observation_id: String,
    pub pipeline: String,
}

impl NewObservationIngestionRun {
    pub fn new(
        ingestion_run_id: impl Into<String>,
        observation_id: impl Into<String>,
        pipeline: impl Into<String>,
    ) -> Self {
        Self {
            ingestion_run_id: ingestion_run_id.into(),
            observation_id: observation_id.into(),
            pipeline: pipeline.into(),
        }
    }

    pub fn validate(&self) -> Result<(), ObservationStoreError> {
        validate_non_empty("ingestion_run_id", &self.ingestion_run_id)?;
        validate_non_empty("observation_id", &self.observation_id)?;
        validate_non_empty("pipeline", &self.pipeline)?;
        Ok(())
    }
}

pub(super) fn validate_non_empty(
    field_name: &'static str,
    value: &str,
) -> Result<(), ObservationStoreError> {
    if value.trim().is_empty() {
        return Err(ObservationStoreError::EmptyField(field_name));
    }

    Ok(())
}

pub(super) fn validate_json_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), ObservationStoreError> {
    if !value.is_object() {
        return Err(ObservationStoreError::InvalidJsonObject(field_name));
    }

    Ok(())
}

pub(super) fn validate_score(
    field_name: &'static str,
    value: f64,
) -> Result<(), ObservationStoreError> {
    if !(0.0..=1.0).contains(&value) {
        return Err(ObservationStoreError::InvalidScore(field_name, value));
    }

    Ok(())
}
