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

pub use blob_store::LocalCommunicationBlobStore as LocalCommunicationBlobPort;
pub use blob_store::{LocalCommunicationBlob, LocalCommunicationBlobStore};
pub use errors::{AttachmentSafetyScanError, CommunicationStorageError};
pub use imports::new_communication_attachment_import_id;
pub use models::{
    CommunicationAttachmentDisposition, ImportedCommunicationAttachment,
    NewCommunicationAttachment, NewCommunicationAttachmentImport, NewCommunicationBlob,
    StoredCommunicationAttachment, StoredCommunicationAttachmentWithBlob, StoredCommunicationBlob,
};
pub use scanner::{
    AttachmentSafetyScanReport, AttachmentSafetyScanRequest, AttachmentSafetyScanStatus,
    AttachmentSafetyScanner, HeuristicAttachmentSafetyScanner, NoopAttachmentSafetyScanner,
    scan_attachment_with_clamav, scan_attachment_with_configured_clamav,
};
pub use store::CommunicationStorageStore;
pub use store::CommunicationStorageStore as CommunicationBlobMetadataPort;
