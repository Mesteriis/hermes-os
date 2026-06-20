mod blob_store;
mod constants;
mod errors;
mod ids;
mod imports;
mod models;
mod rows;
mod scanner;
mod store;
mod validation;

pub use blob_store::{LocalMailBlob, LocalMailBlobStore};
pub use errors::{AttachmentSafetyScanError, MailStorageError};
pub use imports::new_communication_attachment_import_id;
pub use models::{
    ImportedCommunicationAttachment, MailAttachmentDisposition, NewCommunicationAttachmentImport,
    NewMailAttachment, NewMailBlob, StoredMailAttachment, StoredMailAttachmentWithBlob,
    StoredMailBlob,
};
pub use scanner::{
    AttachmentSafetyScanReport, AttachmentSafetyScanRequest, AttachmentSafetyScanStatus,
    AttachmentSafetyScanner, HeuristicAttachmentSafetyScanner, NoopAttachmentSafetyScanner,
};
pub use store::MailStorageStore;
