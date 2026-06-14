use serde_json::Value;

use crate::domains::mail::core::{ProviderAccountSecretPurpose, ProviderCredentialReader};
use crate::integrations::gmail::client::{ImapFetchOptions, ImapNetworkClient};
use crate::platform::secrets::SecretReferenceStore;

use super::super::errors::ProviderSyncError;
use super::super::models::{MailSyncPhase, ProgressMode, ProgressUpdate};
use super::super::service::MailBackgroundSyncService;
use super::types::ImapAccountConfig;
use super::{ProviderSyncContext, ProviderSyncSummary};

impl MailBackgroundSyncService {
    pub(in crate::domains::mail::background_sync::provider) async fn sync_imap(
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
}
