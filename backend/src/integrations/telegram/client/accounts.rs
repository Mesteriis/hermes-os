use chrono::Utc;
use serde_json::json;

use crate::domains::mail::core::{
    CommunicationIngestionStore, CommunicationProviderKind, NewProviderAccount,
    NewProviderAccountSecretBinding, ProviderAccountSecretPurpose,
};
use crate::platform::secrets::{NewSecretReference, SecretKind, SecretReferenceStore};

use super::errors::TelegramError;
use super::identifiers::{
    telegram_account_from_provider_account, telegram_account_lifecycle_state, telegram_secret_ref,
};
use super::models::{
    TelegramAccount, TelegramAccountSetupRequest, TelegramAccountSetupResponse,
    TelegramCredentialBinding, TelegramLiveAccountSetupRequest,
};
use super::store::TelegramStore;
use super::validation::{required_optional_value, validate_object};
use super::vault::{TelegramCredentialWrite, TelegramSecretVault};
use super::{TELEGRAM_ACCOUNT_LOGGED_OUT, TELEGRAM_ACCOUNT_REMOVED};

impl TelegramStore {
    pub async fn setup_fixture_account(
        &self,
        request: &TelegramAccountSetupRequest,
    ) -> Result<TelegramAccountSetupResponse, TelegramError> {
        request.validate()?;
        let provider_kind = request.provider_kind;
        if !provider_kind.is_telegram() {
            return Err(TelegramError::InvalidRequest(
                "provider_kind must be telegram_user or telegram_bot".to_owned(),
            ));
        }

        let account = NewProviderAccount::new(
            &request.account_id,
            provider_kind,
            &request.display_name,
            &request.external_account_id,
        )
        .config(json!({
            "runtime": "fixture",
            "tdlib_data_path": request.tdlib_data_path,
            "transcription_enabled": request.transcription_enabled,
        }));
        let stored_account = CommunicationIngestionStore::new(self.pool.clone())
            .upsert_provider_account(&account)
            .await?;

        Ok(TelegramAccountSetupResponse {
            account_id: stored_account.account_id,
            provider_kind: stored_account.provider_kind.as_str().to_owned(),
            runtime: "fixture".to_owned(),
            transcription_enabled: request.transcription_enabled,
            credential_bindings: vec![],
        })
    }

    pub async fn list_accounts(
        &self,
        include_removed: bool,
    ) -> Result<Vec<TelegramAccount>, TelegramError> {
        let accounts = CommunicationIngestionStore::new(self.pool.clone())
            .list_provider_accounts()
            .await?;

        Ok(accounts
            .into_iter()
            .filter(|account| account.provider_kind.is_telegram())
            .map(telegram_account_from_provider_account)
            .filter(|account| {
                include_removed || account.lifecycle_state != TELEGRAM_ACCOUNT_REMOVED
            })
            .collect())
    }

    pub async fn logout_account(&self, account_id: &str) -> Result<TelegramAccount, TelegramError> {
        self.update_account_lifecycle(account_id, TELEGRAM_ACCOUNT_LOGGED_OUT)
            .await
    }

    pub async fn remove_account(&self, account_id: &str) -> Result<TelegramAccount, TelegramError> {
        self.update_account_lifecycle(account_id, TELEGRAM_ACCOUNT_REMOVED)
            .await
    }

    pub async fn setup_live_blocked_account(
        &self,
        secret_store: &SecretReferenceStore,
        vault: &TelegramSecretVault,
        request: &TelegramLiveAccountSetupRequest,
    ) -> Result<TelegramAccountSetupResponse, TelegramError> {
        request.validate()?;
        let provider_kind = request.provider_kind;
        if !provider_kind.is_telegram() {
            return Err(TelegramError::InvalidRequest(
                "provider_kind must be telegram_user or telegram_bot".to_owned(),
            ));
        }

        let is_qr_authorized = request.is_qr_authorized_user_account();
        let runtime = if is_qr_authorized {
            "tdlib_qr_authorized"
        } else {
            "live_blocked"
        };
        let mut config = json!({
            "runtime": runtime,
            "transcription_enabled": request.transcription_enabled,
        });
        if let Some(object) = config.as_object_mut() {
            if let Some(tdlib_data_path) = request
                .tdlib_data_path
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                object.insert("tdlib_data_path".to_owned(), json!(tdlib_data_path));
            }
            if !is_qr_authorized && let Some(api_id) = request.api_id {
                object.insert("api_id".to_owned(), json!(api_id));
            }
        }

        let stored_account = CommunicationIngestionStore::new(self.pool.clone())
            .upsert_provider_account(
                &NewProviderAccount::new(
                    &request.account_id,
                    provider_kind,
                    &request.display_name,
                    &request.external_account_id,
                )
                .config(config),
            )
            .await?;

        let mut credential_bindings = Vec::new();
        match provider_kind {
            CommunicationProviderKind::TelegramUser => {
                if is_qr_authorized {
                    if let Some(session_encryption_key) = request
                        .session_encryption_key
                        .as_deref()
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                    {
                        credential_bindings.push(
                            self.store_account_credential(
                                secret_store,
                                vault,
                                TelegramCredentialWrite {
                                    account_id: &request.account_id,
                                    provider_kind,
                                    secret_purpose:
                                        ProviderAccountSecretPurpose::TelegramSessionKey,
                                    secret_kind: SecretKind::Other,
                                    label: "Telegram session encryption key",
                                    value: session_encryption_key.to_owned(),
                                },
                            )
                            .await?,
                        );
                    }
                } else {
                    credential_bindings.push(
                        self.store_account_credential(
                            secret_store,
                            vault,
                            TelegramCredentialWrite {
                                account_id: &request.account_id,
                                provider_kind,
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
                    if let Some(session_encryption_key) = request
                        .session_encryption_key
                        .as_deref()
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                    {
                        credential_bindings.push(
                            self.store_account_credential(
                                secret_store,
                                vault,
                                TelegramCredentialWrite {
                                    account_id: &request.account_id,
                                    provider_kind,
                                    secret_purpose:
                                        ProviderAccountSecretPurpose::TelegramSessionKey,
                                    secret_kind: SecretKind::Other,
                                    label: "Telegram session encryption key",
                                    value: session_encryption_key.to_owned(),
                                },
                            )
                            .await?,
                        );
                    }
                }
            }
            CommunicationProviderKind::TelegramBot => {
                credential_bindings.push(
                    self.store_account_credential(
                        secret_store,
                        vault,
                        TelegramCredentialWrite {
                            account_id: &request.account_id,
                            provider_kind,
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

        Ok(TelegramAccountSetupResponse {
            account_id: stored_account.account_id,
            provider_kind: stored_account.provider_kind.as_str().to_owned(),
            runtime: runtime.to_owned(),
            transcription_enabled: request.transcription_enabled,
            credential_bindings,
        })
    }

    async fn update_account_lifecycle(
        &self,
        account_id: &str,
        lifecycle_state: &'static str,
    ) -> Result<TelegramAccount, TelegramError> {
        let communication_store = CommunicationIngestionStore::new(self.pool.clone());
        let account = self
            .telegram_provider_account(&communication_store, account_id)
            .await?;
        let current_state = telegram_account_lifecycle_state(&account);
        if current_state == TELEGRAM_ACCOUNT_REMOVED && lifecycle_state != TELEGRAM_ACCOUNT_REMOVED
        {
            return Err(TelegramError::InvalidRequest(format!(
                "Telegram account `{}` is removed",
                account.account_id
            )));
        }

        let mut config = account.config.clone();
        validate_object("config", &config)?;
        let Some(config_object) = config.as_object_mut() else {
            return Err(TelegramError::InvalidRequest(
                "config must be a JSON object".to_owned(),
            ));
        };
        let now = Utc::now();
        config_object.insert("lifecycle_state".to_owned(), json!(lifecycle_state));
        config_object.insert("lifecycle_updated_at".to_owned(), json!(now));
        match lifecycle_state {
            TELEGRAM_ACCOUNT_LOGGED_OUT => {
                config_object.insert("logged_out_at".to_owned(), json!(now));
            }
            TELEGRAM_ACCOUNT_REMOVED => {
                config_object.insert("removed_at".to_owned(), json!(now));
            }
            _ => {}
        }

        let updated = communication_store
            .update_provider_account_config(&account.account_id, &config)
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram account `{}` is not configured",
                    account.account_id
                ))
            })?;

        Ok(telegram_account_from_provider_account(updated))
    }

    async fn store_account_credential(
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
        CommunicationIngestionStore::new(self.pool.clone())
            .bind_provider_account_secret(&NewProviderAccountSecretBinding::new(
                credential.account_id,
                credential.secret_purpose,
                &secret_ref,
            ))
            .await?;

        Ok(TelegramCredentialBinding {
            secret_purpose: credential.secret_purpose.as_str().to_owned(),
            secret_ref,
            secret_kind: credential.secret_kind,
            store_kind: vault.store_kind(),
        })
    }
}
