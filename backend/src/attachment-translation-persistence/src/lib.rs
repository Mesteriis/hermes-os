#![forbid(unsafe_code)]

mod model;
mod outbox;
mod realtime;
mod repository;
mod schema;

pub use model::{
    AttachmentTranslationInboxResultV1, AttachmentTranslationInferenceResultV1,
    AttachmentTranslationMaterializationResultV1, AttachmentTranslationPersistenceErrorV1,
    AttachmentTranslationSourceAuthorityV1, AttachmentTranslationSourceResultV1,
    CreateAttachmentTranslationOutcomeV1, CreateAttachmentTranslationRunV1,
    PersistedAttachmentTranslationRunV1, UnpublishedAttachmentTranslationEventV1,
};
pub use realtime::AttachmentTranslationRealtimeTransitionV1;
pub use repository::AttachmentTranslationPersistenceV1;
pub use schema::{
    ATTACHMENT_TRANSLATION_SCHEMA_V1, ATTACHMENT_TRANSLATION_STORAGE_BUNDLE_REVISION_V1,
    attachment_translation_storage_bundle_v1,
};

pub const PACKAGE: &str = "hermes-attachment-translation-persistence";
