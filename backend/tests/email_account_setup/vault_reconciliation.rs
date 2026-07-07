use serde_json::json;
use sqlx::Row;
use tempfile::tempdir;
use tokio::time::{Duration, sleep};
use tower::ServiceExt;

use hermes_hub_backend::ai::control_center::{AiControlCenterStore, AiProviderAccount};
use hermes_hub_backend::app::build_router_with_database;
use hermes_hub_backend::domains::calendar::events::CalendarAccountStore;
use hermes_hub_backend::domains::communications::core::{
    CommunicationIngestionStore, CommunicationProviderKind, EmailProviderKind,
    ProviderAccountSecretPurpose,
};
use hermes_hub_backend::platform::secrets::{SecretKind, SecretReferenceStore, SecretResolver};
use hermes_hub_backend::platform::storage::Database;
use hermes_hub_backend::vault::{EntropyEvent, HostVault, HostVaultConfig, SecretEntryContext};
use testkit::context::TestContext;

use super::support::{
    LOCAL_API_TOKEN, delete_request_with_token, json_body, json_request_with_token_and_actor,
    unlock_test_vault, wait_for_calendar_account, wait_for_manifest_metadata_key,
    wait_for_provider_account, wait_for_provider_account_secret_binding, wait_for_secret_reference,
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
    let config =
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("HERMES_DEV_MODE", "true"),
                (
                    "HERMES_VAULT_HOME",
                    vault_home.to_str().expect("vault path"),
                ),
                (
                    "HERMES_DEV_KEY_PATH",
                    dev_key_path.to_str().expect("dev key path"),
                ),
            ])
            .expect("config");
    let app = build_router_with_database(config.clone(), database.clone());
    unlock_test_vault(app.clone()).await;

    let account_id = "icloud-recover";
    let secret_ref = "secret:provider-account:icloud-recover:imap_password";
    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/integrations/mail/accounts/imap",
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
    wait_for_provider_account_secret_binding(
        &CommunicationIngestionStore::new(pool.clone()),
        account_id,
        ProviderAccountSecretPurpose::ImapPassword,
    )
    .await;

    let mut wipe = pool.begin().await.expect("begin metadata wipe");
    sqlx::query(
        "SELECT account_id FROM communication_provider_accounts WHERE account_id = $1 FOR UPDATE",
    )
    .bind(account_id)
    .fetch_optional(&mut *wipe)
    .await
    .expect("lock provider account metadata");
    sqlx::query("DELETE FROM calendar_accounts WHERE account_id = $1")
        .bind(format!("icloud-calendar:{account_id}"))
        .execute(&mut *wipe)
        .await
        .expect("delete calendar metadata");
    sqlx::query("DELETE FROM communication_provider_account_secret_refs WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut *wipe)
        .await
        .expect("delete secret binding");
    sqlx::query("DELETE FROM communication_provider_accounts WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut *wipe)
        .await
        .expect("delete provider account");
    sqlx::query("DELETE FROM secret_references WHERE secret_ref = $1")
        .bind(secret_ref)
        .execute(&mut *wipe)
        .await
        .expect("delete secret reference");
    wipe.commit().await.expect("commit metadata wipe");

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

#[tokio::test]
async fn delete_mail_account_removes_unbound_host_vault_secret_and_reference() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let config =
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("HERMES_DEV_MODE", "true"),
                (
                    "HERMES_VAULT_HOME",
                    vault_home.to_str().expect("vault path"),
                ),
                (
                    "HERMES_DEV_KEY_PATH",
                    dev_key_path.to_str().expect("dev key path"),
                ),
            ])
            .expect("config");
    let app = build_router_with_database(config, database.clone());
    unlock_test_vault(app.clone()).await;

    let account_id = "icloud-delete-vault";
    let secret_ref = "secret:provider-account:icloud-delete-vault:imap_password";
    let smtp_secret_ref = "secret:provider-account:icloud-delete-vault:smtp_password";
    let setup_response = app
        .clone()
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/integrations/mail/accounts/imap",
            json!({
                "account_id": account_id,
                "provider_kind": "icloud",
                "display_name": "Delete Vault iCloud",
                "external_account_id": "delete-vault@icloud.com",
                "host": "imap.mail.me.com",
                "port": 993,
                "tls": true,
                "mailbox": "INBOX",
                "username": "delete-vault@icloud.com",
                "password": "icloud-app-password",
                "secret_kind": "app_password"
            }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("setup response");
    assert_eq!(setup_response.status(), axum::http::StatusCode::OK);

    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_store = SecretReferenceStore::new(pool.clone());
    let vault = HostVault::new(HostVaultConfig {
        home: vault_home,
        dev_mode: true,
        dev_key_path,
    })
    .expect("host vault");
    vault.unlock_existing().expect("unlock host vault");

    assert!(
        communication_store
            .provider_account(account_id)
            .await
            .expect("provider account before delete")
            .is_some()
    );
    assert!(
        secret_store
            .secret_reference(secret_ref)
            .await
            .expect("secret reference before delete")
            .is_some()
    );

    let delete_response = app
        .oneshot(delete_request_with_token(
            &format!("/api/v1/integrations/mail/accounts/{account_id}"),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("delete response");
    assert_eq!(delete_response.status(), axum::http::StatusCode::OK);
    let body = json_body(delete_response).await;
    assert_eq!(body["deleted"], json!(true));
    assert_eq!(
        body["vault_deleted_secret_refs"],
        json!([secret_ref, smtp_secret_ref])
    );
    assert_eq!(body["retained_secret_refs"], json!([]));

    assert!(
        communication_store
            .provider_account(account_id)
            .await
            .expect("provider account after delete")
            .is_none()
    );
    assert!(
        secret_store
            .secret_reference(secret_ref)
            .await
            .expect("secret reference after delete")
            .is_none()
    );
    assert!(
        secret_store
            .secret_reference(smtp_secret_ref)
            .await
            .expect("smtp secret reference after delete")
            .is_none()
    );
    assert!(
        vault
            .account_secret_manifest()
            .expect("manifest after delete")
            .into_iter()
            .all(|entry| entry.secret_ref != secret_ref && entry.secret_ref != smtp_secret_ref)
    );
    assert!(vault.read_secret(secret_ref).is_err());
    assert!(vault.read_secret(smtp_secret_ref).is_err());
}

#[tokio::test]
async fn startup_reconciles_non_mail_provider_account_from_host_vault_manifest() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let config =
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("HERMES_DEV_MODE", "true"),
                (
                    "HERMES_VAULT_HOME",
                    vault_home.to_str().expect("vault path"),
                ),
                (
                    "HERMES_DEV_KEY_PATH",
                    dev_key_path.to_str().expect("dev key path"),
                ),
            ])
            .expect("config");
    let app = build_router_with_database(config.clone(), database.clone());
    unlock_test_vault(app).await;

    let account_id = "zulip-recover";
    let secret_ref = "secret:provider-account:zulip-recover:zulip_api_key";
    let vault = HostVault::new(HostVaultConfig {
        home: vault_home,
        dev_mode: true,
        dev_key_path,
    })
    .expect("host vault");
    vault.unlock_existing().expect("unlock host vault");
    vault
        .store_secret(
            secret_ref,
            "zulip-api-key",
            SecretEntryContext {
                entry_kind: "provider_api_token",
                account_id,
                purpose: ProviderAccountSecretPurpose::ZulipApiKey.as_str(),
                secret_kind: SecretKind::ApiToken.as_str(),
                label: "Zulip API key",
                metadata: &json!({
                    "provider": CommunicationProviderKind::ZulipBot.as_str(),
                    "account_id": account_id,
                    "display_name": "Recovered Zulip",
                    "external_account_id": "bot@example.zulipchat.com",
                    "provider_account_config": {
                        "base_url": "https://example.zulipchat.com",
                        "runtime": "api"
                    }
                }),
            },
        )
        .expect("store zulip secret");

    let restarted_database = Database::connect(Some(&database_url))
        .await
        .expect("restarted database connection");
    let _restarted_app = build_router_with_database(config, restarted_database.clone());
    let restarted_pool = restarted_database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(restarted_pool.clone());
    let secret_store = SecretReferenceStore::new(restarted_pool);

    let account = wait_for_provider_account(&communication_store, account_id).await;
    assert_eq!(account.provider_kind, CommunicationProviderKind::ZulipBot);
    assert_eq!(account.display_name, "Recovered Zulip");
    assert_eq!(account.external_account_id, "bot@example.zulipchat.com");
    assert_eq!(
        account.config["base_url"],
        json!("https://example.zulipchat.com")
    );

    let reference = wait_for_secret_reference(&secret_store, secret_ref).await;
    assert_eq!(reference.secret_kind, SecretKind::ApiToken);
    assert_eq!(reference.store_kind.as_str(), "host_vault");
    let binding = wait_for_provider_account_secret_binding(
        &communication_store,
        account_id,
        ProviderAccountSecretPurpose::ZulipApiKey,
    )
    .await;
    assert_eq!(binding.secret_ref, secret_ref);
    assert_eq!(
        vault
            .resolve(&reference)
            .await
            .expect("resolve restored zulip secret")
            .expose_for_runtime(),
        "zulip-api-key"
    );
}

#[tokio::test]
async fn startup_reconciles_legacy_gmail_manifest_without_provider_metadata() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let config =
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("HERMES_DEV_MODE", "true"),
                (
                    "HERMES_VAULT_HOME",
                    vault_home.to_str().expect("vault path"),
                ),
                (
                    "HERMES_DEV_KEY_PATH",
                    dev_key_path.to_str().expect("dev key path"),
                ),
            ])
            .expect("config");
    let vault = HostVault::new(HostVaultConfig {
        home: vault_home.clone(),
        dev_mode: true,
        dev_key_path: dev_key_path.clone(),
    })
    .expect("host vault");
    let entropy_events: Vec<EntropyEvent> = super::support::vault_entropy_events(2000)
        .into_iter()
        .map(|value| serde_json::from_value(value).expect("entropy event"))
        .collect();
    vault
        .collect_entropy(entropy_events)
        .expect("collect entropy");
    vault.create().expect("create host vault");

    let account_id = "mail-gmail-karelon-gmail-com";
    let secret_ref = "secret:provider-account:mail-gmail-karelon-gmail-com:oauth_token";
    vault
        .store_secret(
            secret_ref,
            "legacy-gmail-oauth-token",
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id: secret_ref,
                purpose: ProviderAccountSecretPurpose::OauthToken.as_str(),
                secret_kind: SecretKind::OauthToken.as_str(),
                label: "OAuth credential",
                metadata: &json!({}),
            },
        )
        .expect("store legacy gmail secret");

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let _app = build_router_with_database(config, database.clone());
    let pool = database.pool().expect("configured pool").clone();
    let communication_store = CommunicationIngestionStore::new(pool.clone());
    let secret_store = SecretReferenceStore::new(pool);

    let account = wait_for_provider_account(&communication_store, account_id).await;
    assert_eq!(account.provider_kind, EmailProviderKind::Gmail);
    assert_eq!(account.display_name, "Google Workspace");
    assert_eq!(account.external_account_id, "karelon@gmail.com");
    assert_eq!(account.config["auth"], json!("oauth"));
    assert_eq!(account.config["api"], json!("gmail"));

    let reference = wait_for_secret_reference(&secret_store, secret_ref).await;
    assert_eq!(reference.secret_kind, SecretKind::OauthToken);
    assert_eq!(reference.store_kind.as_str(), "host_vault");
    let binding = wait_for_provider_account_secret_binding(
        &communication_store,
        account_id,
        ProviderAccountSecretPurpose::OauthToken,
    )
    .await;
    assert_eq!(binding.secret_ref, secret_ref);
    assert_eq!(
        vault
            .resolve(&reference)
            .await
            .expect("resolve restored legacy gmail secret")
            .expose_for_runtime(),
        "legacy-gmail-oauth-token"
    );
}

#[tokio::test]
async fn startup_reconciles_one_account_for_duplicate_provider_external_identity() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let config =
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("HERMES_DEV_MODE", "true"),
                (
                    "HERMES_VAULT_HOME",
                    vault_home.to_str().expect("vault path"),
                ),
                (
                    "HERMES_DEV_KEY_PATH",
                    dev_key_path.to_str().expect("dev key path"),
                ),
            ])
            .expect("config");
    let vault = HostVault::new(HostVaultConfig {
        home: vault_home,
        dev_mode: true,
        dev_key_path,
    })
    .expect("host vault");
    let entropy_events: Vec<EntropyEvent> = super::support::vault_entropy_events(2000)
        .into_iter()
        .map(|value| serde_json::from_value(value).expect("entropy event"))
        .collect();
    vault
        .collect_entropy(entropy_events)
        .expect("collect entropy");
    vault.create().expect("create host vault");

    vault
        .store_secret(
            "secret:provider-account:gmail-duplicate-old:oauth_token",
            "old-gmail-oauth-token",
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id: "gmail-duplicate-old",
                purpose: ProviderAccountSecretPurpose::OauthToken.as_str(),
                secret_kind: SecretKind::OauthToken.as_str(),
                label: "Old Gmail OAuth credential",
                metadata: &json!({
                    "provider": "gmail",
                    "account_id": "gmail-duplicate-old",
                    "display_name": "Old Gmail",
                    "external_account_id": "duplicate@gmail.com",
                    "provider_account_config": {
                        "auth": "oauth",
                        "api": "gmail"
                    }
                }),
            },
        )
        .expect("store old duplicate gmail secret");
    vault
        .store_secret(
            "secret:provider-account:gmail-duplicate-new:oauth_token",
            "new-gmail-oauth-token",
            SecretEntryContext {
                entry_kind: "provider_credential",
                account_id: "gmail-duplicate-new",
                purpose: ProviderAccountSecretPurpose::OauthToken.as_str(),
                secret_kind: SecretKind::OauthToken.as_str(),
                label: "New Gmail OAuth credential",
                metadata: &json!({
                    "provider": "gmail",
                    "account_id": "gmail-duplicate-new",
                    "display_name": "New Gmail",
                    "external_account_id": "duplicate@gmail.com",
                    "provider_account_config": {
                        "auth": "oauth",
                        "api": "gmail"
                    }
                }),
            },
        )
        .expect("store new duplicate gmail secret");

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let _app = build_router_with_database(config, database.clone());
    let pool = database.pool().expect("configured pool").clone();

    for _ in 0..50 {
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*)
             FROM communication_provider_accounts
             WHERE provider_kind = 'gmail'
               AND external_account_id = 'duplicate@gmail.com'",
        )
        .fetch_one(&pool)
        .await
        .expect("duplicate account count");
        if count == 1 {
            sleep(Duration::from_millis(100)).await;
            let stable_count: i64 = sqlx::query_scalar(
                "SELECT count(*)
                 FROM communication_provider_accounts
                 WHERE provider_kind = 'gmail'
                   AND external_account_id = 'duplicate@gmail.com'",
            )
            .fetch_one(&pool)
            .await
            .expect("stable duplicate account count");
            assert_eq!(stable_count, 1);
            let binding_count: i64 = sqlx::query_scalar(
                "SELECT count(*)
                 FROM communication_provider_account_secret_refs refs
                 JOIN communication_provider_accounts accounts
                   ON accounts.account_id = refs.account_id
                 WHERE accounts.provider_kind = 'gmail'
                   AND accounts.external_account_id = 'duplicate@gmail.com'",
            )
            .fetch_one(&pool)
            .await
            .expect("duplicate binding count");
            assert_eq!(binding_count, 1);

            let duplicate_manifest_entries: Vec<_> = vault
                .account_secret_manifest()
                .expect("host vault manifest")
                .into_iter()
                .filter(|entry| {
                    entry
                        .metadata
                        .get("provider")
                        .and_then(|value| value.as_str())
                        == Some("gmail")
                        && entry
                            .metadata
                            .get("external_account_id")
                            .and_then(|value| value.as_str())
                            == Some("duplicate@gmail.com")
                })
                .collect();
            assert_eq!(duplicate_manifest_entries.len(), 1);
            return;
        }
        sleep(Duration::from_millis(50)).await;
    }

    panic!("duplicate Gmail vault entries were not reconciled to one account");
}

#[tokio::test]
async fn startup_reconciles_ai_api_provider_from_host_vault_after_postgres_metadata_wipe() {
    let ctx = TestContext::new().await;
    let vault_dir = tempdir().expect("vault tempdir");
    let database_url = ctx.connection_string();
    let vault_home = vault_dir.path().join("vault");
    let dev_key_path = vault_dir.path().join("dev").join("master.key");
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let config =
        testkit::app::config_with_secret_and_database_url(LOCAL_API_TOKEN, database_url.as_str())
            .with_test_pairs([
                ("HERMES_DEV_MODE", "true"),
                (
                    "HERMES_VAULT_HOME",
                    vault_home.to_str().expect("vault path"),
                ),
                (
                    "HERMES_DEV_KEY_PATH",
                    dev_key_path.to_str().expect("dev key path"),
                ),
            ])
            .expect("config");
    let app = build_router_with_database(config.clone(), database.clone());
    unlock_test_vault(app.clone()).await;

    let response = app
        .oneshot(json_request_with_token_and_actor(
            "/api/v1/ai/providers",
            json!({
                "provider_kind": "api",
                "provider_key": "omniroute",
                "display_name": "Recovered OmniRoute",
                "base_url": "https://ai.sh-inc.ru/v1",
                "capabilities": ["chat", "reasoning", "embeddings"],
                "enabled": true,
                "remote_context_consent": true,
                "api_key": "omniroute-api-key"
            }),
            LOCAL_API_TOKEN,
            "hermes-frontend",
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = json_body(response).await;
    let provider_id = body["provider_id"]
        .as_str()
        .expect("provider id")
        .to_owned();
    let secret_ref = format!("secret:ai-provider:{provider_id}:api_key");

    let pool = database.pool().expect("configured pool").clone();
    let vault = HostVault::new(HostVaultConfig {
        home: vault_home.clone(),
        dev_mode: true,
        dev_key_path: dev_key_path.clone(),
    })
    .expect("host vault");
    vault.unlock_existing().expect("unlock host vault");
    wait_for_manifest_metadata_key(&vault, &secret_ref, "provider_key").await;

    sqlx::query("DELETE FROM ai_provider_accounts WHERE provider_id = $1")
        .bind(&provider_id)
        .execute(&pool)
        .await
        .expect("delete ai provider metadata");
    sqlx::query("DELETE FROM secret_references WHERE secret_ref = $1")
        .bind(&secret_ref)
        .execute(&pool)
        .await
        .expect("delete ai secret reference");

    let restarted_database = Database::connect(Some(&database_url))
        .await
        .expect("restarted database connection");
    let _restarted_app = build_router_with_database(config, restarted_database.clone());
    let restarted_pool = restarted_database.pool().expect("configured pool").clone();
    let ai_store = AiControlCenterStore::new(restarted_pool.clone());
    let secret_store = SecretReferenceStore::new(restarted_pool.clone());

    let provider = wait_for_ai_provider(&ai_store, &provider_id).await;
    assert_eq!(provider.provider_kind, "api");
    assert_eq!(provider.provider_key, "omniroute");
    assert_eq!(provider.display_name, "Recovered OmniRoute");
    assert_eq!(provider.status, "ready");
    assert_eq!(provider.consent_state, "granted");
    assert_eq!(
        provider.config["base_url"],
        json!("https://ai.sh-inc.ru/v1")
    );
    assert!(provider.capabilities.contains(&"chat".to_owned()));

    assert_eq!(
        ai_store
            .api_key_secret_ref(&provider_id)
            .await
            .expect("ai api key ref"),
        Some(secret_ref.clone())
    );
    let reference = wait_for_secret_reference(&secret_store, &secret_ref).await;
    assert_eq!(reference.secret_kind, SecretKind::ApiToken);
    assert_eq!(reference.store_kind.as_str(), "host_vault");
    assert_eq!(
        vault
            .resolve(&reference)
            .await
            .expect("resolve restored ai secret")
            .expose_for_runtime(),
        "omniroute-api-key"
    );
    let model_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM ai_model_catalog WHERE provider_id = $1")
            .bind(&provider_id)
            .fetch_one(&restarted_pool)
            .await
            .expect("model count");
    assert!(model_count > 0);
}

async fn wait_for_ai_provider(
    store: &AiControlCenterStore,
    provider_id: &str,
) -> AiProviderAccount {
    for _ in 0..50 {
        if let Some(provider) = store.provider(provider_id).await.expect("load ai provider") {
            return provider;
        }
        sleep(Duration::from_millis(50)).await;
    }

    panic!("AI provider {provider_id} was not reconciled");
}
