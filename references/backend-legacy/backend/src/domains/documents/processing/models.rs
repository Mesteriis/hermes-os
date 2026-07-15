use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use super::errors::DocumentProcessingError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentProcessingStep {
    ExtractText,
    Ocr,
}

impl DocumentProcessingStep {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::ExtractText => "extract_text",
            Self::Ocr => "ocr",
        }
    }

    pub(super) fn parse(raw: &str) -> Result<Self, DocumentProcessingError> {
        match raw {
            "extract_text" => Ok(Self::ExtractText),
            "ocr" => Ok(Self::Ocr),
            _ => Err(DocumentProcessingError::InvalidStep(raw.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentProcessingStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Skipped,
}

impl DocumentProcessingStatus {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }

    pub(super) fn parse(raw: &str) -> Result<Self, DocumentProcessingError> {
        match raw {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "skipped" => Ok(Self::Skipped),
            _ => Err(DocumentProcessingError::InvalidStatus(raw.to_owned())),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentArtifactKind {
    ExtractedText,
    OcrText,
}

impl DocumentArtifactKind {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::ExtractedText => "extracted_text",
            Self::OcrText => "ocr_text",
        }
    }

    pub(super) fn parse(raw: &str) -> Result<Self, DocumentProcessingError> {
        match raw {
            "extracted_text" => Ok(Self::ExtractedText),
            "ocr_text" => Ok(Self::OcrText),
            _ => Err(DocumentProcessingError::InvalidArtifactKind(raw.to_owned())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DocumentProcessingJob {
    pub job_id: String,
    pub document_id: String,
    pub step: DocumentProcessingStep,
    pub status: DocumentProcessingStatus,
    pub attempts: i32,
    pub max_attempts: i32,
    pub last_error_summary: Option<String>,
    pub queued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DocumentProcessingArtifact {
    pub artifact_id: String,
    pub document_id: String,
    pub job_id: String,
    pub artifact_kind: DocumentArtifactKind,
    pub content_sha256: String,
    pub text_content: Option<String>,
    pub storage_kind: Option<String>,
    pub storage_path: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DocumentProcessingRecord {
    pub document_id: String,
    pub jobs: Vec<DocumentProcessingJob>,
    pub artifacts: Vec<DocumentProcessingArtifact>,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct DocumentProcessingRunReport {
    pub jobs_seen: i64,
    pub jobs_queued: i64,
    pub jobs_succeeded: i64,
    pub jobs_failed: i64,
    pub jobs_skipped: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentProcessingRetryCommand {
    pub command_id: String,
    pub job_id: String,
    pub actor_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DocumentProcessingRetryCommandResult {
    pub job_id: String,
    pub status: DocumentProcessingStatus,
    pub event_id: String,
}
