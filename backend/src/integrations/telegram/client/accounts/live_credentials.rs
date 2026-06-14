use crate::domains::mail::core::{CommunicationProviderKind, ProviderAccountSecretPurpose};
use crate::platform::secrets::{SecretKind, SecretReferenceStore};

use super::super::errors::TelegramError;
use super::super::models::{
    TelegramAccountSetupResponse, TelegramCredentialBinding, TelegramLiveAccountSetupRequest,
};
use super::super::store::TelegramStore;
use super::super::validation::required_optional_value;
use super::super::vault::{TelegramCredentialWrite, TelegramSecretVault};

impl TelegramStore {
    pub(in crate::integrations::telegram::client::accounts) async fn store_live_account_credentials(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &TelegramSecretVault,
        request: &TelegramLiveAccountSetupRequest,
    ) -> Result<Vec<TelegramCredentialBinding>, TelegramError> {
        let mut credential_bindings = Vec::new();
        match request.provider_kind {
            CommunicationProviderKind::TelegramUser => {
                if !request.is_qr_authorized_user_account() {
                    credential_bindings.push(
                        self.store_account_credential(
                            secret_store,
                            vault,
                            TelegramCredentialWrite {
                                account_id: &request.account_id,
                                provider_kind: request.provider_kind,
                                secret_purpose: ProviderAccountSecretPurpose::TelegramApiHash,
                                secret_kind: SecretKind::ApiToken,
                                label: "Telegram API hash",
                                value: required_optional_value(
                                    "api_hash",
                                    request.api_hash.as_deref(),
                                )?,
                            },
                        )
                        .await?,
                    );
                }
                if let Some(binding) = self.store_session_key(secret_store, vault, request).await? {
                    credential_bindings.push(binding);
                }
            }
            CommunicationProviderKind::TelegramBot => {
                credential_bindings.push(
                    self.store_account_credential(
                        secret_store,
                        vault,
                        TelegramCredentialWrite {
                            account_id: &request.account_id,
                            provider_kind: request.provider_kind,
                            secret_purpose: ProviderAccountSecretPurpose::TelegramBotToken,
                            secret_kind: SecretKind::ApiToken,
                            label: "Telegram bot token",
                            value: required_optional_value(
                                "bot_token",
                                request.bot_token.as_deref(),
                            )?,
                        },
                    )
                    .await?,
                );
            }
            _ => unreachable!("validated provider kind must be Telegram"),
        }

        Ok(credential_bindings)
    }

    async fn store_session_key(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &TelegramSecretVault,
        request: &TelegramLiveAccountSetupRequest,
    ) -> Result<Option<TelegramCredentialBinding>, TelegramError> {
        let Some(session_encryption_key) = request
            .session_encryption_key
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Ok(None);
        };

        self.store_account_credential(
            secret_store,
            vault,
            TelegramCredentialWrite {
                account_id: &request.account_id,
                provider_kind: request.provider_kind,
                secret_purpose: ProviderAccountSecretPurpose::TelegramSessionKey,
                secret_kind: SecretKind::Other,
                label: "Telegram session encryption key",
                value: session_encryption_key.to_owned(),
            },
        )
        .await
        .map(Some)
    }
}
