use std::path::PathBuf;

use chrono::Utc;
use serde_json::Value;
use sqlx::postgres::PgPool;

use crate::domains::communications::core::CommunicationIngestionStore;
use crate::integrations::mail::sync::plan_email_sync;
use crate::vault::{CommunicationProviderAccountStore, HostVault};

use super::DEFAULT_GMAIL_API_BASE_URL;
use super::errors::MailSyncError;
use super::models::{
    FinishRun, MailSyncPhase, MailSyncRunResponse, MailSyncRunStatus, MailSyncSettings,
    MailSyncTrigger, ProgressMode, SanitizedSyncFailure,
};
use super::provider::ProviderSyncContext;
use super::store::MailSyncStore;
use super::validation::{next_run_at, require_unlocked_vault};

#[derive(Clone)]
pub struct MailBackgroundSyncService {
    pub(super) pool: PgPool,
    pub(super) vault: HostVault,
    pub(super) blob_root: PathBuf,
    pub(super) gmail_api_base_url: String,
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
        let account = CommunicationProviderAccountStore::new(self.pool.clone())
            .get(account_id)
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

    pub async fn run_account_full_resync(
        &self,
        account_id: &str,
    ) -> Result<MailSyncRunResponse, MailSyncError> {
        let communication_store = CommunicationIngestionStore::new(self.pool.clone());
        let account = CommunicationProviderAccountStore::new(self.pool.clone())
            .get(account_id)
            .await?
            .ok_or(MailSyncError::AccountNotFound)?;
        if let Ok(plan) = plan_email_sync(&account) {
            communication_store
                .delete_checkpoint(account_id, &plan.stream_id)
                .await?;
        }

        self.run_account(account_id, MailSyncTrigger::Manual).await
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
}
