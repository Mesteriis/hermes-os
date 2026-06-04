use std::fs;

use chrono::Utc;
use tempfile::tempdir;

use hermes_hub_backend::secret_vault::EncryptedSecretVault;
use hermes_hub_backend::secrets::{
    ResolvedSecret, SecretKind, SecretReference, SecretResolutionError, SecretResolver,
    SecretStoreKind,
};

#[test]
fn encrypted_vault_persists_secrets_without_plaintext_leakage() {
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
        .expect("resolve secret");
    assert_eq!(resolved.expose_for_runtime(), "gmail-refresh-token");
    assert_eq!(
        format!("{resolved:?}"),
        "ResolvedSecret { value: \"<redacted>\" }"
    );
}

#[test]
fn encrypted_vault_rejects_wrong_master_key() {
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
        .expect_err("wrong master key must fail");

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
