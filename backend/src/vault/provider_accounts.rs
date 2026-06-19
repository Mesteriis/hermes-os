use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Transaction};

use crate::domains::calendar::events::{
    CalendarAccount, CalendarAccountUpdate, CalendarError, CalendarSource,
};
use crate::domains::mail::core::{
    CommunicationIngestionError, DeletedProviderAccount, NewProviderAccount,
    NewProviderAccountSecretBinding, ProviderAccount, ProviderAccountSecretBinding,
    ProviderAccountSecretPurpose, ProviderAccountUsage,
};
use crate::domains::tasks::core::{TaskCoreError, TaskProviderAccount};
use crate::platform::observations::{
    NewObservation, ObservationOriginKind, ObservationStore, link_domain_entity_in_transaction,
};

#[derive(Clone)]
pub struct CalendarAccountStore {
    pool: PgPool,
}

impl CalendarAccountStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        provider: &str,
        account_name: &str,
        email: Option<&str>,
    ) -> Result<CalendarAccount, CalendarError> {
        self.create_with_observation(provider, account_name, email, None, "create", None)
            .await
    }

    pub async fn create_with_observation(
        &self,
        provider: &str,
        account_name: &str,
        email: Option<&str>,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<CalendarAccount, CalendarError> {
        let account_id = next_id("cal");
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "INSERT INTO calendar_accounts (account_id, provider, account_name, email) VALUES ($1,$2,$3,$4) RETURNING account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at",
        )
        .bind(&account_id)
        .bind(provider)
        .bind(account_name)
        .bind(email)
        .fetch_one(&mut *transaction)
        .await?;
        let account = row_to_calendar_account(row).map_err(CalendarError::from)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                observation_id,
                "calendar",
                "calendar_account",
                account.account_id.clone(),
                relationship_kind,
                json!({
                    "account_id": account.account_id,
                    "provider": account.provider,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(account)
    }

    pub async fn get(&self, account_id: &str) -> Result<Option<CalendarAccount>, CalendarError> {
        let row = sqlx::query("SELECT account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at FROM calendar_accounts WHERE account_id=$1")
            .bind(account_id).fetch_optional(&self.pool).await?;
        row.map(row_to_calendar_account)
            .transpose()
            .map_err(CalendarError::from)
    }

    pub async fn list(
        &self,
        provider: Option<&str>,
    ) -> Result<Vec<CalendarAccount>, CalendarError> {
        let rows = if let Some(provider) = provider {
            sqlx::query("SELECT account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at FROM calendar_accounts WHERE provider=$1 ORDER BY account_name")
                .bind(provider).fetch_all(&self.pool).await?
        } else {
            sqlx::query("SELECT account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at FROM calendar_accounts ORDER BY account_name")
                .fetch_all(&self.pool).await?
        };
        rows.into_iter()
            .map(row_to_calendar_account)
            .collect::<Result<Vec<_>, _>>()
            .map_err(CalendarError::from)
    }

    pub async fn update(
        &self,
        account_id: &str,
        update: &CalendarAccountUpdate,
    ) -> Result<CalendarAccount, CalendarError> {
        self.update_with_observation(account_id, update, None, "update", None)
            .await
    }

    pub async fn update_with_observation(
        &self,
        account_id: &str,
        update: &CalendarAccountUpdate,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<CalendarAccount, CalendarError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "UPDATE calendar_accounts SET account_name=COALESCE($2,account_name), email=COALESCE($3,email), sync_status=COALESCE($4,sync_status), updated_at=now() WHERE account_id=$1 RETURNING account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at",
        )
        .bind(account_id)
        .bind(update.account_name.as_deref())
        .bind(update.email.as_deref())
        .bind(update.sync_status.as_deref())
        .fetch_one(&mut *transaction)
        .await?;
        let account = row_to_calendar_account(row).map_err(CalendarError::from)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                observation_id,
                "calendar",
                "calendar_account",
                account.account_id.clone(),
                relationship_kind,
                json!({
                    "account_id": account.account_id,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(account)
    }

    pub async fn upsert_google_workspace_account(
        &self,
        mail_account_id: &str,
        account_name: &str,
        email: Option<&str>,
        credentials_reference: &str,
    ) -> Result<CalendarAccount, CalendarError> {
        self.upsert_linked_provider_account(
            &format!("google-calendar:{mail_account_id}"),
            "google",
            mail_account_id,
            account_name,
            email,
            credentials_reference,
            "gmail",
            "google_calendar_api",
            ObservationOriginKind::LocalRuntime,
            "mail_account_setup.upsert_google_workspace_calendar_account",
        )
        .await
    }

    pub async fn upsert_apple_icloud_account(
        &self,
        mail_account_id: &str,
        account_name: &str,
        email: Option<&str>,
        credentials_reference: &str,
    ) -> Result<CalendarAccount, CalendarError> {
        self.upsert_linked_provider_account(
            &format!("icloud-calendar:{mail_account_id}"),
            "apple",
            mail_account_id,
            account_name,
            email,
            credentials_reference,
            "icloud",
            "apple_caldav",
            ObservationOriginKind::LocalRuntime,
            "mail_account_setup.upsert_apple_icloud_calendar_account",
        )
        .await
    }

    pub async fn restore_google_workspace_account(
        &self,
        mail_account_id: &str,
        account_name: &str,
        email: Option<&str>,
        credentials_reference: &str,
    ) -> Result<CalendarAccount, CalendarError> {
        self.upsert_linked_provider_account(
            &format!("google-calendar:{mail_account_id}"),
            "google",
            mail_account_id,
            account_name,
            email,
            credentials_reference,
            "gmail",
            "google_calendar_api",
            ObservationOriginKind::VaultSource,
            "vault_reconciliation.restore_linked_calendar_account",
        )
        .await
    }

    pub async fn restore_apple_icloud_account(
        &self,
        mail_account_id: &str,
        account_name: &str,
        email: Option<&str>,
        credentials_reference: &str,
    ) -> Result<CalendarAccount, CalendarError> {
        self.upsert_linked_provider_account(
            &format!("icloud-calendar:{mail_account_id}"),
            "apple",
            mail_account_id,
            account_name,
            email,
            credentials_reference,
            "icloud",
            "apple_caldav",
            ObservationOriginKind::VaultSource,
            "vault_reconciliation.restore_linked_calendar_account",
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn upsert_linked_provider_account(
        &self,
        account_id: &str,
        provider: &str,
        mail_account_id: &str,
        account_name: &str,
        email: Option<&str>,
        credentials_reference: &str,
        source_provider: &str,
        sync_mode: &str,
        origin_kind: ObservationOriginKind,
        actor: &str,
    ) -> Result<CalendarAccount, CalendarError> {
        let mut transaction = self.pool.begin().await?;
        let capabilities = json!({
            "mail_account_id": mail_account_id,
            "source_provider": source_provider,
            "connected_services": ["calendar"],
            "sync_mode": sync_mode
        });
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "CALENDAR_ACCOUNT_LINK",
                origin_kind,
                chrono::Utc::now(),
                json!({
                    "account_id": account_id,
                    "provider": provider,
                    "mail_account_id": mail_account_id,
                    "account_name": account_name,
                    "email": email,
                    "credentials_reference": credentials_reference,
                    "source_provider": source_provider,
                    "sync_mode": sync_mode,
                    "action": "upsert_linked_calendar_account",
                }),
                format!("calendar-account://{account_id}/linked-provider"),
            )
            .provenance(json!({
                "captured_by": actor,
                "action": "upsert_linked_calendar_account",
                "source_provider": source_provider,
                "mail_account_id": mail_account_id,
            })),
        )
        .await?;
        let row = sqlx::query(
            r#"
            INSERT INTO calendar_accounts (
                account_id,
                provider,
                account_name,
                email,
                credentials_reference,
                capabilities,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, now())
            ON CONFLICT (account_id)
            DO UPDATE SET
                provider = EXCLUDED.provider,
                account_name = EXCLUDED.account_name,
                email = EXCLUDED.email,
                credentials_reference = EXCLUDED.credentials_reference,
                capabilities = EXCLUDED.capabilities,
                updated_at = now()
            RETURNING account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at
            "#,
        )
        .bind(account_id)
        .bind(provider)
        .bind(account_name)
        .bind(email)
        .bind(credentials_reference)
        .bind(&capabilities)
        .fetch_one(&mut *transaction)
        .await?;
        link_vault_owned_entity_in_transaction(
            &mut transaction,
            &observation.observation_id,
            "calendar",
            "calendar_account",
            account_id.to_owned(),
            "linked_provider_upsert",
            json!({
                "account_id": account_id,
                "provider": provider,
                "mail_account_id": mail_account_id,
                "source_provider": source_provider,
                "sync_mode": sync_mode,
            }),
            None,
        )
        .await?;
        transaction.commit().await?;
        row_to_calendar_account(row).map_err(CalendarError::from)
    }

    pub async fn delete(&self, account_id: &str) -> Result<(), CalendarError> {
        self.delete_with_observation(account_id, None, "delete", None)
            .await
    }

    pub async fn delete_with_observation(
        &self,
        account_id: &str,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), CalendarError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query("DELETE FROM calendar_accounts WHERE account_id=$1")
            .bind(account_id)
            .execute(&mut *transaction)
            .await?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                observation_id,
                "calendar",
                "calendar_account",
                account_id.to_owned(),
                relationship_kind,
                json!({
                    "account_id": account_id,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct CalendarSourceStore {
    pool: PgPool,
}

impl CalendarSourceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        account_id: &str,
        name: &str,
        provider_calendar_id: Option<&str>,
        color: Option<&str>,
        timezone: Option<&str>,
    ) -> Result<CalendarSource, CalendarError> {
        self.create_with_observation(
            account_id,
            name,
            provider_calendar_id,
            color,
            timezone,
            None,
            "create",
            None,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_with_observation(
        &self,
        account_id: &str,
        name: &str,
        provider_calendar_id: Option<&str>,
        color: Option<&str>,
        timezone: Option<&str>,
        observation_id: Option<&str>,
        relationship_kind: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<CalendarSource, CalendarError> {
        let source_id = next_id("src");
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            "INSERT INTO calendar_sources (source_id, account_id, provider_calendar_id, name, color, timezone) VALUES ($1,$2,$3,$4,$5,$6) RETURNING source_id, account_id, provider_calendar_id, name, color, timezone, visibility, read_only, sync_enabled, capabilities, created_at, updated_at",
        )
        .bind(&source_id)
        .bind(account_id)
        .bind(provider_calendar_id)
        .bind(name)
        .bind(color)
        .bind(timezone)
        .fetch_one(&mut *transaction)
        .await?;
        let source = row_to_calendar_source(row).map_err(CalendarError::from)?;
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                observation_id,
                "calendar",
                "calendar_source",
                source.source_id.clone(),
                relationship_kind,
                json!({
                    "source_id": source.source_id,
                    "account_id": source.account_id,
                }),
                metadata,
            )
            .await?;
        }
        transaction.commit().await?;
        Ok(source)
    }

    pub async fn list_by_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<CalendarSource>, CalendarError> {
        let rows = sqlx::query("SELECT source_id, account_id, provider_calendar_id, name, color, timezone, visibility, read_only, sync_enabled, capabilities, created_at, updated_at FROM calendar_sources WHERE account_id=$1 ORDER BY name")
            .bind(account_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(row_to_calendar_source)
            .collect::<Result<Vec<_>, _>>()
            .map_err(CalendarError::from)
    }

    pub async fn get(&self, source_id: &str) -> Result<Option<CalendarSource>, CalendarError> {
        let row = sqlx::query("SELECT source_id, account_id, provider_calendar_id, name, color, timezone, visibility, read_only, sync_enabled, capabilities, created_at, updated_at FROM calendar_sources WHERE source_id=$1")
            .bind(source_id).fetch_optional(&self.pool).await?;
        row.map(row_to_calendar_source)
            .transpose()
            .map_err(CalendarError::from)
    }
}

#[derive(Clone)]
pub struct TaskProviderStore {
    pool: PgPool,
}

impl TaskProviderStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<TaskProviderAccount>, TaskCoreError> {
        let rows = sqlx::query(
            r#"
            SELECT account_id, provider, account_name, credentials_reference,
                   sync_mode, capabilities, created_at, updated_at
            FROM task_provider_accounts
            ORDER BY provider, account_name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_task_provider_account).collect()
    }

    pub async fn create(
        &self,
        provider: &str,
        account_name: &str,
    ) -> Result<TaskProviderAccount, TaskCoreError> {
        self.create_with_origin(
            provider,
            account_name,
            ObservationOriginKind::LocalRuntime,
            "tasks_api.post_task_provider",
        )
        .await
    }

    pub async fn create_with_origin(
        &self,
        provider: &str,
        account_name: &str,
        origin_kind: ObservationOriginKind,
        actor: &str,
    ) -> Result<TaskProviderAccount, TaskCoreError> {
        let account_id = next_id("tprov");
        let mut transaction = self.pool.begin().await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "TASK_PROVIDER_ACCOUNT",
                origin_kind,
                chrono::Utc::now(),
                json!({
                    "account_id": account_id,
                    "provider": provider,
                    "account_name": account_name,
                    "action": "create_task_provider_account",
                }),
                format!("task-provider://{account_id}"),
            )
            .provenance(json!({
                "captured_by": actor,
                "action": "create_task_provider_account",
            })),
        )
        .await?;
        let row = sqlx::query(
            r#"
            INSERT INTO task_provider_accounts (account_id, provider, account_name)
            VALUES ($1, $2, $3)
            RETURNING account_id, provider, account_name, credentials_reference,
                      sync_mode, capabilities, created_at, updated_at
            "#,
        )
        .bind(&account_id)
        .bind(provider)
        .bind(account_name)
        .fetch_one(&mut *transaction)
        .await?;
        link_vault_owned_entity_in_transaction(
            &mut transaction,
            &observation.observation_id,
            "vault",
            "task_provider_account",
            account_id.clone(),
            "create",
            json!({
                "provider": provider,
                "account_name": account_name,
            }),
            None,
        )
        .await?;
        transaction.commit().await?;

        row_to_task_provider_account(row)
    }
}

#[derive(Clone)]
pub struct CommunicationProviderAccountStore {
    pool: PgPool,
}

impl CommunicationProviderAccountStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert_runtime_account(
        &self,
        account_id: impl Into<String>,
        provider_kind: &str,
        display_name: impl Into<String>,
        external_account_id: impl Into<String>,
        config: serde_json::Value,
    ) -> Result<ProviderAccount, CommunicationIngestionError> {
        let provider_kind =
            crate::domains::mail::core::CommunicationProviderKind::try_from(provider_kind)?;
        self.upsert(
            &NewProviderAccount::new(account_id, provider_kind, display_name, external_account_id)
                .config(config),
        )
        .await
    }

    pub async fn upsert(
        &self,
        account: &NewProviderAccount,
    ) -> Result<ProviderAccount, CommunicationIngestionError> {
        self.upsert_with_origin(
            account,
            ObservationOriginKind::LocalRuntime,
            "vault.communication_provider_accounts.upsert",
        )
        .await
    }

    pub async fn restore(
        &self,
        account: &NewProviderAccount,
    ) -> Result<ProviderAccount, CommunicationIngestionError> {
        self.upsert_with_origin(
            account,
            ObservationOriginKind::VaultSource,
            "vault_reconciliation.restore_provider_account",
        )
        .await
    }

    pub async fn upsert_with_origin(
        &self,
        account: &NewProviderAccount,
        origin_kind: ObservationOriginKind,
        actor: &str,
    ) -> Result<ProviderAccount, CommunicationIngestionError> {
        validate_provider_account(account)?;
        let mut transaction = self.pool.begin().await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "COMMUNICATION_PROVIDER_ACCOUNT",
                origin_kind,
                chrono::Utc::now(),
                json!({
                    "account_id": account.account_id.trim(),
                    "provider_kind": account.provider_kind.as_str(),
                    "display_name": account.display_name.trim(),
                    "external_account_id": account.external_account_id.trim(),
                    "config": account.config,
                    "action": "upsert_communication_provider_account",
                }),
                format!(
                    "communication-provider-account://{}",
                    account.account_id.trim()
                ),
            )
            .provenance(json!({
                "captured_by": actor,
                "action": "upsert_communication_provider_account",
                "provider_kind": account.provider_kind.as_str(),
            })),
        )
        .await?;
        let row = sqlx::query(
            r#"
            INSERT INTO communication_provider_accounts (
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (account_id)
            DO UPDATE SET
                provider_kind = EXCLUDED.provider_kind,
                display_name = EXCLUDED.display_name,
                external_account_id = EXCLUDED.external_account_id,
                config = EXCLUDED.config,
                updated_at = now()
            RETURNING
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            "#,
        )
        .bind(account.account_id.trim())
        .bind(account.provider_kind.as_str())
        .bind(account.display_name.trim())
        .bind(account.external_account_id.trim())
        .bind(&account.config)
        .fetch_one(&mut *transaction)
        .await?;
        link_vault_owned_entity_in_transaction(
            &mut transaction,
            &observation.observation_id,
            "vault",
            "communication_provider_account",
            account.account_id.trim().to_owned(),
            "upsert",
            json!({
                "provider_kind": account.provider_kind.as_str(),
                "external_account_id": account.external_account_id.trim(),
            }),
            None,
        )
        .await?;
        transaction.commit().await?;

        row_to_provider_account(row)
    }

    pub async fn get(
        &self,
        account_id: &str,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            FROM communication_provider_accounts
            WHERE account_id = $1
            "#,
        )
        .bind(account_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_provider_account).transpose()
    }

    pub async fn list(&self) -> Result<Vec<ProviderAccount>, CommunicationIngestionError> {
        let rows = sqlx::query(
            r#"
            SELECT
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            FROM communication_provider_accounts
            ORDER BY provider_kind ASC, display_name ASC, account_id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_provider_account).collect()
    }

    pub async fn update_config(
        &self,
        account_id: &str,
        config: &serde_json::Value,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        self.update_config_with_origin(
            account_id,
            config,
            ObservationOriginKind::LocalRuntime,
            "vault.communication_provider_accounts.update_config",
            "update_config",
        )
        .await
    }

    pub async fn mark_logged_out(
        &self,
        account_id: &str,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        let Some(current) = self.get(account_id).await? else {
            return Ok(None);
        };
        let mut config = current.config;
        let config_object = config
            .as_object_mut()
            .ok_or(CommunicationIngestionError::NonObjectJson("config"))?;
        config_object.insert("auth_state".to_owned(), json!("logged_out"));
        config_object.insert("logged_out_at".to_owned(), json!(chrono::Utc::now()));

        self.update_config_with_origin(
            account_id,
            &config,
            ObservationOriginKind::LocalRuntime,
            "vault.communication_provider_accounts.mark_logged_out",
            "logout",
        )
        .await
    }

    pub async fn update_config_with_origin(
        &self,
        account_id: &str,
        config: &serde_json::Value,
        origin_kind: ObservationOriginKind,
        actor: &str,
        action: &str,
    ) -> Result<Option<ProviderAccount>, CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;
        if !config.is_object() {
            return Err(CommunicationIngestionError::NonObjectJson("config"));
        }

        let mut transaction = self.pool.begin().await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "COMMUNICATION_PROVIDER_ACCOUNT_CONFIG_MUTATION",
                origin_kind,
                chrono::Utc::now(),
                json!({
                    "account_id": account_id.trim(),
                    "config": config,
                    "action": action,
                }),
                format!(
                    "communication-provider-account://{}/config",
                    account_id.trim()
                ),
            )
            .provenance(json!({
                "captured_by": actor,
                "action": action,
            })),
        )
        .await?;

        let row = sqlx::query(
            r#"
            UPDATE communication_provider_accounts
            SET config = $2,
                updated_at = now()
            WHERE account_id = $1
            RETURNING
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            "#,
        )
        .bind(account_id.trim())
        .bind(config)
        .fetch_optional(&mut *transaction)
        .await?;

        if row.is_some() {
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                &observation.observation_id,
                "vault",
                "communication_provider_account",
                account_id.trim().to_owned(),
                "config_update",
                json!({
                    "account_id": account_id.trim(),
                    "action": action,
                }),
                None,
            )
            .await?;
            transaction.commit().await?;
        } else {
            transaction.rollback().await?;
        }

        row.map(row_to_provider_account).transpose()
    }

    pub async fn usage(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccountUsage, CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                (SELECT count(*) FROM communication_raw_records WHERE account_id = $1) AS raw_record_count,
                (SELECT count(*) FROM communication_messages WHERE account_id = $1) AS message_count,
                (SELECT count(*) FROM communication_ingestion_checkpoints WHERE account_id = $1) AS checkpoint_count
            "#,
        )
        .bind(account_id.trim())
        .fetch_one(&self.pool)
        .await?;

        Ok(ProviderAccountUsage {
            raw_record_count: row.try_get("raw_record_count")?,
            message_count: row.try_get("message_count")?,
            checkpoint_count: row.try_get("checkpoint_count")?,
        })
    }

    pub async fn delete_metadata(
        &self,
        account_id: &str,
    ) -> Result<DeletedProviderAccount, CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;

        let mut transaction = self.pool.begin().await?;

        let binding_rows = sqlx::query(
            r#"
            DELETE FROM communication_provider_account_secret_refs
            WHERE account_id = $1
            RETURNING account_id, secret_purpose, secret_ref
            "#,
        )
        .bind(account_id.trim())
        .fetch_all(&mut *transaction)
        .await?;
        let mut removed_bindings = Vec::with_capacity(binding_rows.len());
        let unbound_secret_refs = binding_rows
            .into_iter()
            .map(|row| {
                let removed_account_id: String = row.try_get("account_id")?;
                let secret_purpose: String = row.try_get("secret_purpose")?;
                let secret_ref: String = row.try_get("secret_ref")?;
                removed_bindings.push((removed_account_id, secret_purpose, secret_ref.clone()));
                Ok(secret_ref)
            })
            .collect::<Result<Vec<String>, sqlx::Error>>()?;

        for (removed_account_id, secret_purpose, secret_ref) in &removed_bindings {
            let observation = ObservationStore::capture_in_transaction(
                &mut transaction,
                &NewObservation::new(
                    "COMMUNICATION_PROVIDER_SECRET_BINDING_REMOVED",
                    ObservationOriginKind::LocalRuntime,
                    chrono::Utc::now(),
                    json!({
                        "account_id": removed_account_id,
                        "secret_purpose": secret_purpose,
                        "secret_ref": secret_ref,
                        "action": "remove_provider_account_secret_binding",
                    }),
                    format!(
                        "communication-provider-account://{removed_account_id}/secret-binding/{secret_purpose}/delete"
                    ),
                )
                .provenance(json!({
                    "captured_by": "vault.communication_provider_accounts.delete_metadata",
                    "action": "remove_provider_account_secret_binding",
                })),
            )
            .await?;
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                &observation.observation_id,
                "vault",
                "communication_provider_secret_binding",
                format!("{removed_account_id}:{secret_purpose}"),
                "remove",
                json!({
                    "account_id": removed_account_id,
                    "secret_purpose": secret_purpose,
                    "secret_ref": secret_ref,
                }),
                None,
            )
            .await?;
        }

        sqlx::query(
            r#"
            DELETE FROM communication_ingestion_checkpoints
            WHERE account_id = $1
            "#,
        )
        .bind(account_id.trim())
        .execute(&mut *transaction)
        .await?;

        let account_row = sqlx::query(
            r#"
            DELETE FROM communication_provider_accounts
            WHERE account_id = $1
            RETURNING
                account_id,
                provider_kind,
                display_name,
                external_account_id,
                config,
                created_at,
                updated_at
            "#,
        )
        .bind(account_id.trim())
        .fetch_optional(&mut *transaction)
        .await?;

        if let Some(account) = account_row.as_ref() {
            let deleted_account = row_ref_to_provider_account(account)?;
            let observation = ObservationStore::capture_in_transaction(
                &mut transaction,
                &NewObservation::new(
                    "COMMUNICATION_PROVIDER_ACCOUNT_DELETED",
                    ObservationOriginKind::LocalRuntime,
                    chrono::Utc::now(),
                    json!({
                        "account_id": deleted_account.account_id,
                        "provider_kind": deleted_account.provider_kind.as_str(),
                        "display_name": deleted_account.display_name,
                        "external_account_id": deleted_account.external_account_id,
                        "config": deleted_account.config,
                        "action": "delete_communication_provider_account",
                    }),
                    format!(
                        "communication-provider-account://{}/delete",
                        deleted_account.account_id
                    ),
                )
                .provenance(json!({
                    "captured_by": "vault.communication_provider_accounts.delete_metadata",
                    "action": "delete_communication_provider_account",
                    "provider_kind": deleted_account.provider_kind.as_str(),
                })),
            )
            .await?;
            link_vault_owned_entity_in_transaction(
                &mut transaction,
                &observation.observation_id,
                "vault",
                "communication_provider_account",
                deleted_account.account_id.clone(),
                "delete",
                json!({
                    "provider_kind": deleted_account.provider_kind.as_str(),
                    "external_account_id": deleted_account.external_account_id,
                }),
                None,
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(DeletedProviderAccount {
            account: account_row.map(row_to_provider_account).transpose()?,
            unbound_secret_refs,
        })
    }
}

#[derive(Clone)]
pub struct CommunicationProviderSecretBindingStore {
    pool: PgPool,
}

impl CommunicationProviderSecretBindingStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn bind(
        &self,
        binding: &NewProviderAccountSecretBinding,
    ) -> Result<ProviderAccountSecretBinding, CommunicationIngestionError> {
        self.bind_with_origin(
            binding,
            ObservationOriginKind::LocalRuntime,
            "vault.communication_provider_secret_bindings.bind",
        )
        .await
    }

    pub async fn restore(
        &self,
        binding: &NewProviderAccountSecretBinding,
    ) -> Result<ProviderAccountSecretBinding, CommunicationIngestionError> {
        self.bind_with_origin(
            binding,
            ObservationOriginKind::VaultSource,
            "vault_reconciliation.restore_provider_account_secret_binding",
        )
        .await
    }

    pub async fn bind_with_origin(
        &self,
        binding: &NewProviderAccountSecretBinding,
        origin_kind: ObservationOriginKind,
        actor: &str,
    ) -> Result<ProviderAccountSecretBinding, CommunicationIngestionError> {
        validate_provider_secret_binding(binding)?;
        let binding_entity_id = format!(
            "{}:{}",
            binding.account_id.trim(),
            binding.secret_purpose.as_str()
        );
        let mut transaction = self.pool.begin().await?;
        let observation = ObservationStore::capture_in_transaction(
            &mut transaction,
            &NewObservation::new(
                "COMMUNICATION_PROVIDER_SECRET_BINDING",
                origin_kind,
                chrono::Utc::now(),
                json!({
                    "account_id": binding.account_id.trim(),
                    "secret_purpose": binding.secret_purpose.as_str(),
                    "secret_ref": binding.secret_ref.trim(),
                    "action": "bind_provider_account_secret",
                }),
                format!(
                    "communication-provider-account://{}/secret-binding/{}",
                    binding.account_id.trim(),
                    binding.secret_purpose.as_str()
                ),
            )
            .provenance(json!({
                "captured_by": actor,
                "action": "bind_provider_account_secret",
            })),
        )
        .await?;
        let row = sqlx::query(
            r#"
            INSERT INTO communication_provider_account_secret_refs (
                account_id,
                secret_purpose,
                secret_ref,
                updated_at
            )
            VALUES ($1, $2, $3, now())
            ON CONFLICT (account_id, secret_purpose)
            DO UPDATE SET
                secret_ref = EXCLUDED.secret_ref,
                updated_at = now()
            RETURNING
                account_id,
                secret_purpose,
                secret_ref,
                created_at,
                updated_at
            "#,
        )
        .bind(binding.account_id.trim())
        .bind(binding.secret_purpose.as_str())
        .bind(binding.secret_ref.trim())
        .fetch_one(&mut *transaction)
        .await?;
        link_vault_owned_entity_in_transaction(
            &mut transaction,
            &observation.observation_id,
            "vault",
            "communication_provider_secret_binding",
            binding_entity_id,
            "bind",
            json!({
                "account_id": binding.account_id.trim(),
                "secret_purpose": binding.secret_purpose.as_str(),
                "secret_ref": binding.secret_ref.trim(),
            }),
            None,
        )
        .await?;
        transaction.commit().await?;

        row_to_provider_secret_binding(row)
    }

    pub async fn list_for_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<ProviderAccountSecretBinding>, CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;

        let rows = sqlx::query(
            r#"
            SELECT
                account_id,
                secret_purpose,
                secret_ref,
                created_at,
                updated_at
            FROM communication_provider_account_secret_refs
            WHERE account_id = $1
            ORDER BY secret_purpose ASC
            "#,
        )
        .bind(account_id.trim())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_provider_secret_binding)
            .collect()
    }

    pub async fn get_for_account(
        &self,
        account_id: &str,
        secret_purpose: ProviderAccountSecretPurpose,
    ) -> Result<Option<ProviderAccountSecretBinding>, CommunicationIngestionError> {
        validate_non_empty_field("account_id", account_id)?;

        let row = sqlx::query(
            r#"
            SELECT
                account_id,
                secret_purpose,
                secret_ref,
                created_at,
                updated_at
            FROM communication_provider_account_secret_refs
            WHERE account_id = $1
              AND secret_purpose = $2
            "#,
        )
        .bind(account_id.trim())
        .bind(secret_purpose.as_str())
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_provider_secret_binding).transpose()
    }
}

fn row_to_calendar_account(row: PgRow) -> Result<CalendarAccount, sqlx::Error> {
    Ok(CalendarAccount {
        account_id: row.try_get("account_id")?,
        provider: row.try_get("provider")?,
        account_name: row.try_get("account_name")?,
        email: row.try_get("email")?,
        credentials_reference: row.try_get("credentials_reference")?,
        sync_status: row.try_get("sync_status")?,
        capabilities: row.try_get("capabilities")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[allow(clippy::too_many_arguments)]
async fn link_vault_owned_entity_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    observation_id: &str,
    domain: &str,
    entity_kind: &str,
    entity_id: impl Into<String>,
    relationship_kind: &str,
    base_metadata: serde_json::Value,
    extra_metadata: Option<serde_json::Value>,
) -> Result<(), crate::platform::observations::ObservationStoreError> {
    let metadata = match extra_metadata {
        Some(extra) if base_metadata.is_object() && extra.is_object() => {
            let mut merged = base_metadata;
            if let (Some(base), Some(extra)) = (merged.as_object_mut(), extra.as_object()) {
                for (key, value) in extra {
                    base.insert(key.clone(), value.clone());
                }
            }
            merged
        }
        Some(extra) => extra,
        None => base_metadata,
    };

    link_domain_entity_in_transaction(
        transaction,
        observation_id,
        domain,
        entity_kind,
        entity_id.into(),
        Some(relationship_kind),
        None,
        Some(metadata),
    )
    .await
}

fn row_to_calendar_source(row: PgRow) -> Result<CalendarSource, sqlx::Error> {
    Ok(CalendarSource {
        source_id: row.try_get("source_id")?,
        account_id: row.try_get("account_id")?,
        provider_calendar_id: row.try_get("provider_calendar_id")?,
        name: row.try_get("name")?,
        color: row.try_get("color")?,
        timezone: row.try_get("timezone")?,
        visibility: row.try_get("visibility")?,
        read_only: row.try_get("read_only")?,
        sync_enabled: row.try_get("sync_enabled")?,
        capabilities: row.try_get("capabilities")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_task_provider_account(row: PgRow) -> Result<TaskProviderAccount, TaskCoreError> {
    Ok(TaskProviderAccount {
        account_id: row.try_get("account_id")?,
        provider: row.try_get("provider")?,
        account_name: row.try_get("account_name")?,
        credentials_reference: row.try_get("credentials_reference")?,
        sync_mode: row.try_get("sync_mode")?,
        capabilities: row.try_get("capabilities")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_provider_account(row: PgRow) -> Result<ProviderAccount, CommunicationIngestionError> {
    Ok(ProviderAccount {
        account_id: row.try_get("account_id")?,
        provider_kind: row
            .try_get::<String, _>("provider_kind")?
            .as_str()
            .try_into()?,
        display_name: row.try_get("display_name")?,
        external_account_id: row.try_get("external_account_id")?,
        config: row.try_get("config")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_ref_to_provider_account(
    row: &PgRow,
) -> Result<ProviderAccount, CommunicationIngestionError> {
    Ok(ProviderAccount {
        account_id: row.try_get("account_id")?,
        provider_kind: row
            .try_get::<String, _>("provider_kind")?
            .as_str()
            .try_into()?,
        display_name: row.try_get("display_name")?,
        external_account_id: row.try_get("external_account_id")?,
        config: row.try_get("config")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_provider_secret_binding(
    row: PgRow,
) -> Result<ProviderAccountSecretBinding, CommunicationIngestionError> {
    Ok(ProviderAccountSecretBinding {
        account_id: row.try_get("account_id")?,
        secret_purpose: ProviderAccountSecretPurpose::try_from(
            row.try_get::<String, _>("secret_purpose")?.as_str(),
        )?,
        secret_ref: row.try_get("secret_ref")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn validate_provider_account(
    account: &NewProviderAccount,
) -> Result<(), CommunicationIngestionError> {
    validate_non_empty_field("account_id", &account.account_id)?;
    validate_non_empty_field("display_name", &account.display_name)?;
    validate_non_empty_field("external_account_id", &account.external_account_id)?;
    if !account.config.is_object() {
        return Err(CommunicationIngestionError::NonObjectJson("config"));
    }
    Ok(())
}

fn validate_provider_secret_binding(
    binding: &NewProviderAccountSecretBinding,
) -> Result<(), CommunicationIngestionError> {
    validate_non_empty_field("account_id", &binding.account_id)?;
    validate_non_empty_field("secret_ref", &binding.secret_ref)?;
    Ok(())
}

fn validate_non_empty_field(
    field: &'static str,
    value: &str,
) -> Result<(), CommunicationIngestionError> {
    if value.trim().is_empty() {
        return Err(CommunicationIngestionError::EmptyField(field));
    }
    Ok(())
}

fn next_id(prefix: &str) -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{prefix}:v1:{ts:x}")
}
