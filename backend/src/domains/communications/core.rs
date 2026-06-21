mod accounts;
mod checkpoints;
mod errors;
mod models;
mod provider_store;
mod raw_records;
mod rows;
mod secrets;
mod store;
mod validation;

pub use errors::CommunicationIngestionError;
pub use errors::ProviderCredentialError;
pub use models::{
    CommunicationProviderKind, DeletedProviderAccount, EmailProviderKind, IngestionCheckpoint,
    NewIngestionCheckpoint, NewProviderAccount, NewProviderAccountSecretBinding,
    NewRawCommunicationRecord, ProviderAccount, ProviderAccountSecretBinding,
    ProviderAccountSecretPurpose, ProviderAccountUsage, ProviderCredential,
    StoredRawCommunicationRecord,
};
pub use provider_store::CommunicationProviderAccountStore as CommunicationProviderAccountPort;
pub use provider_store::{
    CommunicationProviderAccountStore, CommunicationProviderSecretBindingStore,
};
pub use secrets::ProviderCredentialReader;
pub use store::CommunicationIngestionStore;
pub use store::CommunicationIngestionStore as CommunicationIngestionPort;
