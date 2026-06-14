mod classification;
mod file_kinds;
mod models;

#[cfg(test)]
mod tests;

use crate::domains::mail::storage::StoredMailAttachmentWithBlob;

use classification::classify_by_name_and_type;
use file_kinds::{is_archive_type, is_document_type, is_executable_type};

pub use models::{
    AttachmentCategory, AttachmentClassification, AttachmentIntelligenceError, RiskLevel,
};

pub struct AttachmentIntelligenceService;

impl AttachmentIntelligenceService {
    /// Classify an attachment by filename and content type.
    pub fn classify(attachment: &StoredMailAttachmentWithBlob) -> AttachmentClassification {
        let filename = attachment.attachment.filename.as_deref().unwrap_or("");
        let content_type = &attachment.attachment.content_type;
        let filename_lower = filename.to_lowercase();

        let category = classify_by_name_and_type(&filename_lower, content_type);
        let is_executable = is_executable_type(content_type, &filename_lower);
        let is_archive = is_archive_type(content_type, &filename_lower);
        let is_document = is_document_type(content_type, &filename_lower);
        let risk_level = if is_executable {
            RiskLevel::High
        } else if is_archive {
            RiskLevel::Medium
        } else {
            RiskLevel::Safe
        };

        let size_mb = attachment.attachment.size_bytes as f64 / 1_048_576.0;

        AttachmentClassification {
            attachment_id: attachment.attachment.attachment_id.clone(),
            category,
            is_executable,
            is_archive,
            is_document,
            risk_level,
            summary: format!("{} ({:.1} MB) - {}", filename, size_mb, category.as_str()),
        }
    }
}
