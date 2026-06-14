use crate::domains::mail::core::{CommunicationIngestionStore, ProviderAccount};
use crate::integrations::telegram::client::{TelegramError, ensure_telegram_account_active};

use super::super::status::load_telegram_account;

pub(in crate::integrations::telegram::runtime::manager) async fn load_active_account(
    communication_store: &CommunicationIngestionStore,
    account_id: &str,
) -> Result<ProviderAccount, TelegramError> {
    let account = load_telegram_account(communication_store, account_id).await?;
    ensure_telegram_account_active(&account)?;
    Ok(account)
}
