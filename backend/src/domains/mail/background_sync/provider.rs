use serde_json::Value;

use crate::domains::graph::projection::GraphProjectionService;
use crate::domains::mail::accounts::EmailAccountSetupService;
use crate::domains::mail::core::{
    CommunicationIngestionStore, ProviderAccount, ProviderAccountSecretPurpose,
    ProviderCredentialReader,
};
use crate::domains::mail::storage::LocalMailBlobStore;
use crate::domains::mail::sync::{EmailSyncAdapterConfig, EmailSyncBatch};
use crate::integrations::gmail::client::{
    GmailApiClient, GmailFetchOptions, GmailHistoryFetchOptions, ImapFetchOptions,
    ImapNetworkClient,
};
use crate::platform::secrets::SecretReferenceStore;
use crate::workflows::email_sync_pipeline::{
    EmailSyncPipelineReport, project_email_sync_batch_with_mail_blobs,
};

use super::errors::ProviderSyncError;
use super::models::{MailSyncPhase, MailSyncSettings, ProgressMode, ProgressUpdate};
use super::service::MailBackgroundSyncService;
use super::store::MailSyncStore;
use super::validation::gmail_history_expired;

impl MailBackgroundSyncService {
    pub(super) async fn execute_provider_sync(
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
        let checkpoint_next_page_token = context
            .checkpoint_before
            .as_ref()
            .and_then(|checkpoint| checkpoint.get("next_page_token"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        let checkpoint_page_kind = context
            .checkpoint_before
            .as_ref()
            .and_then(|checkpoint| checkpoint.get("page_kind"))
            .and_then(Value::as_str);

        if checkpoint_next_page_token.is_some() && checkpoint_page_kind != Some("history") {
            self.sync_gmail_message_list_pages(
                &client,
                &access_token,
                &context,
                &mut summary,
                checkpoint_next_page_token,
            )
            .await?;
            return Ok(summary);
        }

        if let Some(history_id) = context
            .checkpoint_before
            .as_ref()
            .and_then(|checkpoint| checkpoint.get("history_id"))
            .and_then(Value::as_str)
            .map(str::to_owned)
        {
            let start_history_id = context
                .checkpoint_before
                .as_ref()
                .and_then(|checkpoint| checkpoint.get("start_history_id"))
                .and_then(Value::as_str)
                .unwrap_or(&history_id)
                .to_owned();
            let history_page_token = if checkpoint_page_kind == Some("history") {
                checkpoint_next_page_token
            } else {
                None
            };
            let history_expired = self
                .sync_gmail_history_pages(
                    &client,
                    &access_token,
                    &context,
                    &mut summary,
                    &start_history_id,
                    history_page_token,
                )
                .await?;
            if !history_expired {
                return Ok(summary);
            }
        }

        self.sync_gmail_message_list_pages(&client, &access_token, &context, &mut summary, None)
            .await?;

        Ok(summary)
    }

    async fn sync_gmail_history_pages(
        &self,
        client: &GmailApiClient,
        access_token: &crate::platform::secrets::ResolvedSecret,
        context: &ProviderSyncContext<'_>,
        summary: &mut ProviderSyncSummary,
        start_history_id: &str,
        mut page_token: Option<String>,
    ) -> Result<bool, ProviderSyncError> {
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
            let mut options =
                GmailHistoryFetchOptions::new(start_history_id, context.settings.batch_size as u16);
            if let Some(token) = page_token {
                options = options.page_token(token);
            }
            let history_batch = client
                .fetch_history_raw_messages(access_token, &options)
                .await;
            let batch = match history_batch {
                Ok(batch) => batch,
                Err(error) if gmail_history_expired(&error) => {
                    context
                        .store
                        .mark_recoverable_full_resync(context.run_id, "gmail_history_expired")
                        .await?;
                    return Ok(true);
                }
                Err(error) => return Err(error.into()),
            };
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
                summary,
                &context.account.account_id,
                batch,
            )
            .await?;
            if page_token.is_none() || fetched_count == 0 {
                break;
            }
        }

        Ok(false)
    }

    async fn sync_gmail_message_list_pages(
        &self,
        client: &GmailApiClient,
        access_token: &crate::platform::secrets::ResolvedSecret,
        context: &ProviderSyncContext<'_>,
        summary: &mut ProviderSyncSummary,
        mut page_token: Option<String>,
    ) -> Result<(), ProviderSyncError> {
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
            let batch = client.fetch_raw_messages(access_token, &options).await?;
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
                summary,
                &context.account.account_id,
                batch,
            )
            .await?;
            if page_token.is_none() || fetched_count == 0 {
                break;
            }
        }

        Ok(())
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

pub(super) struct ProviderSyncContext<'a> {
    pub(super) store: &'a MailSyncStore,
    pub(super) communication_store: &'a CommunicationIngestionStore,
    pub(super) account: &'a ProviderAccount,
    pub(super) run_id: &'a str,
    pub(super) settings: &'a MailSyncSettings,
    pub(super) checkpoint_before: Option<Value>,
}

#[derive(Clone, Copy)]
struct ImapAccountConfig<'a> {
    host: &'a str,
    port: u16,
    tls: bool,
    mailbox: &'a str,
}

#[derive(Default)]
pub(super) struct ProviderSyncSummary {
    pub(super) processed_messages: i64,
    pub(super) estimated_total_messages: Option<i64>,
    pub(super) current_batch_size: i32,
    pub(super) fetched_messages: i64,
    pub(super) projected_messages: i64,
    pub(super) upserted_persons: i64,
    pub(super) upserted_organizations: i64,
    pub(super) checkpoint_after: Option<Value>,
    pub(super) checkpoint_saved: bool,
}

impl ProviderSyncSummary {
    fn apply_pipeline_report(&mut self, report: &EmailSyncPipelineReport) {
        self.projected_messages += report.projected_messages as i64;
        self.upserted_persons += report.upserted_person_identities as i64;
        self.upserted_organizations += report.upserted_organizations as i64;
    }
}
