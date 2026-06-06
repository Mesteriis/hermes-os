use std::env;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use tempfile::tempdir;

use hermes_hub_backend::secret_vault::{DatabaseEncryptedSecretVault, EncryptedSecretVault};
use hermes_hub_backend::secrets::{
    NewSecretReference, ResolvedSecret, SecretKind, SecretReference, SecretReferenceStore,
    SecretResolutionError, SecretResolver, SecretStoreKind,
};
use hermes_hub_backend::storage::Database;

#[tokio::test]
async fn encrypted_vault_persists_secrets_without_plaintext_leakage() {
    let directory = tempdir().expect("tempdir");
    let vault_path = directory.path().join("hermes-secrets.vault.json");
    let vault = EncryptedSecretVault::new(
        &vault_path,
        ResolvedSecret::new("correct horse battery staple").expect("vault key"),
    );

    vault
        .store_secret("secret:test:oauth", "gmail-refresh-token")
        .expect("store secret");

    let file_contents = fs::read_to_string(&vault_path).expect("vault file");
    assert!(!file_contents.contains("gmail-refresh-token"));

    let resolved = vault
        .resolve(&secret_reference("secret:test:oauth"))
        .await
        .expect("resolve secret");
    assert_eq!(resolved.expose_for_runtime(), "gmail-refresh-token");
    assert_eq!(
        format!("{resolved:?}"),
        "ResolvedSecret { value: \"<redacted>\" }"
    );
}

#[tokio::test]
async fn encrypted_vault_rejects_wrong_master_key() {
    let directory = tempdir().expect("tempdir");
    let vault_path = directory.path().join("hermes-secrets.vault.json");
    let vault = EncryptedSecretVault::new(
        &vault_path,
        ResolvedSecret::new("correct horse battery staple").expect("vault key"),
    );
    vault
        .store_secret("secret:test:password", "imap-app-password")
        .expect("store secret");

    let wrong_key_vault = EncryptedSecretVault::new(
        &vault_path,
        ResolvedSecret::new("wrong master key").expect("wrong vault key"),
    );
    let error = wrong_key_vault
        .resolve(&secret_reference("secret:test:password"))
        .await
        .expect_err("wrong master key must fail");

    assert!(matches!(error, SecretResolutionError::StoreFailure { .. }));
}

#[tokio::test]
async fn database_encrypted_vault_persists_ciphertext_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live database encrypted vault test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let secret_store = SecretReferenceStore::new(pool.clone());
    let vault = DatabaseEncryptedSecretVault::new(
        pool.clone(),
        ResolvedSecret::new("database vault key").expect("vault key"),
    );
    let secret_ref = format!("secret:test:database-vault:{}", unique_suffix());

    let reference = secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &secret_ref,
            SecretKind::Password,
            SecretStoreKind::DatabaseEncryptedVault,
            "Database encrypted test secret",
        ))
        .await
        .expect("store database vault secret reference");
    vault
        .store_secret(&secret_ref, "database-vault-secret")
        .await
        .expect("store database vault secret");

    let ciphertext: String = sqlx::query_scalar(
        r#"
        SELECT ciphertext
        FROM encrypted_secret_vault_entries
        WHERE secret_ref = $1
        "#,
    )
    .bind(&secret_ref)
    .fetch_one(&pool)
    .await
    .expect("load stored ciphertext");
    assert!(!ciphertext.contains("database-vault-secret"));

    let resolved = vault
        .resolve(&reference)
        .await
        .expect("resolve database vault secret");
    assert_eq!(resolved.expose_for_runtime(), "database-vault-secret");
}

#[tokio::test]
async fn database_encrypted_vault_rejects_wrong_master_key_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live database encrypted vault wrong key test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let secret_store = SecretReferenceStore::new(pool.clone());
    let vault = DatabaseEncryptedSecretVault::new(
        pool.clone(),
        ResolvedSecret::new("database vault key").expect("vault key"),
    );
    let wrong_key_vault = DatabaseEncryptedSecretVault::new(
        pool,
        ResolvedSecret::new("wrong database vault key").expect("wrong vault key"),
    );
    let secret_ref = format!("secret:test:database-vault-wrong-key:{}", unique_suffix());

    let reference = secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &secret_ref,
            SecretKind::Password,
            SecretStoreKind::DatabaseEncryptedVault,
            "Database encrypted wrong key test secret",
        ))
        .await
        .expect("store database vault secret reference");
    vault
        .store_secret(&secret_ref, "database-vault-secret")
        .await
        .expect("store database vault secret");

    let error = wrong_key_vault
        .resolve(&reference)
        .await
        .expect_err("wrong database vault key must fail");

    assert!(matches!(error, SecretResolutionError::StoreFailure { .. }));
}

fn secret_reference(secret_ref: &str) -> SecretReference {
    let now = Utc::now();

    SecretReference {
        secret_ref: secret_ref.to_owned(),
        secret_kind: SecretKind::OauthToken,
        store_kind: SecretStoreKind::EncryptedVault,
        label: "test secret".to_owned(),
        metadata: serde_json::json!({}),
        created_at: now,
        updated_at: now,
    }
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
