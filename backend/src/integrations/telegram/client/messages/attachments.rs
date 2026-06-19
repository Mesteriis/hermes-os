use chrono::Utc;
use serde_json::json;
use sqlx::{Postgres, Row, Transaction};

use super::super::errors::TelegramError;
use super::super::models::TelegramAttachmentAnchor;
use super::super::rows::row_to_telegram_message;
use super::super::store::TelegramStore;
use super::provider_state::capture_message_projection_observation_in_transaction;

impl TelegramStore {
    pub(crate) async fn attachment_anchor_for_message(
        &self,
        account_id: &str,
        provider_chat_id: &str,
        provider_message_id: &str,
    ) -> Result<TelegramAttachmentAnchor, TelegramError> {
        let row = sqlx::query(
            r#"
            SELECT message_id, raw_record_id
            FROM communication_messages
            WHERE account_id = $1
              AND conversation_id = $2
              AND provider_record_id = $3
              AND channel_kind IN ('telegram_user', 'telegram_bot')
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id ASC
            LIMIT 1
            "#,
        )
        .bind(account_id.trim())
        .bind(provider_chat_id.trim())
        .bind(provider_message_id.trim())
        .fetch_optional(&self.pool)
        .await?;

        let row = row.ok_or_else(|| {
            TelegramError::InvalidRequest(format!(
                "Telegram message `{}` is not projected for chat `{}` and account `{}`",
                provider_message_id.trim(),
                provider_chat_id.trim(),
                account_id.trim()
            ))
        })?;

        Ok(TelegramAttachmentAnchor {
            message_id: row.try_get("message_id")?,
            raw_record_id: row.try_get("raw_record_id")?,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn update_message_attachment_download_state(
        &self,
        message_id: &str,
        provider_attachment_id: &str,
        tdlib_file_id: i64,
        download_state: &str,
        local_path: Option<&str>,
        size_bytes: Option<i64>,
        content_type: &str,
        filename: Option<&str>,
    ) -> Result<(), TelegramError> {
        let current = self.message_by_id(message_id).await?.ok_or_else(|| {
            TelegramError::InvalidRequest(format!("Telegram message `{message_id}` was not found"))
        })?;

        let metadata = current.metadata.clone();

        let mut metadata_object = metadata.as_object().cloned().unwrap_or_default();
        let attachments = metadata_object
            .entry("attachments".to_owned())
            .or_insert_with(|| serde_json::Value::Array(Vec::new()));
        let attachment_array = attachments.as_array_mut().ok_or_else(|| {
            TelegramError::InvalidRequest(
                "telegram attachment metadata must be an array".to_owned(),
            )
        })?;

        let mut updated = false;
        for attachment in attachment_array.iter_mut() {
            let Some(object) = attachment.as_object_mut() else {
                continue;
            };
            let attachment_id_matches = object
                .get("attachment_id")
                .and_then(serde_json::Value::as_str)
                .map(|value| value == provider_attachment_id)
                .unwrap_or(false);
            let tdlib_id_matches = object
                .get("tdlib_file_id")
                .and_then(serde_json::Value::as_i64)
                .map(|value| value == tdlib_file_id)
                .unwrap_or(false);
            if !attachment_id_matches && !tdlib_id_matches {
                continue;
            }

            object.insert(
                "attachment_id".to_owned(),
                json!(provider_attachment_id.to_owned()),
            );
            object.insert("tdlib_file_id".to_owned(), json!(tdlib_file_id));
            object.insert("download_state".to_owned(), json!(download_state));
            object.insert("content_type".to_owned(), json!(content_type));
            if let Some(path) = local_path {
                object.insert("local_path".to_owned(), json!(path));
            }
            if let Some(size) = size_bytes {
                object.insert("size".to_owned(), json!(size));
            }
            if let Some(name) = filename {
                object.insert("filename".to_owned(), json!(name));
            }
            updated = true;
        }

        if !updated {
            attachment_array.push(json!({
                "attachment_id": provider_attachment_id,
                "attachment_type": "file",
                "content_type": content_type,
                "tdlib_file_id": tdlib_file_id,
                "download_state": download_state,
                "local_path": local_path,
                "size": size_bytes,
                "filename": filename,
            }));
        }

        let updated_metadata = serde_json::Value::Object(metadata_object);
        let observed_at = Utc::now();
        let mut transaction: Transaction<'_, Postgres> = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE communication_messages
            SET message_metadata = $2::jsonb,
                projected_at = $3
            WHERE message_id = $1
            RETURNING
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
            "#,
        )
        .bind(message_id.trim())
        .bind(&updated_metadata)
        .bind(observed_at)
        .fetch_one(&mut *transaction)
        .await?;
        let updated = row_to_telegram_message(row)?;
        capture_message_projection_observation_in_transaction(
            &mut transaction,
            "COMMUNICATION_ATTACHMENT",
            &updated,
            observed_at,
            "telegram_attachment_download_state_update",
            json!({
                "message_id": updated.message_id,
                "account_id": updated.account_id,
                "provider_message_id": updated.provider_message_id,
                "provider_chat_id": updated.provider_chat_id,
                "attachment_id": provider_attachment_id,
                "tdlib_file_id": tdlib_file_id,
                "download_state": download_state,
                "local_path": local_path,
                "size_bytes": size_bytes,
                "content_type": content_type,
                "filename": filename,
                "previous_metadata": current.metadata,
                "message_metadata": updated.metadata,
            }),
            "telegram.client.messages.attachments.update_message_attachment_download_state",
        )
        .await?;
        transaction.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;
    use testkit::context::TestContext;

    use super::*;
    use crate::domains::mail::core::{
        CommunicationIngestionStore, CommunicationProviderKind, NewProviderAccount,
        NewRawCommunicationRecord,
    };
    use crate::domains::mail::messages::MessageProjectionStore;
    use crate::integrations::telegram::client::project_raw_telegram_message;
    use crate::vault::CommunicationProviderAccountStore;

    #[tokio::test]
    async fn update_message_attachment_download_state_patches_projected_metadata() {
        let ctx = TestContext::new().await;
        let pool = ctx.pool().clone();
        let communication_store = CommunicationIngestionStore::new(pool.clone());
        let message_store = MessageProjectionStore::new(pool.clone());
        let telegram_store = TelegramStore::new(pool.clone());
        let suffix = format!("{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));
        let account_id = format!("telegram-media-metadata-{suffix}");
        let provider_chat_id = format!("-100{suffix}");
        let provider_message_id = format!("{provider_chat_id}:7001");

        CommunicationProviderAccountStore::new(pool.clone())
            .upsert(
                &NewProviderAccount::new(
                    &account_id,
                    CommunicationProviderKind::TelegramUser,
                    "Telegram Media Metadata",
                    format!("tg-media-metadata-{suffix}"),
                )
                .config(json!({"runtime": "tdlib_qr_authorized"})),
            )
            .await
            .expect("provider account");
        let raw = communication_store
            .record_raw_source(
                &NewRawCommunicationRecord::new(
                    format!("raw:telegram-media-metadata:{suffix}"),
                    &account_id,
                    "telegram_message",
                    &provider_message_id,
                    format!("sha256:{suffix}"),
                    format!("telegram-tdlib-history:{account_id}:{provider_chat_id}"),
                    json!({
                        "provider_chat_id": provider_chat_id,
                        "chat_title": "Media Channel",
                        "chat_kind": "channel",
                        "sender_id": format!("chat:{provider_chat_id}"),
                        "sender_display_name": "Media Channel",
                        "text": "",
                        "delivery_state": "received",
                        "tdlib_raw": {
                            "@type": "message",
                            "id": 7001_i64,
                            "chat_id": provider_chat_id,
                            "content": {"@type": "messagePhoto"}
                        }
                    }),
                )
                .occurred_at(Utc::now())
                .provenance(json!({
                    "provider": "telegram",
                    "provider_kind": "telegram_user",
                    "runtime": "tdlib",
                    "account_id": account_id,
                    "provider_chat_id": provider_chat_id,
                })),
            )
            .await
            .expect("raw source");

        let projected = project_raw_telegram_message(&message_store, &raw)
            .await
            .expect("project media message");

        telegram_store
            .update_message_attachment_download_state(
                &projected.message_id,
                "attachment-1",
                7001,
                "downloaded",
                Some("/tmp/hermes-telegram-photo.jpg"),
                Some(2048),
                "image/jpeg",
                Some("photo.jpg"),
            )
            .await
            .expect("update projected attachment metadata");

        let metadata: serde_json::Value = sqlx::query_scalar(
            "SELECT message_metadata FROM communication_messages WHERE message_id = $1",
        )
        .bind(&projected.message_id)
        .fetch_one(&pool)
        .await
        .expect("message metadata");
        let attachments = metadata["attachments"]
            .as_array()
            .expect("attachments array");
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0]["attachment_id"], json!("attachment-1"));
        assert_eq!(attachments[0]["tdlib_file_id"], json!(7001));
        assert_eq!(attachments[0]["download_state"], json!("downloaded"));
        assert_eq!(
            attachments[0]["local_path"],
            json!("/tmp/hermes-telegram-photo.jpg")
        );
        assert_eq!(attachments[0]["content_type"], json!("image/jpeg"));
        assert_eq!(attachments[0]["filename"], json!("photo.jpg"));
        assert_eq!(attachments[0]["size"], json!(2048));

        let observation_row = sqlx::query(
            r#"
            SELECT kind.code AS kind_code, link.relationship_kind, observation.payload
            FROM observation_links link
            JOIN observations observation
              ON observation.observation_id = link.observation_id
            JOIN observation_kind_definitions kind
              ON kind.kind_definition_id = observation.kind_definition_id
            WHERE link.domain = 'communications'
              AND link.entity_kind = 'communication_message'
              AND link.entity_id = $1
              AND link.relationship_kind = 'telegram_attachment_download_state_update'
            ORDER BY observation.captured_at DESC
            LIMIT 1
            "#,
        )
        .bind(&projected.message_id)
        .fetch_one(&pool)
        .await
        .expect("attachment download observation");
        assert_eq!(
            observation_row.get::<String, _>("kind_code"),
            "COMMUNICATION_ATTACHMENT"
        );
        let payload = observation_row.get::<serde_json::Value, _>("payload");
        assert_eq!(payload["attachment_id"], json!("attachment-1"));
        assert_eq!(payload["download_state"], json!("downloaded"));
        assert_eq!(payload["content_type"], json!("image/jpeg"));
    }
}
