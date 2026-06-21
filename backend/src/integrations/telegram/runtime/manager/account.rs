use crate::domains::communications::core::CommunicationProviderAccountStore;
use crate::integrations::telegram::client::{TelegramError, ensure_telegram_account_active};
use crate::platform::communications::ProviderAccount;

use super::super::status::load_telegram_account;

pub(in crate::integrations::telegram::runtime::manager) async fn load_active_account(
    provider_account_store: &CommunicationProviderAccountStore,
    account_id: &str,
) -> Result<ProviderAccount, TelegramError> {
    let account = load_telegram_account(provider_account_store, account_id).await?;
    ensure_telegram_account_active(&account)?;
    Ok(account)
}
