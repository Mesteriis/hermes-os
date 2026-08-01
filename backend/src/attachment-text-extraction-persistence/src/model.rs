use hermes_attachment_text_extraction_core::{
    AttachmentTextExtractionErrorV1, AttachmentTextExtractionRequestV1,
    AttachmentTextExtractionStateV1, AttachmentTextExtractionStatusV1, AttachmentTextFormatV1,
};
use sha2::{Digest, Sha256};

use crate::AttachmentTextExtractionPersistenceErrorV1;

pub const ATTACHMENT_TEXT_EXTRACTION_REALTIME_LIMIT_V1: u32 = 512;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateAttachmentTextExtractionRunV1 {
    pub logical_owner_id: String,
    pub operation_id: [u8; 16],
    pub attachment_anchor_id: [u8; 16],
    pub created_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedAttachmentTextExtractionRunV1 {
    pub logical_owner_id: String,
    pub request: AttachmentTextExtractionRequestV1,
    pub request_fingerprint: [u8; 32],
    pub status: AttachmentTextExtractionStatusV1,
    pub created_at_unix_millis: i64,
    pub updated_at_unix_millis: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CreateAttachmentTextExtractionRunOutcomeV1 {
    Created(PersistedAttachmentTextExtractionRunV1),
    Replayed(PersistedAttachmentTextExtractionRunV1),
    OperationCollision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PersistedAttachmentTextArtifactV1 {
    pub run_id: [u8; 16],
    pub derived_reference_id: [u8; 16],
    pub derived_receipt_sha256: [u8; 32],
    pub source_receipt_sha256: [u8; 32],
    pub parser_identity_sha256: [u8; 32],
    pub format: AttachmentTextFormatV1,
    pub extracted_size_bytes: u64,
    pub extraction_truncated: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextExtractionRealtimeTransitionV1 {
    pub sequence: u64,
    pub run_id: [u8; 16],
    pub state: AttachmentTextExtractionStateV1,
    pub state_revision: u64,
    pub format: Option<AttachmentTextFormatV1>,
    pub extracted_size_bytes: u64,
    pub extraction_truncated: bool,
    pub error: Option<AttachmentTextExtractionErrorV1>,
    pub occurred_at_unix_millis: i64,
}

#[must_use]
pub fn attachment_text_extraction_run_id_v1(
    logical_owner_id: &str,
    operation_id: [u8; 16],
) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(b"hermes.attachment-text-extraction.run.v1\0");
    hasher.update(logical_owner_id.as_bytes());
    hasher.update(operation_id);
    hasher.finalize()[..16].try_into().expect("digest prefix")
}

#[must_use]
pub fn attachment_text_extraction_request_fingerprint_v1(
    attachment_anchor_id: [u8; 16],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"hermes.attachment-text-extraction.request.v1\0");
    hasher.update(attachment_anchor_id);
    hasher.finalize().into()
}

pub(crate) fn validate_create(
    create: &CreateAttachmentTextExtractionRunV1,
) -> Result<(), AttachmentTextExtractionPersistenceErrorV1> {
    if !valid_owner(&create.logical_owner_id)
        || !valid_id16(&create.operation_id)
        || !valid_id16(&create.attachment_anchor_id)
        || create.created_at_unix_millis <= 0
    {
        return Err(AttachmentTextExtractionPersistenceErrorV1::InvalidInput);
    }
    Ok(())
}

pub(crate) const fn state_code(value: AttachmentTextExtractionStateV1) -> i16 {
    match value {
        AttachmentTextExtractionStateV1::Accepted => 1,
        AttachmentTextExtractionStateV1::AwaitingEvidence => 2,
        AttachmentTextExtractionStateV1::Extracting => 3,
        AttachmentTextExtractionStateV1::Ready => 4,
        AttachmentTextExtractionStateV1::Unsupported => 5,
        AttachmentTextExtractionStateV1::Rejected => 6,
    }
}

pub(crate) fn state_from_code(
    value: i16,
) -> Result<AttachmentTextExtractionStateV1, AttachmentTextExtractionPersistenceErrorV1> {
    match value {
        1 => Ok(AttachmentTextExtractionStateV1::Accepted),
        2 => Ok(AttachmentTextExtractionStateV1::AwaitingEvidence),
        3 => Ok(AttachmentTextExtractionStateV1::Extracting),
        4 => Ok(AttachmentTextExtractionStateV1::Ready),
        5 => Ok(AttachmentTextExtractionStateV1::Unsupported),
        6 => Ok(AttachmentTextExtractionStateV1::Rejected),
        _ => Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) const fn format_code(value: AttachmentTextFormatV1) -> i16 {
    match value {
        AttachmentTextFormatV1::PlainUtf8 => 1,
        AttachmentTextFormatV1::Pdf => 2,
        AttachmentTextFormatV1::Docx => 3,
        AttachmentTextFormatV1::Ocr => 4,
    }
}

pub(crate) fn format_from_code(
    value: i16,
) -> Result<AttachmentTextFormatV1, AttachmentTextExtractionPersistenceErrorV1> {
    match value {
        1 => Ok(AttachmentTextFormatV1::PlainUtf8),
        2 => Ok(AttachmentTextFormatV1::Pdf),
        3 => Ok(AttachmentTextFormatV1::Docx),
        4 => Ok(AttachmentTextFormatV1::Ocr),
        _ => Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) const fn error_code(value: AttachmentTextExtractionErrorV1) -> i16 {
    match value {
        AttachmentTextExtractionErrorV1::NotSafe => 1,
        AttachmentTextExtractionErrorV1::Unsupported => 2,
        AttachmentTextExtractionErrorV1::SourceTooLarge => 3,
        AttachmentTextExtractionErrorV1::InvalidContent => 4,
        AttachmentTextExtractionErrorV1::ParserUnavailable => 5,
        AttachmentTextExtractionErrorV1::ParserFailed => 6,
        AttachmentTextExtractionErrorV1::CustodyRejected => 7,
        AttachmentTextExtractionErrorV1::Unavailable => 8,
    }
}

pub(crate) fn error_from_code(
    value: i16,
) -> Result<AttachmentTextExtractionErrorV1, AttachmentTextExtractionPersistenceErrorV1> {
    match value {
        1 => Ok(AttachmentTextExtractionErrorV1::NotSafe),
        2 => Ok(AttachmentTextExtractionErrorV1::Unsupported),
        3 => Ok(AttachmentTextExtractionErrorV1::SourceTooLarge),
        4 => Ok(AttachmentTextExtractionErrorV1::InvalidContent),
        5 => Ok(AttachmentTextExtractionErrorV1::ParserUnavailable),
        6 => Ok(AttachmentTextExtractionErrorV1::ParserFailed),
        7 => Ok(AttachmentTextExtractionErrorV1::CustodyRejected),
        8 => Ok(AttachmentTextExtractionErrorV1::Unavailable),
        _ => Err(AttachmentTextExtractionPersistenceErrorV1::InvalidRow),
    }
}

pub(crate) fn valid_owner(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128
}

pub(crate) fn valid_id16(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

pub(crate) fn valid_sha256(value: &[u8; 32]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_id_is_owner_scoped_and_request_fingerprint_is_anchor_scoped() {
        let operation = [7; 16];
        assert_ne!(
            attachment_text_extraction_run_id_v1("alice", operation),
            attachment_text_extraction_run_id_v1("bob", operation)
        );
        assert_ne!(
            attachment_text_extraction_request_fingerprint_v1([1; 16]),
            attachment_text_extraction_request_fingerprint_v1([2; 16])
        );
    }
}
