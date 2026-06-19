use serde_json::json;
use sqlx::Row;
use tempfile::tempdir;
use tower::ServiceExt;

use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::domains::calendar::events::CalendarAccountStore;
use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, EmailProviderKind, ProviderAccountSecretPurpose,
};
use hermes_hub_backend::platform::config::AppConfig;
use hermes_hub_backend::platform::secrets::{SecretKind, SecretReferenceStore, SecretResolver};
use hermes_hub_backend::platform::storage::Database;
use hermes_hub_backend::vault::{HostVault, HostVaultConfig, SecretEntryContext};
use testkit::context::TestContext;

use super::support::{
    LOCAL_API_TOKEN, json_request_with_token_and_actor, unlock_test_vault,
    wait_for_calendar_account, wait_for_manifest_metadata_key, wait_for_provider_account,
    wait_for_provider_account_secret_binding, wait_for_secret_reference,
};

#[tokio::test]
async fn startup_reconciles_icloud_account_from_host_vault_manifest_after_postgres_metadata_wipe() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let config = AppConfig::from_pairs([
        ("HERMES_LOCAL_API_SECRET", LOCAL_API_TOKEN),
        ("HERMES_DEV_MODE", "true"),
        (
            "HERMES_VAULT_HOME",
            vault_home.to_str().expect("vault path"),
        ),
        (
            "HERMES_DEV_KEY_PATH",
            dev_key_path.to_str().expect("dev key path"),
        ),
        ("DATABASE_URL", database_url.as_str()),
    ])
    .expect("config");
    let app = build_router_with_database(config.clone(), database.clone());
    unlock_test_vault(app.clone()).await;

    let account_id = "icloud-recover";
    let secret_ref = "secret:provider-account:icloud-recover:imap_password";
    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/email-accounts/imap",
            json!({
                "account_id": account_id,
                "provider_kind": "icloud",
                "display_name": "Recovered iCloud",
                "external_account_id": "recover@icloud.com",
                "host": "imap.mail.me.com",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "recover@icloud.com",
                "password": "icloud-app-password",
                "secret_kind": "app_password"
            }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let pool = database.pool().expect("configured pool").clone();
    let vault = HostVault::new(HostVaultConfig {
        home: vault_home.clone(),
        dev_mode: true,
        dev_key_path: dev_key_path.clone(),
    })
    .expect("host vault");
    vault.unlock_existing().expect("unlock host vault");
    vault
        .upsert_account_secret_manifest_entry(
            secret_ref,
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id,
                purpose: ProviderAccountSecretPurpose::ImapPassword.as_str(),
                secret_kind: SecretKind::AppPassword.as_str(),
                label: "IMAP password",
                metadata: &json!({
                    "provider": "icloud",
                    "account_id": account_id
                }),
            },
        )
        .expect("write sparse manifest entry");

    let _enriching_app = build_router_with_database(config.clone(), database.clone());
    wait_for_manifest_metadata_key(&vault, secret_ref, "display_name").await;

    sqlx::query("DELETE FROM calendar_accounts WHERE account_id = $1")
        .bind(format!("icloud-calendar:{account_id}"))
        .execute(&pool)
        .await
        .expect("delete calendar metadata");
    sqlx::query("DELETE FROM communication_provider_account_secret_refs WHERE account_id = $1")
        .bind(account_id)
        .execute(&pool)
        .await
        .expect("delete secret binding");
    sqlx::query("DELETE FROM communication_provider_accounts WHERE account_id = $1")
        .bind(account_id)
        .execute(&pool)
        .await
        .expect("delete provider account");
    sqlx::query("DELETE FROM secret_references WHERE secret_ref = $1")
        .bind(secret_ref)
        .execute(&pool)
        .await
        .expect("delete secret reference");

    assert!(
        CommunicationIngestionStore::new(pool.clone())
            .provider_account(account_id)
            .await
            .expect("load deleted account")
            .is_none()
    );

    let restarted_database = Database::connect(Some(&database_url))
        .await
        .expect("restarted database connection");
    let _restarted_app = build_router_with_database(config, restarted_database.clone());
    let restarted_pool = restarted_database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(restarted_pool.clone());
    let secret_store = SecretReferenceStore::new(restarted_pool.clone());

    let account = wait_for_provider_account(&communication_store, account_id).await;
    assert_eq!(account.provider_kind, EmailProviderKind::Icloud);
    assert_eq!(account.display_name, "Recovered iCloud");
    assert_eq!(account.external_account_id, "recover@icloud.com");
    assert_eq!(
        account.config["connected_services"],
        json!(["mail", "calendar", "contacts"])
    );
    let provider_account_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'vault'
           AND entity_kind = 'communication_provider_account'
           AND entity_id = $1
           AND relationship_kind = 'upsert'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(account_id)
    .fetch_one(&restarted_pool)
    .await
    .expect("provider account observation link");
    let provider_account_observation = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
    )
    .bind(&provider_account_observation_id)
    .fetch_one(&restarted_pool)
    .await
    .expect("provider account observation");
    assert_eq!(
        provider_account_observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "vault_source"
    );
    assert_eq!(
        provider_account_observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "COMMUNICATION_PROVIDER_ACCOUNT"
    );

    let reference = wait_for_secret_reference(&secret_store, secret_ref).await;
    assert_eq!(reference.store_kind.as_str(), "host_vault");
    assert_eq!(reference.secret_kind, SecretKind::AppPassword);

    let binding = wait_for_provider_account_secret_binding(
        &communication_store,
        account_id,
        ProviderAccountSecretPurpose::ImapPassword,
    )
    .await;
    assert_eq!(binding.secret_ref, secret_ref);
    let binding_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'vault'
           AND entity_kind = 'communication_provider_secret_binding'
           AND entity_id = $1
           AND relationship_kind = 'bind'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(format!(
        "{}:{}",
        account_id,
        ProviderAccountSecretPurpose::ImapPassword.as_str()
    ))
    .fetch_one(&restarted_pool)
    .await
    .expect("provider secret binding observation link");
    let binding_observation = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
    )
    .bind(&binding_observation_id)
    .fetch_one(&restarted_pool)
    .await
    .expect("provider secret binding observation");
    assert_eq!(
        binding_observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "vault_source"
    );
    assert_eq!(
        binding_observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "COMMUNICATION_PROVIDER_SECRET_BINDING"
    );

    let calendar_store = CalendarAccountStore::new(restarted_pool.clone());
    let calendar_account =
        wait_for_calendar_account(&calendar_store, &format!("icloud-calendar:{account_id}")).await;
    assert_eq!(calendar_account.provider, "apple");
    assert_eq!(
        calendar_account.email.as_deref(),
        Some("recover@icloud.com")
    );
    assert_eq!(
        calendar_account.credentials_reference.as_deref(),
        Some(secret_ref)
    );
    let calendar_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'calendar'
           AND entity_kind = 'calendar_account'
           AND entity_id = $1
           AND relationship_kind = 'linked_provider_upsert'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(format!("icloud-calendar:{account_id}"))
    .fetch_one(&restarted_pool)
    .await
    .expect("calendar account observation link");
    let calendar_observation = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
    )
    .bind(&calendar_observation_id)
    .fetch_one(&restarted_pool)
    .await
    .expect("calendar account observation");
    assert_eq!(
        calendar_observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "vault_source"
    );
    assert_eq!(
        calendar_observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "CALENDAR_ACCOUNT_LINK"
    );

    assert_eq!(
        vault
            .resolve(&reference)
            .await
            .expect("resolve restored secret")
            .expose_for_runtime(),
        "icloud-app-password"
    );
}
