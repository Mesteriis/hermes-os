use serde_json::json;

use crate::platform::communications::NewProviderAccountSecretBinding;
use crate::platform::secrets::{NewSecretReference, SecretReferenceStore};
use crate::vault::CommunicationProviderSecretBindingStore;

use super::super::errors::TelegramError;
use super::super::identifiers::telegram_secret_ref;
use super::super::models::TelegramCredentialBinding;
use super::super::store::TelegramStore;
use super::super::vault::{TelegramCredentialWrite, TelegramSecretVault};

impl TelegramStore {
    pub(in crate::integrations::telegram::client::accounts) async fn store_account_credential(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &TelegramSecretVault,
        credential: TelegramCredentialWrite<'_>,
    ) -> Result<TelegramCredentialBinding, TelegramError> {
        let secret_ref = telegram_secret_ref(credential.account_id, credential.secret_purpose);
        secret_store
            .upsert_secret_reference(
                &NewSecretReference::new(
                    &secret_ref,
                    credential.secret_kind,
                    vault.store_kind(),
                    format!("{} for {}", credential.label, credential.account_id),
                )
                .metadata(json!({
                    "provider": credential.provider_kind.as_str(),
                    "account_id": credential.account_id,
                    "secret_purpose": credential.secret_purpose.as_str()
                })),
            )
            .await?;
        vault.store_secret(&secret_ref, &credential).await?;
        CommunicationProviderSecretBindingStore::new(self.pool.clone())
            .bind(&NewProviderAccountSecretBinding::new(
                credential.account_id,
                credential.secret_purpose,
                &secret_ref,
            ))
            .await
            .map_err(|error| TelegramError::ProviderAccountStore(error.to_string()))?;

        Ok(TelegramCredentialBinding {
            secret_purpose: credential.secret_purpose.as_str().to_owned(),
            secret_ref,
            secret_kind: credential.secret_kind,
            store_kind: vault.store_kind(),
        })
    }
}
