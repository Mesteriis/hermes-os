use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;

use hermes_hub_backend::communications::{
    CommunicationIngestionStore, EmailProviderKind, NewIngestionCheckpoint, NewProviderAccount,
    NewProviderAccountSecretBinding, NewRawCommunicationRecord, ProviderAccountSecretPurpose,
};
use hermes_hub_backend::secrets::{
    NewSecretReference, SecretKind, SecretReferenceStore, SecretStoreKind,
};
use hermes_hub_backend::storage::Database;

#[test]
fn email_provider_kind_supports_gmail_icloud_and_raw_imap() {
    assert_eq!(
        EmailProviderKind::try_from("gmail").expect("gmail provider kind"),
        EmailProviderKind::Gmail
    );
    assert_eq!(
        EmailProviderKind::try_from("icloud").expect("icloud provider kind"),
        EmailProviderKind::Icloud
    );
    assert_eq!(
        EmailProviderKind::try_from("imap").expect("imap provider kind"),
        EmailProviderKind::Imap
    );
    assert!(EmailProviderKind::try_from("exchange").is_err());
}

#[tokio::test]
async fn communication_ingestion_registers_email_provider_accounts_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live communication ingestion test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = CommunicationIngestionStore::new(database.pool().expect("configured pool").clone());
    let suffix = unique_suffix();

    let accounts = [
        NewProviderAccount::new(
            format!("acct_gmail_{suffix}"),
            EmailProviderKind::Gmail,
            "Gmail primary",
            format!("gmail-user-{suffix}@example.com"),
        )
        .config(json!({"auth": "oauth", "api": "gmail"})),
        NewProviderAccount::new(
            format!("acct_icloud_{suffix}"),
            EmailProviderKind::Icloud,
            "iCloud Mail",
            format!("icloud-user-{suffix}@icloud.com"),
        )
        .config(json!({"auth": "app_password", "transport": "imap"})),
        NewProviderAccount::new(
            format!("acct_imap_{suffix}"),
            EmailProviderKind::Imap,
            "Generic IMAP",
            format!("imap-user-{suffix}@example.net"),
        )
        .config(json!({"host": "imap.example.net", "port": 993, "tls": true})),
    ];

    for account in accounts {
        let stored = store
            .upsert_provider_account(&account)
            .await
            .expect("store provider account");

        assert_eq!(stored.account_id, account.account_id);
        assert_eq!(stored.provider_kind, account.provider_kind);
        assert_eq!(stored.external_account_id, account.external_account_id);
        assert_eq!(stored.config, account.config);
    }
}

#[tokio::test]
async fn communication_ingestion_records_raw_sources_idempotently_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live communication raw source test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = CommunicationIngestionStore::new(pool.clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_raw_{suffix}");
    let provider_record_id = format!("gmail-message-{suffix}");

    store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Gmail raw source test",
            format!("raw-{suffix}@example.com"),
        ))
        .await
        .expect("store provider account");

    let first = store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_{suffix}"),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:{suffix}"),
                format!("batch_{suffix}"),
                json!({"id": provider_record_id, "provider": "gmail"}),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source": "gmail-api"})),
        )
        .await
        .expect("record raw source");

    let duplicate = store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_duplicate_{suffix}"),
                &account_id,
                "email_message",
                &provider_record_id,
                format!("sha256:different-{suffix}"),
                format!("batch_{suffix}"),
                json!({"id": provider_record_id, "provider": "gmail", "changed": true}),
            )
            .provenance(json!({"source": "retry"})),
        )
        .await
        .expect("record duplicate raw source");

    assert_eq!(duplicate.raw_record_id, first.raw_record_id);
    assert_eq!(duplicate.payload, first.payload);

    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM communication_raw_records
        WHERE account_id = $1
          AND record_kind = 'email_message'
          AND provider_record_id = $2
        "#,
    )
    .bind(&account_id)
    .bind(&provider_record_id)
    .fetch_one(&pool)
    .await
    .expect("raw record count");
    assert_eq!(count, 1);

    let mutation = sqlx::query(
        "UPDATE communication_raw_records SET payload = '{}'::jsonb WHERE raw_record_id = $1",
    )
    .bind(&first.raw_record_id)
    .execute(&pool)
    .await;
    assert!(
        mutation.is_err(),
        "raw provider records must be append-only"
    );
}

#[tokio::test]
async fn communication_ingestion_tracks_checkpoints_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live communication checkpoint test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = CommunicationIngestionStore::new(database.pool().expect("configured pool").clone());
    let suffix = unique_suffix();
    let account_id = format!("acct_checkpoint_{suffix}");

    store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Icloud,
            "iCloud checkpoint test",
            format!("checkpoint-{suffix}@icloud.com"),
        ))
        .await
        .expect("store provider account");

    let saved = store
        .save_checkpoint(&NewIngestionCheckpoint::new(
            &account_id,
            "imap:INBOX",
            json!({
                "provider": "icloud",
                "mailbox": "INBOX",
                "uid_validity": 42,
                "last_seen_uid": 1001
            }),
        ))
        .await
        .expect("save checkpoint");

    assert_eq!(saved.account_id, account_id);
    assert_eq!(saved.stream_id, "imap:INBOX");
    assert_eq!(saved.checkpoint["last_seen_uid"], 1001);

    let updated = store
        .save_checkpoint(&NewIngestionCheckpoint::new(
            &saved.account_id,
            &saved.stream_id,
            json!({
                "provider": "icloud",
                "mailbox": "INBOX",
                "uid_validity": 42,
                "last_seen_uid": 1002
            }),
        ))
        .await
        .expect("update checkpoint");

    assert_eq!(updated.checkpoint["last_seen_uid"], 1002);

    let loaded = store
        .checkpoint(&saved.account_id, &saved.stream_id)
        .await
        .expect("load checkpoint")
        .expect("checkpoint exists");
    assert_eq!(loaded.checkpoint["last_seen_uid"], 1002);
}

#[tokio::test]
async fn communication_ingestion_binds_provider_accounts_to_secret_refs_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live communication secret binding test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_store = SecretReferenceStore::new(pool);
    let suffix = unique_suffix();
    let gmail_account_id = format!("acct_gmail_secret_{suffix}");
    let icloud_account_id = format!("acct_icloud_secret_{suffix}");
    let imap_account_id = format!("acct_imap_secret_{suffix}");

    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &gmail_account_id,
            EmailProviderKind::Gmail,
            "Gmail secret binding",
            format!("gmail-secret-{suffix}@example.com"),
        ))
        .await
        .expect("store gmail account");
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &icloud_account_id,
            EmailProviderKind::Icloud,
            "iCloud secret binding",
            format!("icloud-secret-{suffix}@icloud.com"),
        ))
        .await
        .expect("store icloud account");
    communication_store
        .upsert_provider_account(&NewProviderAccount::new(
            &imap_account_id,
            EmailProviderKind::Imap,
            "IMAP secret binding",
            format!("imap-secret-{suffix}@example.net"),
        ))
        .await
        .expect("store imap account");

    let gmail_secret_ref = format!("secret:gmail:oauth:{suffix}");
    let icloud_secret_ref = format!("secret:icloud:app-password:{suffix}");
    let imap_secret_ref = format!("secret:imap:password:{suffix}");

    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &gmail_secret_ref,
            SecretKind::OauthToken,
            SecretStoreKind::OsKeychain,
            "Gmail OAuth credential",
        ))
        .await
        .expect("store gmail secret reference");
    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &icloud_secret_ref,
            SecretKind::AppPassword,
            SecretStoreKind::OsKeychain,
            "iCloud app-specific password",
        ))
        .await
        .expect("store icloud secret reference");
    secret_store
        .upsert_secret_reference(&NewSecretReference::new(
            &imap_secret_ref,
            SecretKind::Password,
            SecretStoreKind::OsKeychain,
            "Generic IMAP password",
        ))
        .await
        .expect("store imap secret reference");

    let gmail_binding = communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            &gmail_account_id,
            ProviderAccountSecretPurpose::OauthToken,
            &gmail_secret_ref,
        ))
        .await
        .expect("bind gmail oauth secret");
    let icloud_binding = communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            &icloud_account_id,
            ProviderAccountSecretPurpose::ImapPassword,
            &icloud_secret_ref,
        ))
        .await
        .expect("bind icloud imap secret");
    let imap_binding = communication_store
        .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
            &imap_account_id,
            ProviderAccountSecretPurpose::ImapPassword,
            &imap_secret_ref,
        ))
        .await
        .expect("bind generic imap secret");

    assert_eq!(gmail_binding.secret_ref, gmail_secret_ref);
    assert_eq!(icloud_binding.secret_ref, icloud_secret_ref);
    assert_eq!(imap_binding.secret_ref, imap_secret_ref);

    let gmail_bindings = communication_store
        .provider_account_secret_bindings(&gmail_account_id)
        .await
        .expect("load gmail secret bindings");
    assert_eq!(gmail_bindings, vec![gmail_binding]);
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
