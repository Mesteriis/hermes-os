#![forbid(unsafe_code)]

mod join;
mod lifecycle;
mod policy;

pub use join::{
    AttachmentPreviewCustodyDelegationIntentV1, AttachmentPreviewEvidenceJoinV1,
    AttachmentPreviewJoinErrorV1, AttachmentPreviewRequestFactV1, AttachmentPreviewSafetyFactV1,
    AttachmentPreviewScanCandidateFactV1,
};
pub use lifecycle::{AttachmentPreviewTransitionErrorV1, transition_attachment_preview_v1};
pub use policy::{
    AttachmentPreviewOutputPolicyErrorV1, preview_output_limit_v1, validate_preview_output_v1,
};

pub const PACKAGE: &str = "hermes-attachment-preview-core";
