use super::super::errors::MailSyncError;
use super::MailSyncStore;

impl MailSyncStore {
    pub(in crate::domains::mail::background_sync::store) async fn require_account(
        &self,
        account_id: &str,
    ) -> Result<(), MailSyncError> {
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
