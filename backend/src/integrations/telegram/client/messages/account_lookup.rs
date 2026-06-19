use crate::domains::mail::core::ProviderAccount;
use crate::vault::CommunicationProviderAccountStore;

use super::super::errors::TelegramError;
use super::super::store::TelegramStore;

impl TelegramStore {
    pub(in crate::integrations::telegram::client) async fn telegram_provider_account(
        &self,
        account_id: &str,
    ) -> Result<ProviderAccount, TelegramError> {
        let provider_account = CommunicationProviderAccountStore::new(self.pool.clone())
            .get(account_id)
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram account `{account_id}` is not configured"
                ))
            })?;
        if !provider_account.provider_kind.is_telegram() {
            return Err(TelegramError::InvalidRequest(format!(
                "account `{}` is not a Telegram provider account",
                provider_account.account_id
            )));
        }
        Ok(provider_account)
    }
}
