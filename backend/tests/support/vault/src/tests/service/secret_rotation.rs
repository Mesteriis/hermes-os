use std::os::unix::fs::PermissionsExt;

use hermes_vault_key_provider::WrappingKeyProvider;
use hermes_vault_key_provider_file::FileWrappingKeyProvider;
use hermes_vault_protocol::{SecretClassV1, VaultActionV1, VaultPurposeRequestV1};
use hermes_vault_store_sqlcipher::{SecretRecordScope, VaultStore};
use tempfile::TempDir;

#[test]
fn replacement_advances_one_revision_and_removes_the_prior_record() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let store = initialize_store(&temporary);
    let purpose = credential_purpose();
    let first_scope = scope(&purpose, 1);
    let first_record = store
        .store_secret(&first_scope, b"credential-revision-one")
        .expect("first credential");
    let second_scope = scope(&purpose, 2);

    let second_record = store
        .replace_secret(
            &first_record,
            &first_scope,
            &second_scope,
            b"credential-revision-two",
        )
        .expect("atomic replacement");
    assert_eq!(
        store
            .resolve_scoped_secret(&second_record, &second_scope)
            .expect("replacement credential")
            .as_slice(),
        b"credential-revision-two"
    );
    assert!(
        store
            .resolve_scoped_secret(&first_record, &first_scope)
            .is_err()
    );
}

#[test]
fn replacement_rejects_a_non_sequential_revision_without_destroying_the_prior_record() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let store = initialize_store(&temporary);
    let purpose = credential_purpose();
    let first_scope = scope(&purpose, 1);
    let first_record = store
        .store_secret(&first_scope, b"credential-revision-one")
        .expect("first credential");

    assert!(
        store
            .replace_secret(
                &first_record,
                &first_scope,
                &scope(&purpose, 3),
                b"credential-revision-three",
            )
            .is_err()
    );
    assert_eq!(
        store
            .resolve_scoped_secret(&first_record, &first_scope)
            .expect("prior credential remains")
            .as_slice(),
        b"credential-revision-one"
    );
}

#[test]
fn one_scope_revision_has_exactly_one_active_secret_record() {
    let temporary = TempDir::new().expect("temporary Vault directory");
    std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o700))
        .expect("private temporary Vault directory");
    let store = initialize_store(&temporary);
    let scope = scope(&credential_purpose(), 1);

    store
        .store_secret(&scope, b"credential-revision-one")
        .expect("first credential");
    assert!(store.store_secret(&scope, b"duplicate-credential").is_err());
}

fn initialize_store(temporary: &TempDir) -> VaultStore {
    let provider = FileWrappingKeyProvider::new(&temporary.path().join("wrapping-key.bin"));
    let key = provider.load_or_create().expect("file wrapping key");
    VaultStore::initialize(
        &temporary.path().join("vault.db"),
        &temporary.path().join("vault.anchor"),
        "vault-instance",
        &key,
    )
    .expect("Vault store")
}

fn credential_purpose() -> VaultPurposeRequestV1 {
    VaultPurposeRequestV1::new(
        "mail.credential".to_owned(),
        "account-a".to_owned(),
        vec![SecretClassV1::ProviderCredential],
        vec![VaultActionV1::Resolve, VaultActionV1::Create],
        60,
    )
    .expect("typed purpose")
}

fn scope(purpose: &VaultPurposeRequestV1, revision: u64) -> SecretRecordScope {
    SecretRecordScope::new(
        "mail".to_owned(),
        purpose,
        SecretClassV1::ProviderCredential,
        revision,
    )
    .expect("record scope")
}
