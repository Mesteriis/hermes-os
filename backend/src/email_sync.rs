use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD as BASE64_STANDARD, URL_SAFE, URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use thiserror::Error;

use crate::communications::{
    CommunicationIngestionError, CommunicationIngestionStore, EmailProviderKind,
    NewIngestionCheckpoint, NewRawCommunicationRecord, ProviderAccount,
    ProviderAccountSecretPurpose, StoredRawCommunicationRecord,
};
use crate::mail_storage::{LocalMailBlobStore, MailStorageError, MailStorageStore, NewMailBlob};

const EMAIL_MESSAGE_RECORD_KIND: &str = "email_message";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailSyncPlan {
    pub account_id: String,
    pub provider_kind: EmailProviderKind,
    pub credential_purpose: ProviderAccountSecretPurpose,
    pub stream_id: String,
    pub adapter_config: EmailSyncAdapterConfig,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmailSyncAdapterConfig {
    Gmail {
        history_stream_id: String,
    },
    Imap {
        host: String,
        port: u16,
        tls: bool,
        mailbox: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FetchedEmailMessage {
    pub provider_record_id: String,
    pub source_fingerprint: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub payload: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailSyncBatch {
    pub provider_kind: EmailProviderKind,
    pub stream_id: String,
    pub checkpoint: Option<Value>,
    pub messages: Vec<FetchedEmailMessage>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailSyncImportReport {
    pub inserted_or_existing_records: usize,
    pub checkpoint_saved: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailSyncBlobImportReport {
    pub inserted_or_existing_records: usize,
    pub checkpoint_saved: bool,
    pub blobs_upserted: usize,
    pub raw_records: Vec<StoredRawCommunicationRecord>,
}

pub fn plan_email_sync(account: &ProviderAccount) -> Result<EmailSyncPlan, EmailSyncPlanError> {
    let account_id = validate_non_empty("account_id", &account.account_id)?;
    reject_secret_like_config_keys(&account.config)?;

    match account.provider_kind {
        EmailProviderKind::Gmail => {
            let history_stream_id = optional_string(&account.config, "history_stream_id")?
                .unwrap_or_else(|| "gmail:history".to_owned());
            validate_non_empty("history_stream_id", &history_stream_id)?;
            validate_no_control_chars("history_stream_id", &history_stream_id)?;

            Ok(EmailSyncPlan {
                account_id,
                provider_kind: account.provider_kind,
                credential_purpose: ProviderAccountSecretPurpose::OauthToken,
                stream_id: history_stream_id.clone(),
                adapter_config: EmailSyncAdapterConfig::Gmail { history_stream_id },
            })
        }
        EmailProviderKind::Icloud | EmailProviderKind::Imap => {
            let host = required_string(&account.config, "host")?;
            let port = required_port(&account.config, "port")?;
            let tls = required_bool(&account.config, "tls")?;
            let mailbox =
                optional_string(&account.config, "mailbox")?.unwrap_or_else(|| "INBOX".to_owned());
            validate_non_empty("mailbox", &mailbox)?;
            validate_no_control_chars("mailbox", &mailbox)?;
            let stream_id = imap_mailbox_stream_id(&mailbox);

            Ok(EmailSyncPlan {
                account_id,
                provider_kind: account.provider_kind,
                credential_purpose: ProviderAccountSecretPurpose::ImapPassword,
                stream_id,
                adapter_config: EmailSyncAdapterConfig::Imap {
                    host,
                    port,
                    tls,
                    mailbox,
                },
            })
        }
        EmailProviderKind::TelegramUser
        | EmailProviderKind::TelegramBot
        | EmailProviderKind::WhatsappWeb => Err(EmailSyncPlanError::InvalidProviderConfig {
            field: "provider_kind",
            message: "email sync supports only gmail, icloud or imap",
        }),
    }
}

pub async fn record_email_sync_batch(
    store: &CommunicationIngestionStore,
    account_id: &str,
    import_batch_id: &str,
    batch: &EmailSyncBatch,
) -> Result<EmailSyncImportReport, EmailSyncRecordError> {
    let account_id = validate_non_empty("account_id", account_id)?;
    let import_batch_id = validate_non_empty("import_batch_id", import_batch_id)?;
    validate_non_empty("stream_id", &batch.stream_id)?;

    let mut inserted_or_existing_records = 0;
    for message in &batch.messages {
        let mut raw_record = NewRawCommunicationRecord::new(
            raw_record_id(
                &account_id,
                EMAIL_MESSAGE_RECORD_KIND,
                &message.provider_record_id,
            ),
            &account_id,
            EMAIL_MESSAGE_RECORD_KIND,
            &message.provider_record_id,
            &message.source_fingerprint,
            &import_batch_id,
            message.payload.clone(),
        )
        .provenance(json!({
            "source": "email_provider_sync",
            "provider": batch.provider_kind.as_str(),
            "stream_id": batch.stream_id
        }));

        if let Some(occurred_at) = message.occurred_at {
            raw_record = raw_record.occurred_at(occurred_at);
        }

        store.record_raw_source(&raw_record).await?;
        inserted_or_existing_records += 1;
    }

    let checkpoint_saved = if let Some(checkpoint) = &batch.checkpoint {
        store
            .save_checkpoint(&NewIngestionCheckpoint::new(
                &account_id,
                &batch.stream_id,
                checkpoint.clone(),
            ))
            .await?;
        true
    } else {
        false
    };

    Ok(EmailSyncImportReport {
        inserted_or_existing_records,
        checkpoint_saved,
    })
}

pub async fn record_email_sync_batch_with_mail_blobs(
    store: &CommunicationIngestionStore,
    mail_store: &MailStorageStore,
    blob_store: &LocalMailBlobStore,
    account_id: &str,
    import_batch_id: &str,
    batch: &EmailSyncBatch,
) -> Result<EmailSyncBlobImportReport, EmailSyncRecordError> {
    let account_id = validate_non_empty("account_id", account_id)?;
    let import_batch_id = validate_non_empty("import_batch_id", import_batch_id)?;
    validate_non_empty("stream_id", &batch.stream_id)?;

    let mut inserted_or_existing_records = 0;
    let mut blobs_upserted = 0;
    let mut raw_records = Vec::new();
    for message in &batch.messages {
        let raw_bytes = raw_message_bytes(batch.provider_kind, &message.payload)?;
        let local_blob = blob_store.put_blob(&raw_bytes).await?;
        let stored_blob = mail_store
            .upsert_blob(&NewMailBlob::from_local_blob(&local_blob).content_type("message/rfc822"))
            .await?;
        let payload = payload_with_raw_blob_reference(&message.payload, &stored_blob)?;

        let mut raw_record = NewRawCommunicationRecord::new(
            raw_record_id(
                &account_id,
                EMAIL_MESSAGE_RECORD_KIND,
                &message.provider_record_id,
            ),
            &account_id,
            EMAIL_MESSAGE_RECORD_KIND,
            &message.provider_record_id,
            &message.source_fingerprint,
            &import_batch_id,
            payload,
        )
        .provenance(json!({
            "source": "email_provider_sync",
            "provider": batch.provider_kind.as_str(),
            "stream_id": batch.stream_id,
            "raw_storage": stored_blob.storage_kind
        }));

        if let Some(occurred_at) = message.occurred_at {
            raw_record = raw_record.occurred_at(occurred_at);
        }

        raw_records.push(store.record_raw_source(&raw_record).await?);
        inserted_or_existing_records += 1;
        blobs_upserted += 1;
    }

    let checkpoint_saved = if let Some(checkpoint) = &batch.checkpoint {
        store
            .save_checkpoint(&NewIngestionCheckpoint::new(
                &account_id,
                &batch.stream_id,
                checkpoint.clone(),
            ))
            .await?;
        true
    } else {
        false
    };

    Ok(EmailSyncBlobImportReport {
        inserted_or_existing_records,
        checkpoint_saved,
        blobs_upserted,
        raw_records,
    })
}

pub fn imap_mailbox_stream_id(mailbox: &str) -> String {
    let mut stream_id = String::from("imap:");

    for character in mailbox.chars() {
        match character {
            '%' => stream_id.push_str("%25"),
            ':' => stream_id.push_str("%3A"),
            _ => stream_id.push(character),
        }
    }

    stream_id
}

fn required_string(config: &Value, field: &'static str) -> Result<String, EmailSyncPlanError> {
    let Some(value) = optional_string(config, field)? else {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "missing string value",
        });
    };
    validate_non_empty(field, &value)
}

fn optional_string(
    config: &Value,
    field: &'static str,
) -> Result<Option<String>, EmailSyncPlanError> {
    let Some(value) = config.get(field) else {
        return Ok(None);
    };
    let Some(value) = value.as_str() else {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "expected string value",
        });
    };

    Ok(Some(value.trim().to_owned()))
}

fn required_port(config: &Value, field: &'static str) -> Result<u16, EmailSyncPlanError> {
    let Some(value) = config.get(field).and_then(Value::as_u64) else {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "expected integer port",
        });
    };
    let Ok(port) = u16::try_from(value) else {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "port must fit u16",
        });
    };
    if port == 0 {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "port must be greater than zero",
        });
    }

    Ok(port)
}

fn required_bool(config: &Value, field: &'static str) -> Result<bool, EmailSyncPlanError> {
    config
        .get(field)
        .and_then(Value::as_bool)
        .ok_or(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "expected boolean value",
        })
}

fn reject_secret_like_config_keys(config: &Value) -> Result<(), EmailSyncPlanError> {
    let Some(object) = config.as_object() else {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field: "config",
            message: "expected object",
        });
    };

    for (key, value) in object {
        let key_path = key.clone();
        reject_secret_like_config_key(key, &key_path)?;
        reject_secret_like_config_value(value, &key_path)?;
    }

    Ok(())
}

fn reject_secret_like_config_value(value: &Value, path: &str) -> Result<(), EmailSyncPlanError> {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                let key_path = format!("{path}.{key}");
                reject_secret_like_config_key(key, &key_path)?;
                reject_secret_like_config_value(value, &key_path)?;
            }
            Ok(())
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                reject_secret_like_config_value(value, &format!("{path}[{index}]"))?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn reject_secret_like_config_key(key: &str, key_path: &str) -> Result<(), EmailSyncPlanError> {
    let normalized = key.to_ascii_lowercase();
    if normalized.contains("password")
        || normalized.contains("secret")
        || normalized.contains("token")
        || normalized.contains("credential")
    {
        return Err(EmailSyncPlanError::SecretLikeConfigKey {
            key: key_path.to_owned(),
        });
    }

    Ok(())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<String, EmailSyncPlanError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "must not be empty",
        });
    }

    Ok(value.to_owned())
}

fn validate_no_control_chars(field: &'static str, value: &str) -> Result<(), EmailSyncPlanError> {
    if value.chars().any(char::is_control) {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "must not contain control characters",
        });
    }

    Ok(())
}

fn raw_record_id(account_id: &str, record_kind: &str, provider_record_id: &str) -> String {
    let mut encoded = String::from("raw:v1:");
    append_raw_record_id_component(&mut encoded, account_id);
    encoded.push(':');
    append_raw_record_id_component(&mut encoded, record_kind);
    encoded.push(':');
    append_raw_record_id_component(&mut encoded, provider_record_id);
    encoded
}

fn append_raw_record_id_component(encoded: &mut String, value: &str) {
    encoded.push_str(&value.len().to_string());
    encoded.push(':');
    encoded.push_str(value);
}

fn raw_message_bytes(
    provider_kind: EmailProviderKind,
    payload: &Value,
) -> Result<Vec<u8>, EmailSyncRecordError> {
    match provider_kind {
        EmailProviderKind::Gmail => {
            let raw = required_payload_string(payload, "raw_base64url")?;
            URL_SAFE_NO_PAD
                .decode(raw)
                .or_else(|_| URL_SAFE.decode(raw))
                .map_err(|source| EmailSyncRecordError::InvalidRawPayloadBase64 {
                    field: "raw_base64url",
                    source,
                })
        }
        EmailProviderKind::Icloud | EmailProviderKind::Imap => {
            let raw = required_payload_string(payload, "raw_rfc822_base64")?;
            BASE64_STANDARD.decode(raw).map_err(|source| {
                EmailSyncRecordError::InvalidRawPayloadBase64 {
                    field: "raw_rfc822_base64",
                    source,
                }
            })
        }
        EmailProviderKind::TelegramUser
        | EmailProviderKind::TelegramBot
        | EmailProviderKind::WhatsappWeb => Err(EmailSyncRecordError::UnsupportedProviderKind(
            provider_kind.as_str().to_owned(),
        )),
    }
}

fn payload_with_raw_blob_reference(
    payload: &Value,
    blob: &crate::mail_storage::StoredMailBlob,
) -> Result<Value, EmailSyncRecordError> {
    let Some(object) = payload.as_object() else {
        return Err(EmailSyncRecordError::InvalidRawPayloadObject);
    };
    let mut object = object.clone();
    object.remove("raw_base64url");
    object.remove("raw_rfc822_base64");
    object.insert("raw_blob_id".to_owned(), json!(blob.blob_id));
    object.insert("raw_blob_sha256".to_owned(), json!(blob.sha256));
    object.insert("raw_blob_storage_kind".to_owned(), json!(blob.storage_kind));
    object.insert("raw_blob_storage_path".to_owned(), json!(blob.storage_path));
    object.insert("raw_blob_size_bytes".to_owned(), json!(blob.size_bytes));

    Ok(Value::Object(object))
}

fn required_payload_string<'a>(
    payload: &'a Value,
    field: &'static str,
) -> Result<&'a str, EmailSyncRecordError> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .ok_or(EmailSyncRecordError::MissingRawPayloadField { field })
}

#[derive(Debug, Error)]
pub enum EmailSyncPlanError {
    #[error("invalid provider config field {field}: {message}")]
    InvalidProviderConfig {
        field: &'static str,
        message: &'static str,
    },

    #[error("provider account config must not contain secret-like key: {key}")]
    SecretLikeConfigKey { key: String },
}

#[derive(Debug, Error)]
pub enum EmailSyncRecordError {
    #[error(transparent)]
    Plan(#[from] EmailSyncPlanError),

    #[error(transparent)]
    Communication(#[from] CommunicationIngestionError),

    #[error(transparent)]
    MailStorage(#[from] MailStorageError),

    #[error("email sync payload must be a JSON object before raw blob projection")]
    InvalidRawPayloadObject,

    #[error("email sync payload missing provider raw field: {field}")]
    MissingRawPayloadField { field: &'static str },

    #[error("email sync payload field {field} is invalid base64: {source}")]
    InvalidRawPayloadBase64 {
        field: &'static str,
        #[source]
        source: base64::DecodeError,
    },

    #[error("email sync does not support provider kind: {0}")]
    UnsupportedProviderKind(String),
}
