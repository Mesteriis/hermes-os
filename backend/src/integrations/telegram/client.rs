use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

use crate::domains::mail::core::{
    CommunicationIngestionError, CommunicationIngestionStore, CommunicationProviderKind,
    NewProviderAccount, NewProviderAccountSecretBinding, NewRawCommunicationRecord,
    ProviderAccountSecretPurpose,
};
use crate::domains::mail::messages::{
    MessageProjectionError, MessageProjectionStore, NewProjectedMessage,
};
use crate::platform::secrets::{
    DatabaseEncryptedSecretVault, DatabaseEncryptedVaultError, NewSecretReference, SecretKind,
    SecretReferenceError, SecretReferenceStore, SecretStoreKind,
};
use crate::vault::{HostVault, HostVaultError, SecretEntryContext};

const TELEGRAM_MESSAGE_RECORD_KIND: &str = "telegram_message";

struct TelegramCredentialWrite<'a> {
    account_id: &'a str,
    provider_kind: CommunicationProviderKind,
    secret_purpose: ProviderAccountSecretPurpose,
    secret_kind: SecretKind,
    label: &'a str,
    value: String,
}

pub enum TelegramSecretVault {
    Database(DatabaseEncryptedSecretVault),
    Host(HostVault),
}

impl TelegramSecretVault {
    pub fn database(vault: DatabaseEncryptedSecretVault) -> Self {
        Self::Database(vault)
    }

    pub fn host(vault: HostVault) -> Self {
        Self::Host(vault)
    }

    fn store_kind(&self) -> SecretStoreKind {
        match self {
            Self::Database(_) => SecretStoreKind::DatabaseEncryptedVault,
            Self::Host(_) => SecretStoreKind::HostVault,
        }
    }

    async fn store_secret(
        &self,
        secret_ref: &str,
        credential: &TelegramCredentialWrite<'_>,
    ) -> Result<(), TelegramError> {
        match self {
            Self::Database(vault) => vault.store_secret(secret_ref, &credential.value).await?,
            Self::Host(vault) => vault.store_secret(
                secret_ref,
                &credential.value,
                SecretEntryContext {
                    entry_kind: "provider_credential",
                    account_id: credential.account_id,
                    purpose: credential.secret_purpose.as_str(),
                    secret_kind: credential.secret_kind.as_str(),
                    label: credential.label,
                    metadata: &json!({
                        "provider": credential.provider_kind.as_str(),
                        "account_id": credential.account_id,
                        "secret_purpose": credential.secret_purpose.as_str()
                    }),
                },
            )?,
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct TelegramStore {
    pool: PgPool,
}

impl TelegramStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

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
            if !is_qr_authorized {
                if let Some(api_id) = request.api_id {
                    object.insert("api_id".to_owned(), json!(api_id));
                }
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
                if !is_qr_authorized {
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

    pub async fn upsert_chat(&self, chat: &NewTelegramChat) -> Result<TelegramChat, TelegramError> {
        chat.validate()?;
        let telegram_chat_id = telegram_chat_id(&chat.account_id, &chat.provider_chat_id);
        let row = sqlx::query(
            r#"
            INSERT INTO telegram_chats (
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())
            ON CONFLICT (account_id, provider_chat_id)
            DO UPDATE SET
                chat_kind = EXCLUDED.chat_kind,
                title = EXCLUDED.title,
                username = EXCLUDED.username,
                sync_state = EXCLUDED.sync_state,
                last_message_at = EXCLUDED.last_message_at,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                created_at,
                updated_at
            "#,
        )
        .bind(&telegram_chat_id)
        .bind(chat.account_id.trim())
        .bind(chat.provider_chat_id.trim())
        .bind(chat.chat_kind.as_str())
        .bind(chat.title.trim())
        .bind(
            chat.username
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(chat.sync_state.as_str())
        .bind(chat.last_message_at)
        .bind(&chat.metadata)
        .fetch_one(&self.pool)
        .await?;

        row_to_telegram_chat(row)
    }

    pub async fn list_chats(
        &self,
        account_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<TelegramChat>, TelegramError> {
        let limit = validate_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let rows = sqlx::query(
            r#"
            SELECT
                telegram_chat_id,
                account_id,
                provider_chat_id,
                chat_kind,
                title,
                username,
                sync_state,
                last_message_at,
                metadata,
                created_at,
                updated_at
            FROM telegram_chats
            WHERE ($1::text IS NULL OR account_id = $1)
            ORDER BY COALESCE(last_message_at, updated_at) DESC, telegram_chat_id ASC
            LIMIT $2
            "#,
        )
        .bind(account_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_telegram_chat).collect()
    }

    pub async fn ingest_fixture_message(
        &self,
        message: &NewTelegramMessage,
    ) -> Result<TelegramMessageIngestResult, TelegramError> {
        message.validate()?;
        let communication_store = CommunicationIngestionStore::new(self.pool.clone());
        let provider_account = communication_store
            .provider_account(&message.account_id)
            .await?
            .ok_or_else(|| {
                TelegramError::InvalidRequest(format!(
                    "Telegram account `{}` is not configured",
                    message.account_id
                ))
            })?;
        if !provider_account.provider_kind.is_telegram() {
            return Err(TelegramError::InvalidRequest(format!(
                "account `{}` is not a Telegram provider account",
                message.account_id
            )));
        }

        let chat = NewTelegramChat {
            account_id: message.account_id.clone(),
            provider_chat_id: message.provider_chat_id.clone(),
            chat_kind: message.chat_kind,
            title: message.chat_title.clone(),
            username: None,
            sync_state: TelegramSyncState::Synced,
            last_message_at: Some(message.occurred_at),
            metadata: json!({"runtime": "fixture"}),
        };
        self.upsert_chat(&chat).await?;

        let raw_record_id = telegram_raw_record_id(
            &message.account_id,
            TELEGRAM_MESSAGE_RECORD_KIND,
            &message.provider_message_id,
        );
        let raw = NewRawCommunicationRecord::new(
            &raw_record_id,
            &message.account_id,
            TELEGRAM_MESSAGE_RECORD_KIND,
            &message.provider_message_id,
            message.source_fingerprint(),
            &message.import_batch_id,
            json!({
                "provider_chat_id": message.provider_chat_id,
                "chat_title": message.chat_title,
                "chat_kind": message.chat_kind.as_str(),
                "sender_id": message.sender_id,
                "sender_display_name": message.sender_display_name,
                "text": message.text,
                "delivery_state": message.delivery_state.as_str(),
            }),
        )
        .occurred_at(message.occurred_at)
        .provenance(json!({
            "provider": "telegram",
            "provider_kind": provider_account.provider_kind.as_str(),
            "runtime": "fixture",
            "account_id": message.account_id,
            "provider_chat_id": message.provider_chat_id,
        }));
        let raw = communication_store.record_raw_source(&raw).await?;
        let projected =
            project_raw_telegram_message(&MessageProjectionStore::new(self.pool.clone()), &raw)
                .await?;

        Ok(TelegramMessageIngestResult {
            raw_record_id: raw.raw_record_id,
            message_id: projected.message_id,
        })
    }

    pub async fn recent_messages(
        &self,
        account_id: Option<&str>,
        provider_chat_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<TelegramMessage>, TelegramError> {
        let limit = validate_limit(limit)?;
        let account_id = account_id.map(str::trim).filter(|value| !value.is_empty());
        let provider_chat_id = provider_chat_id
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                raw_record_id,
                account_id,
                provider_record_id,
                subject,
                sender,
                body_text,
                occurred_at,
                projected_at,
                channel_kind,
                conversation_id,
                sender_display_name,
                delivery_state,
                message_metadata
            FROM communication_messages
            WHERE channel_kind IN ('telegram_user', 'telegram_bot')
              AND ($1::text IS NULL OR account_id = $1)
              AND ($2::text IS NULL OR conversation_id = $2)
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id ASC
            LIMIT $3
            "#,
        )
        .bind(account_id)
        .bind(provider_chat_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_telegram_message).collect()
    }
}

pub async fn project_raw_telegram_message(
    store: &MessageProjectionStore,
    raw: &crate::domains::mail::core::StoredRawCommunicationRecord,
) -> Result<crate::domains::mail::messages::ProjectedMessage, TelegramError> {
    if raw.record_kind != TELEGRAM_MESSAGE_RECORD_KIND {
        return Err(TelegramError::InvalidRequest(
            "raw record kind must be telegram_message".to_owned(),
        ));
    }

    let provider_chat_id = required_payload_string(&raw.payload, "provider_chat_id")?;
    let chat_title = required_payload_string(&raw.payload, "chat_title")?;
    let sender_display_name = required_payload_string(&raw.payload, "sender_display_name")?;
    let text = required_payload_string(&raw.payload, "text")?;
    let delivery_state =
        TelegramDeliveryState::try_from(required_payload_string(&raw.payload, "delivery_state")?)?;
    let channel_kind = raw
        .provenance
        .get("provider_kind")
        .and_then(Value::as_str)
        .unwrap_or("telegram_user");

    Ok(store
        .upsert_channel_message(&NewProjectedMessage {
            message_id: telegram_message_id(&raw.account_id, &raw.provider_record_id),
            raw_record_id: raw.raw_record_id.clone(),
            account_id: raw.account_id.clone(),
            provider_record_id: raw.provider_record_id.clone(),
            subject: chat_title,
            sender: sender_display_name.clone(),
            recipients: vec![provider_chat_id.clone()],
            body_text: text,
            occurred_at: raw.occurred_at,
            channel_kind: channel_kind.to_owned(),
            conversation_id: Some(provider_chat_id),
            sender_display_name: Some(sender_display_name),
            delivery_state: delivery_state.as_message_delivery_state().to_owned(),
            message_metadata: raw.payload.clone(),
        })
        .await?)
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramAccountSetupRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub tdlib_data_path: Option<String>,
    #[serde(default)]
    pub transcription_enabled: bool,
}

impl TelegramAccountSetupRequest {
    fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramLiveAccountSetupRequest {
    pub account_id: String,
    pub provider_kind: CommunicationProviderKind,
    pub display_name: String,
    pub external_account_id: String,
    pub api_id: Option<i64>,
    pub api_hash: Option<String>,
    pub bot_token: Option<String>,
    pub session_encryption_key: Option<String>,
    pub tdlib_data_path: Option<String>,
    #[serde(default)]
    pub qr_authorized: bool,
    #[serde(default)]
    pub transcription_enabled: bool,
}

impl TelegramLiveAccountSetupRequest {
    pub(crate) fn with_inferred_qr_authorization(mut self) -> Self {
        if self.is_finalized_qr_user_account() {
            self.qr_authorized = true;
        }
        self
    }

    pub(crate) fn with_app_credentials(
        mut self,
        api_id: Option<i64>,
        api_hash: Option<String>,
    ) -> Self {
        if self.is_qr_authorized_user_account() {
            return self;
        }
        if self.api_id.is_none() {
            self.api_id = api_id;
        }
        if self
            .api_hash
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_none()
        {
            self.api_hash = api_hash;
        }
        self
    }

    fn is_qr_authorized_user_account(&self) -> bool {
        self.qr_authorized && self.provider_kind == CommunicationProviderKind::TelegramUser
    }

    fn is_finalized_qr_user_account(&self) -> bool {
        self.provider_kind == CommunicationProviderKind::TelegramUser
            && self
                .external_account_id
                .trim()
                .strip_prefix("telegram:")
                .is_some_and(|provider_user_id| !provider_user_id.trim().is_empty())
            && self
                .tdlib_data_path
                .as_deref()
                .map(str::trim)
                .is_some_and(|value| !value.is_empty())
    }

    fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        match self.provider_kind {
            CommunicationProviderKind::TelegramUser => {
                if self.is_qr_authorized_user_account() {
                    required_optional_value("tdlib_data_path", self.tdlib_data_path.as_deref())?;
                    return Ok(());
                }
                let api_id = self.api_id.ok_or_else(|| {
                    TelegramError::InvalidRequest("api_id must not be empty".to_owned())
                })?;
                if api_id <= 0 {
                    return Err(TelegramError::InvalidRequest(
                        "api_id must be greater than zero".to_owned(),
                    ));
                }
                required_optional_value("api_hash", self.api_hash.as_deref())?;
            }
            CommunicationProviderKind::TelegramBot => {
                if self.qr_authorized {
                    return Err(TelegramError::InvalidRequest(
                        "qr_authorized is only supported for telegram_user".to_owned(),
                    ));
                }
                required_optional_value("bot_token", self.bot_token.as_deref())?;
            }
            _ => {
                return Err(TelegramError::InvalidRequest(
                    "provider_kind must be telegram_user or telegram_bot".to_owned(),
                ));
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramQrLoginStartRequest {
    pub account_id: String,
    pub display_name: String,
    pub external_account_id: String,
    pub api_id: Option<i64>,
    pub api_hash: Option<String>,
    pub session_encryption_key: Option<String>,
    pub tdlib_data_path: Option<String>,
    #[serde(default)]
    pub transcription_enabled: bool,
}

impl TelegramQrLoginStartRequest {
    pub(crate) fn with_app_credentials(
        mut self,
        api_id: Option<i64>,
        api_hash: Option<String>,
    ) -> Self {
        if self.api_id.is_none() {
            self.api_id = api_id;
        }
        if self
            .api_hash
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_none()
        {
            self.api_hash = api_hash;
        }
        self
    }

    pub(crate) fn required_api_id(&self) -> Result<i64, TelegramError> {
        let api_id = self
            .api_id
            .ok_or_else(|| TelegramError::InvalidRequest("api_id must not be empty".to_owned()))?;
        if api_id <= 0 {
            return Err(TelegramError::InvalidRequest(
                "api_id must be greater than zero".to_owned(),
            ));
        }
        Ok(api_id)
    }

    pub(crate) fn required_api_hash(&self) -> Result<&str, TelegramError> {
        self.api_hash
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| TelegramError::InvalidRequest("api_hash must not be empty".to_owned()))
    }

    pub(crate) fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        self.required_api_id()?;
        self.required_api_hash()?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TelegramQrLoginPasswordRequest {
    pub password: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramQrLoginStatus {
    WaitingQrScan,
    WaitingPassword,
    Ready,
    Expired,
    Failed,
    RuntimeUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramQrLoginStatusResponse {
    pub setup_id: String,
    pub account_id: String,
    pub status: TelegramQrLoginStatus,
    pub qr_link: Option<String>,
    pub qr_svg: Option<String>,
    pub telegram_user_id: Option<String>,
    pub telegram_username: Option<String>,
    pub suggested_account_id: Option<String>,
    pub suggested_display_name: Option<String>,
    pub suggested_external_account_id: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub poll_after_ms: u64,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramAccountSetupResponse {
    pub account_id: String,
    pub provider_kind: String,
    pub runtime: String,
    pub transcription_enabled: bool,
    pub credential_bindings: Vec<TelegramCredentialBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramCredentialBinding {
    pub secret_purpose: String,
    pub secret_ref: String,
    pub secret_kind: SecretKind,
    pub store_kind: SecretStoreKind,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramChat {
    pub telegram_chat_id: String,
    pub account_id: String,
    pub provider_chat_id: String,
    pub chat_kind: String,
    pub title: String,
    pub username: Option<String>,
    pub sync_state: String,
    pub last_message_at: Option<DateTime<Utc>>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewTelegramChat {
    pub account_id: String,
    pub provider_chat_id: String,
    pub chat_kind: TelegramChatKind,
    pub title: String,
    pub username: Option<String>,
    pub sync_state: TelegramSyncState,
    pub last_message_at: Option<DateTime<Utc>>,
    pub metadata: Value,
}

impl NewTelegramChat {
    fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("title", &self.title)?;
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramChatKind {
    Private,
    Group,
    Channel,
    Bot,
}

impl TelegramChatKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Private => "private",
            Self::Group => "group",
            Self::Channel => "channel",
            Self::Bot => "bot",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelegramSyncState {
    Fixture,
    Syncing,
    Synced,
    Degraded,
    Error,
}

impl TelegramSyncState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::Syncing => "syncing",
            Self::Synced => "synced",
            Self::Degraded => "degraded",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct NewTelegramMessage {
    pub account_id: String,
    pub provider_chat_id: String,
    pub provider_message_id: String,
    pub chat_kind: TelegramChatKind,
    pub chat_title: String,
    pub sender_id: String,
    pub sender_display_name: String,
    pub text: String,
    pub import_batch_id: String,
    pub occurred_at: DateTime<Utc>,
    pub delivery_state: TelegramDeliveryState,
}

impl NewTelegramMessage {
    fn validate(&self) -> Result<(), TelegramError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("provider_chat_id", &self.provider_chat_id)?;
        validate_non_empty("provider_message_id", &self.provider_message_id)?;
        validate_non_empty("chat_title", &self.chat_title)?;
        validate_non_empty("sender_id", &self.sender_id)?;
        validate_non_empty("sender_display_name", &self.sender_display_name)?;
        validate_non_empty("text", &self.text)?;
        validate_non_empty("import_batch_id", &self.import_batch_id)?;
        Ok(())
    }

    fn source_fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.account_id.as_bytes());
        hasher.update(b"\0");
        hasher.update(self.provider_chat_id.as_bytes());
        hasher.update(b"\0");
        hasher.update(self.provider_message_id.as_bytes());
        format!("sha256:{:x}", hasher.finalize())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramDeliveryState {
    Received,
    Sent,
    SendDryRun,
    SendBlocked,
}

impl TelegramDeliveryState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Received => "received",
            Self::Sent => "sent",
            Self::SendDryRun => "send_dry_run",
            Self::SendBlocked => "send_blocked",
        }
    }

    pub fn as_message_delivery_state(self) -> &'static str {
        self.as_str()
    }
}

impl TryFrom<String> for TelegramDeliveryState {
    type Error = TelegramError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "received" => Ok(Self::Received),
            "sent" => Ok(Self::Sent),
            "send_dry_run" => Ok(Self::SendDryRun),
            "send_blocked" => Ok(Self::SendBlocked),
            _ => Err(TelegramError::InvalidRequest(format!(
                "unsupported Telegram delivery_state `{value}`"
            ))),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramMessageIngestResult {
    pub raw_record_id: String,
    pub message_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TelegramMessage {
    pub message_id: String,
    pub raw_record_id: String,
    pub account_id: String,
    pub provider_message_id: String,
    pub provider_chat_id: Option<String>,
    pub chat_title: String,
    pub sender: String,
    pub sender_display_name: Option<String>,
    pub text: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub projected_at: DateTime<Utc>,
    pub channel_kind: String,
    pub delivery_state: String,
    pub metadata: Value,
}

#[derive(Debug, Error)]
pub enum TelegramError {
    #[error("invalid Telegram request: {0}")]
    InvalidRequest(String),

    #[error("Telegram TDLib runtime is not available: {0}")]
    TdlibRuntimeUnavailable(String),

    #[error("Telegram TDLib runtime failed: {0}")]
    TdlibRuntime(String),

    #[error("Telegram QR generation failed: {0}")]
    QrGeneration(String),

    #[error("Telegram QR login setup was not found")]
    QrLoginNotFound,

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    SecretReference(#[from] SecretReferenceError),

    #[error(transparent)]
    DatabaseVault(#[from] DatabaseEncryptedVaultError),

    #[error(transparent)]
    HostVault(#[from] HostVaultError),

    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

fn row_to_telegram_chat(row: PgRow) -> Result<TelegramChat, TelegramError> {
    Ok(TelegramChat {
        telegram_chat_id: row.try_get("telegram_chat_id")?,
        account_id: row.try_get("account_id")?,
        provider_chat_id: row.try_get("provider_chat_id")?,
        chat_kind: row.try_get("chat_kind")?,
        title: row.try_get("title")?,
        username: row.try_get("username")?,
        sync_state: row.try_get("sync_state")?,
        last_message_at: row.try_get("last_message_at")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_telegram_message(row: PgRow) -> Result<TelegramMessage, TelegramError> {
    Ok(TelegramMessage {
        message_id: row.try_get("message_id")?,
        raw_record_id: row.try_get("raw_record_id")?,
        account_id: row.try_get("account_id")?,
        provider_message_id: row.try_get("provider_record_id")?,
        provider_chat_id: row.try_get("conversation_id")?,
        chat_title: row.try_get("subject")?,
        sender: row.try_get("sender")?,
        sender_display_name: row.try_get("sender_display_name")?,
        text: row.try_get("body_text")?,
        occurred_at: row.try_get("occurred_at")?,
        projected_at: row.try_get("projected_at")?,
        channel_kind: row.try_get("channel_kind")?,
        delivery_state: row.try_get("delivery_state")?,
        metadata: row.try_get("message_metadata")?,
    })
}

fn required_payload_string(payload: &Value, field: &'static str) -> Result<String, TelegramError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            TelegramError::InvalidRequest(format!("payload field `{field}` is required"))
        })
}

fn telegram_chat_id(account_id: &str, provider_chat_id: &str) -> String {
    format!(
        "telegram_chat:v4:{}",
        stable_hash([account_id, provider_chat_id].join("\0").as_bytes())
    )
}

fn telegram_message_id(account_id: &str, provider_message_id: &str) -> String {
    format!(
        "message:v4:telegram:{}",
        stable_hash([account_id, provider_message_id].join("\0").as_bytes())
    )
}

fn telegram_raw_record_id(account_id: &str, record_kind: &str, provider_record_id: &str) -> String {
    format!(
        "raw:v4:telegram:{}",
        stable_hash(
            [account_id, record_kind, provider_record_id]
                .join("\0")
                .as_bytes()
        )
    )
}

fn stable_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn validate_limit(limit: i64) -> Result<i64, TelegramError> {
    if !(1..=100).contains(&limit) {
        return Err(TelegramError::InvalidRequest(
            "limit must be between 1 and 100".to_owned(),
        ));
    }
    Ok(limit)
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<String, TelegramError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(TelegramError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

fn required_optional_value(
    field: &'static str,
    value: Option<&str>,
) -> Result<String, TelegramError> {
    let Some(value) = value else {
        return Err(TelegramError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    };

    validate_non_empty(field, value)
}

fn validate_object(field: &'static str, value: &Value) -> Result<(), TelegramError> {
    if !value.is_object() {
        return Err(TelegramError::InvalidRequest(format!(
            "{field} must be a JSON object"
        )));
    }
    Ok(())
}

fn telegram_secret_ref(account_id: &str, secret_purpose: ProviderAccountSecretPurpose) -> String {
    format!(
        "secret:provider-account:{}:{}",
        account_id.trim(),
        secret_purpose.as_str()
    )
}
