use serde::Serialize;
use thiserror::Error;

#[derive(Clone, Debug, Serialize)]
pub struct AttachmentClassification {
    pub attachment_id: String,
    pub category: AttachmentCategory,
    pub is_executable: bool,
    pub is_archive: bool,
    pub is_document: bool,
    pub risk_level: RiskLevel,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentCategory {
    Invoice,
    Contract,
    LegalDocument,
    TaxDocument,
    IdentityDocument,
    BankDocument,
    Certificate,
    Report,
    Presentation,
    Spreadsheet,
    SourceCode,
    Image,
    Screenshot,
    Archive,
    Unknown,
}

impl AttachmentCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Invoice => "invoice",
            Self::Contract => "contract",
            Self::LegalDocument => "legal_document",
            Self::TaxDocument => "tax_document",
            Self::IdentityDocument => "identity_document",
            Self::BankDocument => "bank_document",
            Self::Certificate => "certificate",
            Self::Report => "report",
            Self::Presentation => "presentation",
            Self::Spreadsheet => "spreadsheet",
            Self::SourceCode => "source_code",
            Self::Image => "image",
            Self::Screenshot => "screenshot",
            Self::Archive => "archive",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

#[derive(Debug, Error)]
pub enum AttachmentIntelligenceError {
    #[error("attachment not found")]
    NotFound,
}
