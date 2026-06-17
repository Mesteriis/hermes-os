use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) use chrono::Utc;
pub(crate) use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, EmailProviderKind, NewIngestionCheckpoint, NewProviderAccount,
    NewProviderAccountSecretBinding, NewRawCommunicationRecord, ProviderAccountSecretPurpose,
    ProviderCredentialError, ProviderCredentialReader,
};
pub(crate) use hermes_hub_backend::platform::secrets::{
    InMemorySecretResolver, NewSecretReference, SecretKind, SecretReferenceStore,
    SecretResolutionError, SecretResolver, SecretStoreKind,
};
pub(crate) use hermes_hub_backend::platform::storage::Database;
pub(crate) use serde_json::json;

pub(crate) fn test_database_url(test_name: &str) -> Option<String> {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live {test_name}: HERMES_TEST_DATABASE_URL is not set");
        return None;
    };
    Some(database_url)
}

pub(crate) async fn connect_database(test_name: &str) -> Option<Database> {
    let database_url = test_database_url(test_name)?;
    Some(
        Database::connect(Some(&database_url))
            .await
            .expect("database connection"),
    )
}

pub(crate) fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
