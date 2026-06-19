use crate::domains::mail::core::EmailProviderKind;
use crate::vault::CalendarAccountStore;

use super::errors::HostVaultReconciliationError;
use super::provider_recovery::RecoverableProviderSecret;

pub(super) async fn restore_linked_calendar_account(
    calendar_store: &CalendarAccountStore,
    secret: &RecoverableProviderSecret,
) -> Result<bool, HostVaultReconciliationError> {
    match secret.provider_kind {
        EmailProviderKind::Gmail => {
            let calendar_account_id = format!("google-calendar:{}", secret.account_id);
            if calendar_store.get(&calendar_account_id).await?.is_some() {
                return Ok(false);
            }
            calendar_store
                .restore_google_workspace_account(
                    &secret.account_id,
                    &secret.display_name,
                    Some(&secret.external_account_id),
                    &secret.secret_ref,
                )
                .await?;
            Ok(true)
        }
        EmailProviderKind::Icloud => {
            let calendar_account_id = format!("icloud-calendar:{}", secret.account_id);
            if calendar_store.get(&calendar_account_id).await?.is_some() {
                return Ok(false);
            }
            calendar_store
                .restore_apple_icloud_account(
                    &secret.account_id,
                    &secret.display_name,
                    Some(&secret.external_account_id),
                    &secret.secret_ref,
                )
                .await?;
            Ok(true)
        }
        _ => Ok(false),
    }
}
