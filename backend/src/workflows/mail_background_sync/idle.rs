use std::time::Duration;

use crate::domains::communications::core::CommunicationProviderAccountPort;
use crate::platform::communications::{
    EmailSyncAdapterConfig, ImapIdleWaitOutcome, ImapIdleWaitRequest, plan_email_sync,
};

use super::errors::MailSyncError;
use super::service::MailBackgroundSyncService;
use super::store::MailSyncStatePort;

const DEFAULT_IDLE_MAILBOX: &str = "INBOX";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailImapIdleOutcome {
    Changed,
    TimedOut,
    Unsupported,
    Disabled,
}

impl MailBackgroundSyncService {
    pub async fn wait_for_imap_change(
        &self,
        account_id: &str,
        timeout: Duration,
    ) -> Result<MailImapIdleOutcome, MailSyncError> {
        let account = CommunicationProviderAccountPort::new(self.pool.clone())
            .get(account_id)
            .await?
            .ok_or(MailSyncError::AccountNotFound)?;
        let settings = MailSyncStatePort::new(self.pool.clone())
            .settings_for_account(account_id)
            .await?;
        if !settings.sync_enabled {
            return Ok(MailImapIdleOutcome::Disabled);
        }

        let plan = plan_email_sync(&account)?;
        let EmailSyncAdapterConfig::Imap {
            host,
            port,
            tls,
            mailboxes,
        } = plan.adapter_config
        else {
            return Ok(MailImapIdleOutcome::Unsupported);
        };
        let mailbox = idle_mailbox(&mailboxes).to_owned();
        let outcome = self
            .provider_sync
            .wait_for_imap_change(ImapIdleWaitRequest {
                account_id: account.account_id,
                host,
                port,
                tls,
                mailbox,
                username: account.external_account_id,
                timeout,
            })
            .await?;
        Ok(match outcome {
            ImapIdleWaitOutcome::Changed => MailImapIdleOutcome::Changed,
            ImapIdleWaitOutcome::TimedOut => MailImapIdleOutcome::TimedOut,
            ImapIdleWaitOutcome::Unsupported => MailImapIdleOutcome::Unsupported,
        })
    }
}

fn idle_mailbox(mailboxes: &[String]) -> &str {
    mailboxes
        .iter()
        .find(|mailbox| mailbox.as_str() != crate::platform::communications::IMAP_ALL_MAILBOXES)
        .map(String::as_str)
        .unwrap_or(DEFAULT_IDLE_MAILBOX)
}

#[cfg(test)]
mod tests {
    use super::idle_mailbox;

    #[test]
    fn all_mailboxes_idle_uses_inbox_while_polling_still_covers_all_folders() {
        assert_eq!(idle_mailbox(&["*".to_owned()]), "INBOX");
        assert_eq!(idle_mailbox(&["Primary".to_owned()]), "Primary");
    }
}
