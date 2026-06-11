use std::path::PathBuf;

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::domains::graph::projection::{GraphProjectionError, GraphProjectionService};
use crate::domains::mail::accounts::{EmailAccountSetupError, EmailAccountSetupService};
use crate::domains::mail::core::{
    CommunicationIngestionError, CommunicationIngestionStore, EmailProviderKind, ProviderAccount,
    ProviderAccountSecretPurpose, ProviderCredentialError, ProviderCredentialReader,
};
use crate::domains::mail::storage::LocalMailBlobStore;
use crate::domains::mail::sync::{
    EmailSyncAdapterConfig, EmailSyncBatch, EmailSyncPlanError, plan_email_sync,
};
use crate::integrations::gmail::client::{
    EmailProviderNetworkError, GmailApiClient, GmailFetchOptions, GmailHistoryFetchOptions,
    ImapFetchOptions, ImapNetworkClient,
};
use crate::platform::secrets::SecretReferenceStore;
use crate::vault::{HostVault, HostVaultError, VaultMode};
use crate::workflows::email_sync_pipeline::{
    EmailSyncPipelineError, EmailSyncPipelineReport, project_email_sync_batch_with_mail_blobs,
};

pub const DEFAULT_MAIL_SYNC_BATCH_SIZE: i32 = 5;
pub const DEFAULT_MAIL_SYNC_POLL_INTERVAL_SECONDS: i32 = 300;
pub const DEFAULT_MAIL_SYNC_BLOB_ROOT: &str = "docker/data/mail";
const MAX_BATCH_SIZE: i32 = 500;
const MIN_POLL_INTERVAL_SECONDS: i32 = 60;
const MAX_POLL_INTERVAL_SECONDS: i32 = 86_400;
const DEFAULT_GMAIL_API_BASE_URL: &str = "https://www.googleapis.com";

#[derive(Clone)]
pub struct MailBackgroundSyncService {
    pool: PgPool,
    vault: HostVault,
    blob_root: PathBuf,
    gmail_api_base_url: String,
}

impl MailBackgroundSyncService {
    pub fn new(pool: PgPool, vault: HostVault, blob_root: impl Into<PathBuf>) -> Self {
        Self {
            pool,
            vault,
            blob_root: blob_root.into(),
            gmail_api_base_url: DEFAULT_GMAIL_API_BASE_URL.to_owned(),
        }
    }

    #[cfg(test)]
    pub fn gmail_api_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.gmail_api_base_url = base_url.into();
        self
    }

    pub async fn run_due_accounts(&self) -> Result<Vec<MailSyncRunResponse>, MailSyncError> {
        let store = MailSyncStore::new(self.pool.clone());
        let accounts = store.due_accounts(Utc::now(), 20).await?;
        let mut responses = Vec::new();
        for account in accounts {
            responses.push(
                self.run_account(&account.account_id, MailSyncTrigger::Scheduled)
                    .await?,
            );
        }
        Ok(responses)
    }

    pub async fn run_account(
        &self,
        account_id: &str,
        trigger: MailSyncTrigger,
    ) -> Result<MailSyncRunResponse, MailSyncError> {
        let store = MailSyncStore::new(self.pool.clone());
        let communication_store = CommunicationIngestionStore::new(self.pool.clone());
        let account = communication_store
            .provider_account(account_id)
            .await?
            .ok_or(MailSyncError::AccountNotFound)?;
        let settings = store.settings_for_account(account_id).await?;

        if !settings.sync_enabled {
            let run = store
                .start_run(account_id, trigger, &settings, None)
                .await
                .map_err(|error| match error {
                    MailSyncError::RunAlreadyActive => MailSyncError::RunAlreadyActive,
                    other => other,
                })?;
            return store
                .finish_run(
                    &run.run_id,
                    FinishRun {
                        status: MailSyncRunStatus::Skipped,
                        phase: MailSyncPhase::Completed,
                        progress_mode: ProgressMode::None,
                        progress_percent: None,
                        processed_messages: run.processed_messages,
                        estimated_total_messages: run.estimated_total_messages,
                        fetched_messages: 0,
                        projected_messages: 0,
                        upserted_persons: 0,
                        upserted_organizations: 0,
                        checkpoint_after: None,
                        checkpoint_saved: false,
                        error_code: Some("sync_disabled".to_owned()),
                        error_message: Some("Mail sync is disabled for this account".to_owned()),
                        next_run_at: next_run_at(&settings),
                    },
                )
                .await
                .map(Into::into);
        }

        let plan = match plan_email_sync(&account) {
            Ok(plan) => plan,
            Err(error) => {
                return self
                    .fail_without_provider_io(
                        account_id,
                        trigger,
                        &settings,
                        None,
                        SanitizedSyncFailure::from_plan(error),
                    )
                    .await;
            }
        };
        let checkpoint_before = communication_store
            .checkpoint(account_id, &plan.stream_id)
            .await?
            .map(|checkpoint| checkpoint.checkpoint);

        let run = match store
            .start_run(account_id, trigger, &settings, checkpoint_before.clone())
            .await
        {
            Ok(run) => run,
            Err(MailSyncError::RunAlreadyActive) => {
                return store.latest_run_response(account_id).await;
            }
            Err(error) => return Err(error),
        };

        if let Err(error) = require_unlocked_vault(&self.vault) {
            return store
                .finish_run(
                    &run.run_id,
                    FinishRun::failed(
                        MailSyncPhase::Failed,
                        SanitizedSyncFailure::from_vault(error),
                        &settings,
                    ),
                )
                .await
                .map(Into::into);
        }

        let result = self
            .execute_provider_sync(
                &plan.adapter_config,
                ProviderSyncContext {
                    store: &store,
                    communication_store: &communication_store,
                    account: &account,
                    run_id: &run.run_id,
                    settings: &settings,
                    checkpoint_before,
                },
            )
            .await;

        match result {
            Ok(summary) => store
                .finish_run(
                    &run.run_id,
                    FinishRun {
                        status: MailSyncRunStatus::Completed,
                        phase: MailSyncPhase::Completed,
                        progress_mode: ProgressMode::Determinate,
                        progress_percent: Some(100),
                        processed_messages: summary.processed_messages,
                        estimated_total_messages: summary.estimated_total_messages,
                        fetched_messages: summary.fetched_messages,
                        projected_messages: summary.projected_messages,
                        upserted_persons: summary.upserted_persons,
                        upserted_organizations: summary.upserted_organizations,
                        checkpoint_after: summary.checkpoint_after,
                        checkpoint_saved: summary.checkpoint_saved,
                        error_code: None,
                        error_message: None,
                        next_run_at: next_run_at(&settings),
                    },
                )
                .await
                .map(Into::into),
            Err(error) => store
                .finish_run(
                    &run.run_id,
                    FinishRun::failed(
                        MailSyncPhase::Failed,
                        SanitizedSyncFailure::from(error),
                        &settings,
                    ),
                )
                .await
                .map(Into::into),
        }
    }

    async fn fail_without_provider_io(
        &self,
        account_id: &str,
        trigger: MailSyncTrigger,
        settings: &MailSyncSettings,
        checkpoint_before: Option<Value>,
        failure: SanitizedSyncFailure,
    ) -> Result<MailSyncRunResponse, MailSyncError> {
        let store = MailSyncStore::new(self.pool.clone());
        let run = match store
            .start_run(account_id, trigger, settings, checkpoint_before)
            .await
        {
            Ok(run) => run,
            Err(MailSyncError::RunAlreadyActive) => {
                return store.latest_run_response(account_id).await;
            }
            Err(error) => return Err(error),
        };
        store
            .finish_run(
                &run.run_id,
                FinishRun::failed(MailSyncPhase::Failed, failure, settings),
            )
            .await
            .map(Into::into)
    }

    async fn execute_provider_sync(
        &self,
        adapter: &EmailSyncAdapterConfig,
        context: ProviderSyncContext<'_>,
    ) -> Result<ProviderSyncSummary, ProviderSyncError> {
        match adapter {
            EmailSyncAdapterConfig::Gmail { .. } => self.sync_gmail(context).await,
            EmailSyncAdapterConfig::Imap {
                host,
                port,
                tls,
                mailbox,
            } => {
                self.sync_imap(
                    context,
                    ImapAccountConfig {
                        host,
                        port: *port,
                        tls: *tls,
                        mailbox,
                    },
                )
                .await
            }
        }
    }

    async fn sync_gmail(
        &self,
        context: ProviderSyncContext<'_>,
    ) -> Result<ProviderSyncSummary, ProviderSyncError> {
        let binding = context
            .communication_store
            .provider_account_secret_binding(
                &context.account.account_id,
                ProviderAccountSecretPurpose::OauthToken,
            )
            .await?
            .ok_or(ProviderSyncError::MissingCredential)?;
        let account_setup = EmailAccountSetupService::new_with_host_vault(
            context.communication_store.clone(),
            SecretReferenceStore::new(self.pool.clone()),
            self.vault.clone(),
        );
        let access_token = account_setup
            .refresh_gmail_access_token(&binding.secret_ref)
            .await?;
        let client = GmailApiClient::new(&self.gmail_api_base_url).user_id("me");
        let mut summary = ProviderSyncSummary::default();

        if let Some(history_id) = context
            .checkpoint_before
            .as_ref()
            .and_then(|checkpoint| checkpoint.get("history_id"))
            .and_then(Value::as_str)
            .map(str::to_owned)
        {
            context
                .store
                .update_progress(ProgressUpdate {
                    run_id: context.run_id,
                    phase: MailSyncPhase::Listing,
                    progress_mode: ProgressMode::Indeterminate,
                    progress_percent: None,
                    processed_messages: summary.processed_messages,
                    estimated_total_messages: summary.estimated_total_messages,
                    current_batch_size: context.settings.batch_size,
                })
                .await?;
            let history_batch = client
                .fetch_history_raw_messages(
                    &access_token,
                    &GmailHistoryFetchOptions::new(history_id, context.settings.batch_size as u16),
                )
                .await;
            match history_batch {
                Ok(batch) => {
                    self.project_batch(
                        context.store,
                        context.run_id,
                        context.settings,
                        &mut summary,
                        &context.account.account_id,
                        batch,
                    )
                    .await?;
                    return Ok(summary);
                }
                Err(error) if gmail_history_expired(&error) => {
                    context
                        .store
                        .mark_recoverable_full_resync(context.run_id, "gmail_history_expired")
                        .await?;
                }
                Err(error) => return Err(error.into()),
            }
        }

        let mut page_token = context
            .checkpoint_before
            .as_ref()
            .and_then(|checkpoint| checkpoint.get("next_page_token"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        loop {
            context
                .store
                .update_progress(ProgressUpdate {
                    run_id: context.run_id,
                    phase: MailSyncPhase::Listing,
                    progress_mode: ProgressMode::Indeterminate,
                    progress_percent: None,
                    processed_messages: summary.processed_messages,
                    estimated_total_messages: summary.estimated_total_messages,
                    current_batch_size: context.settings.batch_size,
                })
                .await?;
            let mut options = GmailFetchOptions::new(context.settings.batch_size as u16);
            if let Some(token) = page_token {
                options = options.page_token(token);
            }
            let batch = client.fetch_raw_messages(&access_token, &options).await?;
            page_token = batch
                .checkpoint
                .as_ref()
                .and_then(|checkpoint| checkpoint.get("next_page_token"))
                .and_then(Value::as_str)
                .map(str::to_owned);
            let fetched_count = batch.messages.len();
            self.project_batch(
                context.store,
                context.run_id,
                context.settings,
                &mut summary,
                &context.account.account_id,
                batch,
            )
            .await?;
            if page_token.is_none() || fetched_count == 0 {
                break;
            }
        }

        Ok(summary)
    }

    async fn sync_imap(
        &self,
        context: ProviderSyncContext<'_>,
        config: ImapAccountConfig<'_>,
    ) -> Result<ProviderSyncSummary, ProviderSyncError> {
        let credential_reader = ProviderCredentialReader::new(
            context.communication_store.clone(),
            SecretReferenceStore::new(self.pool.clone()),
            &self.vault,
        );
        let credential = credential_reader
            .read(
                &context.account.account_id,
                ProviderAccountSecretPurpose::ImapPassword,
            )
            .await?;
        let client = ImapNetworkClient::new();
        let mut summary = ProviderSyncSummary::default();
        let mut last_seen_uid = context
            .checkpoint_before
            .as_ref()
            .and_then(|checkpoint| checkpoint.get("last_seen_uid"))
            .and_then(Value::as_u64)
            .and_then(|uid| u32::try_from(uid).ok());
        let checkpoint_uid_validity = context
            .checkpoint_before
            .as_ref()
            .and_then(|checkpoint| checkpoint.get("uid_validity"))
            .and_then(Value::as_u64)
            .and_then(|uid_validity| u32::try_from(uid_validity).ok());
        let mut retried_after_uid_validity_reset = false;

        loop {
            context
                .store
                .update_progress(ProgressUpdate {
                    run_id: context.run_id,
                    phase: MailSyncPhase::Fetching,
                    progress_mode: ProgressMode::Indeterminate,
                    progress_percent: None,
                    processed_messages: summary.processed_messages,
                    estimated_total_messages: summary.estimated_total_messages,
                    current_batch_size: context.settings.batch_size,
                })
                .await?;
            let mut options = ImapFetchOptions::new(
                config.host,
                config.port,
                config.tls,
                config.mailbox,
                &context.account.external_account_id,
            )
            .provider_kind(context.account.provider_kind)
            .max_messages(context.settings.batch_size as usize);
            if let Some(uid) = last_seen_uid {
                options = options.last_seen_uid(uid);
            }
            let batch = client
                .fetch_raw_messages(&credential.secret, &options)
                .await?;
            let fetched_count = batch.messages.len();
            let batch_uid_validity = batch
                .checkpoint
                .as_ref()
                .and_then(|checkpoint| checkpoint.get("uid_validity"))
                .and_then(Value::as_u64)
                .and_then(|uid_validity| u32::try_from(uid_validity).ok());
            if !retried_after_uid_validity_reset
                && checkpoint_uid_validity.is_some()
                && batch_uid_validity.is_some()
                && checkpoint_uid_validity != batch_uid_validity
            {
                retried_after_uid_validity_reset = true;
                last_seen_uid = None;
                continue;
            }

            last_seen_uid = batch
                .checkpoint
                .as_ref()
                .and_then(|checkpoint| checkpoint.get("last_seen_uid"))
                .and_then(Value::as_u64)
                .and_then(|uid| u32::try_from(uid).ok())
                .or(last_seen_uid);
            self.project_batch(
                context.store,
                context.run_id,
                context.settings,
                &mut summary,
                &context.account.account_id,
                batch,
            )
            .await?;
            if fetched_count == 0 {
                break;
            }
        }

        Ok(summary)
    }

    async fn project_batch(
        &self,
        store: &MailSyncStore,
        run_id: &str,
        settings: &MailSyncSettings,
        summary: &mut ProviderSyncSummary,
        account_id: &str,
        batch: EmailSyncBatch,
    ) -> Result<(), ProviderSyncError> {
        let fetched_count = batch.messages.len() as i64;
        summary.fetched_messages += fetched_count;
        summary.processed_messages += fetched_count;
        summary.current_batch_size = i32::try_from(fetched_count).unwrap_or(i32::MAX);
        summary.checkpoint_after = batch.checkpoint.clone();

        store
            .update_progress(ProgressUpdate {
                run_id,
                phase: MailSyncPhase::Projecting,
                progress_mode: ProgressMode::Indeterminate,
                progress_percent: None,
                processed_messages: summary.processed_messages,
                estimated_total_messages: summary.estimated_total_messages,
                current_batch_size: settings.batch_size,
            })
            .await?;

        let blob_store = LocalMailBlobStore::new(&self.blob_root);
        let report = project_email_sync_batch_with_mail_blobs(
            self.pool.clone(),
            &blob_store,
            account_id,
            &format!("{run_id}:batch:{}", summary.processed_messages),
            &batch,
        )
        .await?;
        summary.apply_pipeline_report(&report);

        store
            .update_progress(ProgressUpdate {
                run_id,
                phase: MailSyncPhase::PersonsGraph,
                progress_mode: ProgressMode::Indeterminate,
                progress_percent: None,
                processed_messages: summary.processed_messages,
                estimated_total_messages: summary.estimated_total_messages,
                current_batch_size: settings.batch_size,
            })
            .await?;

        GraphProjectionService::new(self.pool.clone())
            .project_from_v1()
            .await?;

        if batch.checkpoint.is_some() {
            summary.checkpoint_saved = report.checkpoint_saved;
        }

        Ok(())
    }
}

struct ProviderSyncContext<'a> {
    store: &'a MailSyncStore,
    communication_store: &'a CommunicationIngestionStore,
    account: &'a ProviderAccount,
    run_id: &'a str,
    settings: &'a MailSyncSettings,
    checkpoint_before: Option<Value>,
}

#[derive(Clone, Copy)]
struct ImapAccountConfig<'a> {
    host: &'a str,
    port: u16,
    tls: bool,
    mailbox: &'a str,
}

struct ProgressUpdate<'a> {
    run_id: &'a str,
    phase: MailSyncPhase,
    progress_mode: ProgressMode,
    progress_percent: Option<i32>,
    processed_messages: i64,
    estimated_total_messages: Option<i64>,
    current_batch_size: i32,
}

#[derive(Default)]
struct ProviderSyncSummary {
    processed_messages: i64,
    estimated_total_messages: Option<i64>,
    current_batch_size: i32,
    fetched_messages: i64,
    projected_messages: i64,
    upserted_persons: i64,
    upserted_organizations: i64,
    checkpoint_after: Option<Value>,
    checkpoint_saved: bool,
}

impl ProviderSyncSummary {
    fn apply_pipeline_report(&mut self, report: &EmailSyncPipelineReport) {
        self.projected_messages += report.projected_messages as i64;
        self.upserted_persons += report.upserted_person_identities as i64;
        self.upserted_organizations += report.upserted_organizations as i64;
    }
}

#[derive(Clone)]
pub struct MailSyncStore {
    pool: PgPool,
}

impl MailSyncStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn mark_orphaned_active_runs_failed(
        &self,
        now: DateTime<Utc>,
    ) -> Result<u64, MailSyncError> {
        let result = sqlx::query(
            r#"
            UPDATE communication_mail_sync_runs
            SET
                status = 'failed',
                phase = 'failed',
                progress_mode = 'none',
                progress_percent = NULL,
                error_code = 'backend_restarted',
                error_message = 'Mail sync run was interrupted by backend restart',
                completed_at = $1,
                next_run_at = $1,
                updated_at = $1
            WHERE status IN ('queued', 'running', 'recoverable_full_resync_needed')
            "#,
        )
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn settings_for_account(
        &self,
        account_id: &str,
    ) -> Result<MailSyncSettings, MailSyncError> {
        validate_account_id(account_id)?;
        self.require_account(account_id).await?;
        let row = sqlx::query(
            r#"
            INSERT INTO communication_account_sync_settings (account_id)
            VALUES ($1)
            ON CONFLICT (account_id) DO UPDATE SET account_id = EXCLUDED.account_id
            RETURNING account_id, sync_enabled, batch_size, poll_interval_seconds, updated_at
            "#,
        )
        .bind(account_id.trim())
        .fetch_one(&self.pool)
        .await?;

        row_to_settings(row)
    }

    pub async fn update_settings(
        &self,
        account_id: &str,
        update: MailSyncSettingsUpdate,
    ) -> Result<MailSyncSettings, MailSyncError> {
        validate_account_id(account_id)?;
        validate_settings(update.batch_size, update.poll_interval_seconds)?;
        self.require_account(account_id).await?;
        let row = sqlx::query(
            r#"
            INSERT INTO communication_account_sync_settings (
                account_id,
                sync_enabled,
                batch_size,
                poll_interval_seconds,
                updated_at
            )
            VALUES ($1, $2, $3, $4, now())
            ON CONFLICT (account_id)
            DO UPDATE SET
                sync_enabled = EXCLUDED.sync_enabled,
                batch_size = EXCLUDED.batch_size,
                poll_interval_seconds = EXCLUDED.poll_interval_seconds,
                updated_at = now()
            RETURNING account_id, sync_enabled, batch_size, poll_interval_seconds, updated_at
            "#,
        )
        .bind(account_id.trim())
        .bind(update.sync_enabled)
        .bind(update.batch_size)
        .bind(update.poll_interval_seconds)
        .fetch_one(&self.pool)
        .await?;

        row_to_settings(row)
    }

    pub async fn sync_statuses(&self) -> Result<Vec<MailSyncStatus>, MailSyncError> {
        let rows = sqlx::query(
            r#"
            WITH latest AS (
                SELECT DISTINCT ON (account_id)
                    account_id,
                    status,
                    phase,
                    progress_mode,
                    progress_percent,
                    processed_messages,
                    estimated_total_messages,
                    current_batch_size,
                    started_at,
                    completed_at,
                    next_run_at,
                    error_code,
                    error_message,
                    fetched_messages,
                    projected_messages,
                    upserted_persons,
                    upserted_organizations
                FROM communication_mail_sync_runs
                ORDER BY account_id, started_at DESC
            )
            SELECT
                a.account_id,
                COALESCE(latest.status, 'idle') AS status,
                COALESCE(latest.phase, 'idle') AS phase,
                COALESCE(latest.progress_mode, 'none') AS progress_mode,
                latest.progress_percent,
                COALESCE(latest.processed_messages, 0) AS processed_messages,
                latest.estimated_total_messages,
                COALESCE(latest.current_batch_size, COALESCE(settings.batch_size, $1)) AS current_batch_size,
                latest.started_at AS last_started_at,
                latest.completed_at AS last_completed_at,
                COALESCE(
                    latest.next_run_at,
                    CASE
                        WHEN COALESCE(settings.sync_enabled, true) THEN now()
                        ELSE NULL
                    END
                ) AS next_run_at,
                latest.error_code AS last_error_code,
                latest.error_message AS last_error_message,
                COALESCE(latest.fetched_messages, 0) AS last_fetched_messages,
                COALESCE(latest.projected_messages, 0) AS last_projected_messages,
                COALESCE(latest.upserted_persons, 0) AS last_upserted_persons,
                COALESCE(latest.upserted_organizations, 0) AS last_upserted_organizations
            FROM communication_provider_accounts a
            LEFT JOIN communication_account_sync_settings settings ON settings.account_id = a.account_id
            LEFT JOIN latest ON latest.account_id = a.account_id
            WHERE a.provider_kind IN ('gmail', 'icloud', 'imap')
            ORDER BY a.display_name ASC, a.account_id ASC
            "#,
        )
        .bind(DEFAULT_MAIL_SYNC_BATCH_SIZE)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_status).collect()
    }

    pub async fn due_accounts(
        &self,
        now: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<MailSyncDueAccount>, MailSyncError> {
        let rows = sqlx::query(
            r#"
            WITH latest AS (
                SELECT DISTINCT ON (account_id)
                    account_id,
                    status,
                    completed_at,
                    next_run_at
                FROM communication_mail_sync_runs
                ORDER BY account_id, started_at DESC
            )
            SELECT
                a.account_id,
                COALESCE(settings.batch_size, $2) AS batch_size,
                COALESCE(settings.poll_interval_seconds, $3) AS poll_interval_seconds
            FROM communication_provider_accounts a
            LEFT JOIN communication_account_sync_settings settings ON settings.account_id = a.account_id
            LEFT JOIN latest ON latest.account_id = a.account_id
            WHERE a.provider_kind IN ('gmail', 'icloud', 'imap')
              AND COALESCE(settings.sync_enabled, true)
              AND NOT EXISTS (
                  SELECT 1
                  FROM communication_mail_sync_runs active
                  WHERE active.account_id = a.account_id
                    AND active.status IN ('queued', 'running', 'recoverable_full_resync_needed')
              )
              AND (
                  COALESCE(
                      latest.next_run_at,
                      latest.completed_at + (COALESCE(settings.poll_interval_seconds, $3)::text || ' seconds')::interval,
                      $1
                  ) <= $1
              )
            ORDER BY latest.completed_at ASC NULLS FIRST, a.account_id ASC
            LIMIT $4
            "#,
        )
        .bind(now)
        .bind(DEFAULT_MAIL_SYNC_BATCH_SIZE)
        .bind(DEFAULT_MAIL_SYNC_POLL_INTERVAL_SECONDS)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_due_account).collect()
    }

    async fn start_run(
        &self,
        account_id: &str,
        trigger: MailSyncTrigger,
        settings: &MailSyncSettings,
        checkpoint_before: Option<Value>,
    ) -> Result<MailSyncRun, MailSyncError> {
        validate_account_id(account_id)?;
        let run_id = mail_sync_run_id(account_id);
        let result = sqlx::query(
            r#"
            INSERT INTO communication_mail_sync_runs (
                run_id,
                account_id,
                trigger,
                status,
                phase,
                progress_mode,
                current_batch_size,
                checkpoint_before
            )
            VALUES ($1, $2, $3, 'running', 'listing', 'indeterminate', $4, $5)
            RETURNING
                run_id,
                account_id,
                trigger,
                status,
                phase,
                progress_mode,
                progress_percent,
                processed_messages,
                estimated_total_messages,
                current_batch_size,
                fetched_messages,
                projected_messages,
                upserted_persons,
                upserted_organizations,
                checkpoint_before,
                checkpoint_after,
                checkpoint_saved,
                error_code,
                error_message,
                started_at,
                completed_at,
                next_run_at
            "#,
        )
        .bind(&run_id)
        .bind(account_id.trim())
        .bind(trigger.as_str())
        .bind(settings.batch_size)
        .bind(checkpoint_before.unwrap_or_else(|| json!({})))
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(row) => row_to_run(row),
            Err(sqlx::Error::Database(error)) if error.is_unique_violation() => {
                Err(MailSyncError::RunAlreadyActive)
            }
            Err(error) => Err(MailSyncError::Sqlx(error)),
        }
    }

    async fn update_progress(&self, update: ProgressUpdate<'_>) -> Result<(), MailSyncError> {
        sqlx::query(
            r#"
            UPDATE communication_mail_sync_runs
            SET
                status = 'running',
                phase = $2,
                progress_mode = $3,
                progress_percent = $4,
                processed_messages = $5,
                estimated_total_messages = $6,
                current_batch_size = $7,
                updated_at = now()
            WHERE run_id = $1
            "#,
        )
        .bind(update.run_id)
        .bind(update.phase.as_str())
        .bind(update.progress_mode.as_str())
        .bind(update.progress_percent)
        .bind(update.processed_messages)
        .bind(update.estimated_total_messages)
        .bind(update.current_batch_size)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn mark_recoverable_full_resync(
        &self,
        run_id: &str,
        error_code: &'static str,
    ) -> Result<(), MailSyncError> {
        sqlx::query(
            r#"
            UPDATE communication_mail_sync_runs
            SET
                status = 'recoverable_full_resync_needed',
                phase = 'listing',
                progress_mode = 'indeterminate',
                error_code = $2,
                error_message = 'Gmail history expired; restarting full mailbox listing',
                updated_at = now()
            WHERE run_id = $1
            "#,
        )
        .bind(run_id)
        .bind(error_code)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn finish_run(
        &self,
        run_id: &str,
        finish: FinishRun,
    ) -> Result<MailSyncRun, MailSyncError> {
        let row = sqlx::query(
            r#"
            UPDATE communication_mail_sync_runs
            SET
                status = $2,
                phase = $3,
                progress_mode = $4,
                progress_percent = $5,
                processed_messages = $6,
                estimated_total_messages = $7,
                fetched_messages = $8,
                projected_messages = $9,
                upserted_persons = $10,
                upserted_organizations = $11,
                checkpoint_after = $12,
                checkpoint_saved = $13,
                error_code = $14,
                error_message = $15,
                completed_at = now(),
                next_run_at = $16,
                updated_at = now()
            WHERE run_id = $1
            RETURNING
                run_id,
                account_id,
                trigger,
                status,
                phase,
                progress_mode,
                progress_percent,
                processed_messages,
                estimated_total_messages,
                current_batch_size,
                fetched_messages,
                projected_messages,
                upserted_persons,
                upserted_organizations,
                checkpoint_before,
                checkpoint_after,
                checkpoint_saved,
                error_code,
                error_message,
                started_at,
                completed_at,
                next_run_at
            "#,
        )
        .bind(run_id)
        .bind(finish.status.as_str())
        .bind(finish.phase.as_str())
        .bind(finish.progress_mode.as_str())
        .bind(finish.progress_percent)
        .bind(finish.processed_messages)
        .bind(finish.estimated_total_messages)
        .bind(finish.fetched_messages)
        .bind(finish.projected_messages)
        .bind(finish.upserted_persons)
        .bind(finish.upserted_organizations)
        .bind(finish.checkpoint_after.unwrap_or_else(|| json!({})))
        .bind(finish.checkpoint_saved)
        .bind(finish.error_code)
        .bind(finish.error_message)
        .bind(finish.next_run_at)
        .fetch_one(&self.pool)
        .await?;

        row_to_run(row)
    }

    async fn latest_run_response(
        &self,
        account_id: &str,
    ) -> Result<MailSyncRunResponse, MailSyncError> {
        let row = sqlx::query(
            r#"
            SELECT
                run_id,
                account_id,
                trigger,
                status,
                phase,
                progress_mode,
                progress_percent,
                processed_messages,
                estimated_total_messages,
                current_batch_size,
                fetched_messages,
                projected_messages,
                upserted_persons,
                upserted_organizations,
                checkpoint_before,
                checkpoint_after,
                checkpoint_saved,
                error_code,
                error_message,
                started_at,
                completed_at,
                next_run_at
            FROM communication_mail_sync_runs
            WHERE account_id = $1
            ORDER BY started_at DESC
            LIMIT 1
            "#,
        )
        .bind(account_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Err(MailSyncError::RunNotFound);
        };

        row_to_run(row).map(Into::into)
    }

    async fn require_account(&self, account_id: &str) -> Result<(), MailSyncError> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM communication_provider_accounts WHERE account_id = $1)",
        )
        .bind(account_id.trim())
        .fetch_one(&self.pool)
        .await?;
        if exists {
            Ok(())
        } else {
            Err(MailSyncError::AccountNotFound)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MailSyncSettings {
    pub account_id: String,
    pub sync_enabled: bool,
    pub batch_size: i32,
    pub poll_interval_seconds: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub struct MailSyncSettingsUpdate {
    pub sync_enabled: bool,
    pub batch_size: i32,
    pub poll_interval_seconds: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MailSyncStatus {
    pub account_id: String,
    pub status: String,
    pub phase: String,
    pub progress_mode: String,
    pub progress_percent: Option<i32>,
    pub processed_messages: i64,
    pub estimated_total_messages: Option<i64>,
    pub current_batch_size: i32,
    pub last_started_at: Option<DateTime<Utc>>,
    pub last_completed_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_error_code: Option<String>,
    pub last_error_message: Option<String>,
    pub last_fetched_messages: i64,
    pub last_projected_messages: i64,
    pub last_upserted_persons: i64,
    pub last_upserted_organizations: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailSyncRun {
    pub run_id: String,
    pub account_id: String,
    pub trigger: String,
    pub status: String,
    pub phase: String,
    pub progress_mode: String,
    pub progress_percent: Option<i32>,
    pub processed_messages: i64,
    pub estimated_total_messages: Option<i64>,
    pub current_batch_size: i32,
    pub fetched_messages: i64,
    pub projected_messages: i64,
    pub upserted_persons: i64,
    pub upserted_organizations: i64,
    pub checkpoint_before: Option<Value>,
    pub checkpoint_after: Option<Value>,
    pub checkpoint_saved: bool,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MailSyncRunResponse {
    pub run_id: String,
    pub account_id: String,
    pub trigger: String,
    pub status: String,
    pub phase: String,
    pub progress_mode: String,
    pub progress_percent: Option<i32>,
    pub processed_messages: i64,
    pub estimated_total_messages: Option<i64>,
    pub current_batch_size: i32,
    pub fetched_messages: i64,
    pub projected_messages: i64,
    pub upserted_persons: i64,
    pub upserted_organizations: i64,
    pub checkpoint_before_present: bool,
    pub checkpoint_after_present: bool,
    pub checkpoint_saved: bool,
    pub failure_reason: Option<MailSyncFailureReason>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
}

impl From<MailSyncRun> for MailSyncRunResponse {
    fn from(run: MailSyncRun) -> Self {
        Self {
            run_id: run.run_id,
            account_id: run.account_id,
            trigger: run.trigger,
            status: run.status,
            phase: run.phase,
            progress_mode: run.progress_mode,
            progress_percent: run.progress_percent,
            processed_messages: run.processed_messages,
            estimated_total_messages: run.estimated_total_messages,
            current_batch_size: run.current_batch_size,
            fetched_messages: run.fetched_messages,
            projected_messages: run.projected_messages,
            upserted_persons: run.upserted_persons,
            upserted_organizations: run.upserted_organizations,
            checkpoint_before_present: checkpoint_is_present(run.checkpoint_before.as_ref()),
            checkpoint_after_present: checkpoint_is_present(run.checkpoint_after.as_ref()),
            checkpoint_saved: run.checkpoint_saved,
            failure_reason: run.error_code.map(|code| MailSyncFailureReason {
                code,
                message: run
                    .error_message
                    .unwrap_or_else(|| "Mail sync failed".to_owned()),
            }),
            started_at: run.started_at,
            completed_at: run.completed_at,
            next_run_at: run.next_run_at,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MailSyncFailureReason {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailSyncDueAccount {
    pub account_id: String,
    pub batch_size: i32,
    pub poll_interval_seconds: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailSyncTrigger {
    Scheduled,
    Manual,
}

impl MailSyncTrigger {
    fn as_str(self) -> &'static str {
        match self {
            Self::Scheduled => "scheduled",
            Self::Manual => "manual",
        }
    }
}

#[derive(Clone, Copy)]
enum MailSyncRunStatus {
    Completed,
    Failed,
    Skipped,
}

impl MailSyncRunStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

#[derive(Clone, Copy)]
enum MailSyncPhase {
    Listing,
    Fetching,
    Projecting,
    PersonsGraph,
    Completed,
    Failed,
}

impl MailSyncPhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::Listing => "listing",
            Self::Fetching => "fetching",
            Self::Projecting => "projecting",
            Self::PersonsGraph => "persons_graph",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy)]
enum ProgressMode {
    None,
    Determinate,
    Indeterminate,
}

impl ProgressMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Determinate => "determinate",
            Self::Indeterminate => "indeterminate",
        }
    }
}

struct FinishRun {
    status: MailSyncRunStatus,
    phase: MailSyncPhase,
    progress_mode: ProgressMode,
    progress_percent: Option<i32>,
    processed_messages: i64,
    estimated_total_messages: Option<i64>,
    fetched_messages: i64,
    projected_messages: i64,
    upserted_persons: i64,
    upserted_organizations: i64,
    checkpoint_after: Option<Value>,
    checkpoint_saved: bool,
    error_code: Option<String>,
    error_message: Option<String>,
    next_run_at: Option<DateTime<Utc>>,
}

impl FinishRun {
    fn failed(
        phase: MailSyncPhase,
        failure: SanitizedSyncFailure,
        settings: &MailSyncSettings,
    ) -> Self {
        Self {
            status: MailSyncRunStatus::Failed,
            phase,
            progress_mode: ProgressMode::None,
            progress_percent: None,
            processed_messages: 0,
            estimated_total_messages: None,
            fetched_messages: 0,
            projected_messages: 0,
            upserted_persons: 0,
            upserted_organizations: 0,
            checkpoint_after: None,
            checkpoint_saved: false,
            error_code: Some(failure.code),
            error_message: Some(failure.message),
            next_run_at: next_run_at(settings),
        }
    }
}

#[derive(Debug)]
struct SanitizedSyncFailure {
    code: String,
    message: String,
}

impl SanitizedSyncFailure {
    fn from_plan(error: EmailSyncPlanError) -> Self {
        tracing::warn!(error = %error, "mail sync provider configuration is invalid");
        Self {
            code: "provider_config_invalid".to_owned(),
            message: "Mail provider configuration is invalid".to_owned(),
        }
    }

    fn from_vault(error: HostVaultError) -> Self {
        match error {
            HostVaultError::Locked => Self {
                code: "vault_locked".to_owned(),
                message: "Host vault is locked".to_owned(),
            },
            HostVaultError::Uninitialized => Self {
                code: "vault_uninitialized".to_owned(),
                message: "Host vault is not initialized".to_owned(),
            },
            other => {
                tracing::warn!(error = %other, "mail sync vault check failed");
                Self {
                    code: "vault_unavailable".to_owned(),
                    message: "Host vault is unavailable".to_owned(),
                }
            }
        }
    }
}

impl From<ProviderSyncError> for SanitizedSyncFailure {
    fn from(error: ProviderSyncError) -> Self {
        match error {
            ProviderSyncError::MissingCredential | ProviderSyncError::Credential(_) => Self {
                code: "credential_unavailable".to_owned(),
                message: "Provider credential is unavailable for this account".to_owned(),
            },
            ProviderSyncError::AccountSetup(_) => Self {
                code: "oauth_refresh_failed".to_owned(),
                message: "OAuth access token refresh failed".to_owned(),
            },
            ProviderSyncError::ProviderNetwork(error) => {
                tracing::warn!(error = %error, "mail provider sync network call failed");
                Self {
                    code: "provider_network_error".to_owned(),
                    message: "Mail provider network request failed".to_owned(),
                }
            }
            ProviderSyncError::Pipeline(error) => {
                tracing::error!(error = %error, "mail sync projection pipeline failed");
                Self {
                    code: "projection_failed".to_owned(),
                    message: "Mail sync projection failed".to_owned(),
                }
            }
            ProviderSyncError::Graph(error) => {
                tracing::error!(error = %error, "mail sync graph projection failed");
                Self {
                    code: "graph_projection_failed".to_owned(),
                    message: "Mail graph projection failed".to_owned(),
                }
            }
            ProviderSyncError::Communication(error) => {
                tracing::error!(error = %error, "mail sync communication store failed");
                Self {
                    code: "communication_store_error".to_owned(),
                    message: "Mail sync communication store failed".to_owned(),
                }
            }
            ProviderSyncError::SyncStore(error) => {
                tracing::error!(error = %error, "mail sync status store failed");
                Self {
                    code: "sync_store_error".to_owned(),
                    message: "Mail sync status store failed".to_owned(),
                }
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum MailSyncError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error("mail sync account was not found")]
    AccountNotFound,

    #[error("mail sync run is already active for account")]
    RunAlreadyActive,

    #[error("mail sync run was not found")]
    RunNotFound,

    #[error("invalid mail sync setting {field}: {message}")]
    InvalidSetting {
        field: &'static str,
        message: &'static str,
    },
}

#[derive(Debug, Error)]
enum ProviderSyncError {
    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    Credential(#[from] ProviderCredentialError),

    #[error(transparent)]
    AccountSetup(#[from] EmailAccountSetupError),

    #[error(transparent)]
    ProviderNetwork(#[from] EmailProviderNetworkError),

    #[error(transparent)]
    Pipeline(#[from] EmailSyncPipelineError),

    #[error(transparent)]
    Graph(#[from] GraphProjectionError),

    #[error(transparent)]
    SyncStore(#[from] MailSyncError),

    #[error("missing provider credential binding")]
    MissingCredential,
}

fn require_unlocked_vault(vault: &HostVault) -> Result<(), HostVaultError> {
    match vault.status()?.state {
        VaultMode::Unlocked => Ok(()),
        VaultMode::Locked => Err(HostVaultError::Locked),
        VaultMode::Uninitialized => Err(HostVaultError::Uninitialized),
    }
}

fn gmail_history_expired(error: &EmailProviderNetworkError) -> bool {
    matches!(
        error,
        EmailProviderNetworkError::Http(source)
            if source.status().is_some_and(|status| status.as_u16() == 404)
    )
}

fn row_to_settings(row: PgRow) -> Result<MailSyncSettings, MailSyncError> {
    Ok(MailSyncSettings {
        account_id: row.try_get("account_id")?,
        sync_enabled: row.try_get("sync_enabled")?,
        batch_size: row.try_get("batch_size")?,
        poll_interval_seconds: row.try_get("poll_interval_seconds")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_status(row: PgRow) -> Result<MailSyncStatus, MailSyncError> {
    Ok(MailSyncStatus {
        account_id: row.try_get("account_id")?,
        status: row.try_get("status")?,
        phase: row.try_get("phase")?,
        progress_mode: row.try_get("progress_mode")?,
        progress_percent: row.try_get("progress_percent")?,
        processed_messages: row.try_get("processed_messages")?,
        estimated_total_messages: row.try_get("estimated_total_messages")?,
        current_batch_size: row.try_get("current_batch_size")?,
        last_started_at: row.try_get("last_started_at")?,
        last_completed_at: row.try_get("last_completed_at")?,
        next_run_at: row.try_get("next_run_at")?,
        last_error_code: row.try_get("last_error_code")?,
        last_error_message: row.try_get("last_error_message")?,
        last_fetched_messages: row.try_get("last_fetched_messages")?,
        last_projected_messages: row.try_get("last_projected_messages")?,
        last_upserted_persons: row.try_get("last_upserted_persons")?,
        last_upserted_organizations: row.try_get("last_upserted_organizations")?,
    })
}

fn row_to_due_account(row: PgRow) -> Result<MailSyncDueAccount, MailSyncError> {
    Ok(MailSyncDueAccount {
        account_id: row.try_get("account_id")?,
        batch_size: row.try_get("batch_size")?,
        poll_interval_seconds: row.try_get("poll_interval_seconds")?,
    })
}

fn row_to_run(row: PgRow) -> Result<MailSyncRun, MailSyncError> {
    Ok(MailSyncRun {
        run_id: row.try_get("run_id")?,
        account_id: row.try_get("account_id")?,
        trigger: row.try_get("trigger")?,
        status: row.try_get("status")?,
        phase: row.try_get("phase")?,
        progress_mode: row.try_get("progress_mode")?,
        progress_percent: row.try_get("progress_percent")?,
        processed_messages: row.try_get("processed_messages")?,
        estimated_total_messages: row.try_get("estimated_total_messages")?,
        current_batch_size: row.try_get("current_batch_size")?,
        fetched_messages: row.try_get("fetched_messages")?,
        projected_messages: row.try_get("projected_messages")?,
        upserted_persons: row.try_get("upserted_persons")?,
        upserted_organizations: row.try_get("upserted_organizations")?,
        checkpoint_before: row.try_get("checkpoint_before")?,
        checkpoint_after: row.try_get("checkpoint_after")?,
        checkpoint_saved: row.try_get("checkpoint_saved")?,
        error_code: row.try_get("error_code")?,
        error_message: row.try_get("error_message")?,
        started_at: row.try_get("started_at")?,
        completed_at: row.try_get("completed_at")?,
        next_run_at: row.try_get("next_run_at")?,
    })
}

fn checkpoint_is_present(checkpoint: Option<&Value>) -> bool {
    checkpoint
        .and_then(Value::as_object)
        .is_some_and(|object| !object.is_empty())
}

fn validate_account_id(account_id: &str) -> Result<(), MailSyncError> {
    if account_id.trim().is_empty() {
        return Err(MailSyncError::InvalidSetting {
            field: "account_id",
            message: "must not be empty",
        });
    }
    Ok(())
}

fn validate_settings(batch_size: i32, poll_interval_seconds: i32) -> Result<(), MailSyncError> {
    if !(1..=MAX_BATCH_SIZE).contains(&batch_size) {
        return Err(MailSyncError::InvalidSetting {
            field: "batch_size",
            message: "must be between 1 and 500",
        });
    }
    if !(MIN_POLL_INTERVAL_SECONDS..=MAX_POLL_INTERVAL_SECONDS).contains(&poll_interval_seconds) {
        return Err(MailSyncError::InvalidSetting {
            field: "poll_interval_seconds",
            message: "must be between 60 and 86400",
        });
    }
    Ok(())
}

fn next_run_at(settings: &MailSyncSettings) -> Option<DateTime<Utc>> {
    if settings.sync_enabled {
        Some(Utc::now() + TimeDelta::seconds(i64::from(settings.poll_interval_seconds)))
    } else {
        None
    }
}

fn mail_sync_run_id(account_id: &str) -> String {
    format!(
        "mail-sync-run:v1:{}:{}",
        account_id.trim(),
        Utc::now().timestamp_micros()
    )
}
