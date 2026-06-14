mod gmail;
mod imap;
mod projection;
mod summary;
mod types;

use crate::domains::mail::sync::EmailSyncAdapterConfig;

use super::errors::ProviderSyncError;
use super::service::MailBackgroundSyncService;

pub(super) use self::summary::ProviderSyncSummary;
use self::types::ImapAccountConfig;
pub(super) use self::types::ProviderSyncContext;

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
}
