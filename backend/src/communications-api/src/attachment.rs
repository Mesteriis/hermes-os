//! Provider-neutral attachment descriptor and safety lifecycle contract.

use crate::{CommunicationAttachmentAnchorIdV1, CommunicationObservationIdV1};

const MAX_FILENAME_BYTES: usize = 512;
const MAX_MEDIA_TYPE_BYTES: usize = 256;
const MAX_DECLARED_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentDescriptorV1 {
    filename: Option<String>,
    media_type: String,
    declared_bytes: u64,
    sha256: Option<[u8; 32]>,
    disposition: AttachmentDispositionV1,
}

impl AttachmentDescriptorV1 {
    pub fn new(
        filename: Option<String>,
        media_type: String,
        declared_bytes: u64,
        sha256: Option<[u8; 32]>,
        disposition: AttachmentDispositionV1,
    ) -> Result<Self, AttachmentDescriptorViolationV1> {
        if filename.as_deref().is_some_and(|value| {
            value.is_empty() || value.len() > MAX_FILENAME_BYTES || !value.is_ascii()
        }) || media_type.is_empty()
            || media_type.len() > MAX_MEDIA_TYPE_BYTES
            || !valid_media_type(&media_type)
            || declared_bytes > MAX_DECLARED_BYTES
        {
            return Err(AttachmentDescriptorViolationV1::InvalidDescriptor);
        }
        Ok(Self {
            filename,
            media_type,
            declared_bytes,
            sha256,
            disposition,
        })
    }

    #[must_use]
    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    #[must_use]
    pub fn media_type(&self) -> &str {
        &self.media_type
    }

    #[must_use]
    pub const fn declared_bytes(&self) -> u64 {
        self.declared_bytes
    }

    #[must_use]
    pub const fn sha256(&self) -> Option<[u8; 32]> {
        self.sha256
    }

    #[must_use]
    pub const fn disposition(&self) -> AttachmentDispositionV1 {
        self.disposition
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentDispositionV1 {
    Attachment,
    Inline,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSafetyStateV1 {
    DescriptorOnly,
    BlobPending,
    BlobAdmitted,
    Quarantined,
    SafeForDelivery,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSafetyTransitionV1 {
    BlobAdmissionRequested,
    BlobAdmitted,
    Quarantined,
    DeclaredClean,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentSafetyTransitionViolationV1 {
    InvalidTransition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachmentSafetyTransitionCommandV1 {
    pub attachment_anchor_id: CommunicationAttachmentAnchorIdV1,
    pub current_state: AttachmentSafetyStateV1,
    pub transition: AttachmentSafetyTransitionV1,
    pub evidence_id: CommunicationObservationIdV1,
    pub observed_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachmentSafetyTransitionDecisionV1 {
    pub attachment_anchor_id: CommunicationAttachmentAnchorIdV1,
    pub expected_state: AttachmentSafetyStateV1,
    pub next_state: AttachmentSafetyStateV1,
    pub evidence_id: CommunicationObservationIdV1,
    pub observed_at_unix_seconds: i64,
}

impl AttachmentSafetyStateV1 {
    pub fn transition(
        self,
        transition: AttachmentSafetyTransitionV1,
    ) -> Result<Self, AttachmentSafetyTransitionViolationV1> {
        match (self, transition) {
            (Self::DescriptorOnly, AttachmentSafetyTransitionV1::BlobAdmissionRequested) => {
                Ok(Self::BlobPending)
            }
            (Self::BlobPending, AttachmentSafetyTransitionV1::BlobAdmitted) => {
                Ok(Self::BlobAdmitted)
            }
            (Self::BlobAdmitted, AttachmentSafetyTransitionV1::DeclaredClean) => {
                Ok(Self::SafeForDelivery)
            }
            (
                Self::DescriptorOnly | Self::BlobPending | Self::BlobAdmitted,
                AttachmentSafetyTransitionV1::Quarantined,
            ) => Ok(Self::Quarantined),
            (
                Self::DescriptorOnly | Self::BlobPending | Self::BlobAdmitted,
                AttachmentSafetyTransitionV1::Rejected,
            ) => Ok(Self::Rejected),
            _ => Err(AttachmentSafetyTransitionViolationV1::InvalidTransition),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentDescriptorViolationV1 {
    InvalidDescriptor,
}

fn valid_media_type(value: &str) -> bool {
    value.is_ascii() && value.contains('/') && !value.contains([' ', '\t', '\n', '\r', ';'])
}
