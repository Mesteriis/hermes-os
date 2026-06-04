use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use hermes_hub_backend::secrets::{
    NewSecretReference, SecretKind, SecretReferenceStore, SecretStoreKind,
};
use hermes_hub_backend::storage::Database;

#[test]
fn secret_reference_enums_reject_unsupported_values() {
    assert_eq!(
        SecretKind::try_from("oauth_token").expect("oauth token kind"),
        SecretKind::OauthToken
    );
    assert_eq!(
        SecretKind::try_from("app_password").expect("app password kind"),
        SecretKind::AppPassword
    );
    assert_eq!(
        SecretStoreKind::try_from("os_keychain").expect("os keychain store kind"),
        SecretStoreKind::OsKeychain
    );
    assert!(SecretKind::try_from("plain_text").is_err());
    assert!(SecretStoreKind::try_from("postgres").is_err());
}

#[tokio::test]
async fn secret_references_store_only_metadata_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live secret reference test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = SecretReferenceStore::new(database.pool().expect("configured pool").clone());
    let suffix = unique_suffix();
    let secret_ref = format!("secret:gmail:oauth:{suffix}");

    let stored = store
        .upsert_secret_reference(
            &NewSecretReference::new(
                &secret_ref,
                SecretKind::OauthToken,
                SecretStoreKind::OsKeychain,
                "Gmail OAuth credential",
            )
            .metadata(json!({
                "service": "hermes-hub",
                "account": format!("gmail-user-{suffix}@example.com")
            })),
        )
        .await
        .expect("store secret reference");

    assert_eq!(stored.secret_ref, secret_ref);
    assert_eq!(stored.secret_kind, SecretKind::OauthToken);
    assert_eq!(stored.store_kind, SecretStoreKind::OsKeychain);
    assert_eq!(stored.metadata["service"], "hermes-hub");

    let loaded = store
        .secret_reference(&stored.secret_ref)
        .await
        .expect("load secret reference")
        .expect("secret reference exists");
    assert_eq!(loaded, stored);
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
