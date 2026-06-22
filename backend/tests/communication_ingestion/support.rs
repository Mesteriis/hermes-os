use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

pub(crate) use chrono::Utc;
pub(crate) use hermes_hub_backend::domains::communications::core::{
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

pub(crate) async fn test_database_url(test_name: &str) -> Option<String> {
    let _ = test_name;
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    Some(database_url)
}

pub(crate) async fn connect_database(test_name: &str) -> Option<Database> {
    let database_url = test_database_url(test_name).await?;
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
