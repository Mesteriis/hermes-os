mod accounts;
mod checkpoints;
mod errors;
mod models;
mod raw_records;
mod rows;
mod secrets;
mod store;
mod validation;

pub use errors::{CommunicationIngestionError, ProviderCredentialError};
pub use models::{
    CommunicationProviderKind, DeletedProviderAccount, EmailProviderKind, IngestionCheckpoint,
    NewIngestionCheckpoint, NewProviderAccount, NewProviderAccountSecretBinding,
    NewRawCommunicationRecord, ProviderAccount, ProviderAccountSecretBinding,
    ProviderAccountSecretPurpose, ProviderAccountUsage, ProviderCredential,
    StoredRawCommunicationRecord,
};
pub use secrets::ProviderCredentialReader;
pub use store::CommunicationIngestionStore;
