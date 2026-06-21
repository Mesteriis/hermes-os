use hermes_hub_backend::domains::communications::core::CommunicationProviderAccountStore;
use hermes_hub_backend::domains::communications::core::NewProviderAccount;
use serde_json::json;

use crate::config::DevEmailSyncConfig;
use crate::errors::DevEmailSyncError;

pub(super) async fn upsert_dev_provider_account(
    store: &CommunicationProviderAccountStore,
    config: &DevEmailSyncConfig,
) -> Result<(), DevEmailSyncError> {
    let account = NewProviderAccount::new(
        &config.account_id,
        config.provider_kind,
        &config.display_name,
        &config.external_account_id,
    )
    .config(json!({
        "host": config.host,
        "port": config.port,
        "tls": config.tls,
        "mailbox": config.mailbox
    }));

    store.upsert(&account).await?;

    Ok(())
}
