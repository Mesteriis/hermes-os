use serde_json::Value;

use crate::integrations::telegram::tdjson::snapshots::TelegramTdlibMessageSnapshot;

use super::super::errors::TelegramError;
use super::super::models::chats::TelegramChatKind;
use super::super::models::messages::{NewTelegramMessage, TelegramObservedMessage};
use super::super::store::TelegramStore;
use super::message_metadata::derive_tdlib_attachment_metadata;
use super::reaction_metadata::derive_tdlib_reaction_summary_metadata;

impl TelegramStore {
    pub(crate) async fn ingest_tdlib_message_snapshot(
        &self,
        account_id: &str,
        snapshot: &TelegramTdlibMessageSnapshot,
        import_batch_id: &str,
    ) -> Result<TelegramObservedMessage, TelegramError> {
        let provider_account = self.telegram_provider_account(account_id).await?;
        let existing_chat = self
            .telegram_chat(&provider_account.account_id, &snapshot.provider_chat_id)
            .await?;
        let (chat_kind, chat_title) = match existing_chat {
            Some(chat) => (
                TelegramChatKind::try_from(chat.chat_kind.as_str())?,
                chat.title,
            ),
            None => (
                TelegramChatKind::Private,
                format!("Telegram Chat {}", snapshot.provider_chat_id),
            ),
        };
        let provider_message_id = format!(
            "{}:{}",
            snapshot.provider_chat_id, snapshot.provider_message_id
        );
        let message = NewTelegramMessage {
            account_id: provider_account.account_id,
            provider_chat_id: snapshot.provider_chat_id.clone(),
            provider_message_id,
            chat_kind,
            chat_title,
            sender_id: snapshot.sender_id.clone(),
            sender_display_name: snapshot.sender_display_name.clone(),
            text: snapshot.text.clone(),
            import_batch_id: import_batch_id.trim().to_owned(),
            occurred_at: snapshot.occurred_at,
            delivery_state: snapshot.delivery_state,
        };

        let observed = self
            .ingest_message_with_runtime(&message, "tdlib", Some(snapshot.raw.clone()))
            .await?;
        self.append_missing_tdlib_metadata_observation(
            &message.account_id,
            &message.provider_message_id,
            &snapshot.raw,
        )
        .await?;

        Ok(observed)
    }

    async fn append_missing_tdlib_metadata_observation(
        &self,
        account_id: &str,
        provider_message_id: &str,
        raw: &Value,
    ) -> Result<(), TelegramError> {
        let derived_attachments = derive_tdlib_attachment_metadata(raw);
        let reaction_summary = derive_tdlib_reaction_summary_metadata(raw);
        if derived_attachments.is_empty() && reaction_summary.is_none() {
            return Ok(());
        }

        let Some(message) = self
            .message_by_provider_message_id(account_id, provider_message_id)
            .await?
        else {
            // New messages are projected from their raw source with the attachments already present.
            return Ok(());
        };
        let Some(metadata) =
            merge_missing_tdlib_metadata(&message.metadata, derived_attachments, reaction_summary)
        else {
            return Ok(());
        };

        self.append_message_metadata_observation(&message, &metadata)
            .await?;
        Ok(())
    }
}

fn merge_missing_tdlib_metadata(
    existing_metadata: &Value,
    derived_attachments: Vec<Value>,
    reaction_summary: Option<Value>,
) -> Option<Value> {
    let mut metadata = existing_metadata.as_object()?.clone();
    let mut attachments = metadata
        .get("attachments")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut inserted = false;

    for attachment in derived_attachments {
        let Some(identity) = attachment_identity(&attachment) else {
            continue;
        };
        if attachments
            .iter()
            .any(|current| attachment_identity(current) == Some(identity))
        {
            continue;
        }
        attachments.push(attachment);
        inserted = true;
    }

    if !metadata.contains_key("reaction_summary")
        && let Some(reaction_summary) = reaction_summary
    {
        metadata.insert("reaction_summary".to_owned(), reaction_summary);
        inserted = true;
    }

    if !inserted {
        return None;
    }

    metadata.insert("attachments".to_owned(), Value::Array(attachments));
    Some(Value::Object(metadata))
}

fn attachment_identity(attachment: &Value) -> Option<&str> {
    attachment
        .get("provider_attachment_id")
        .or_else(|| attachment.get("attachment_id"))
        .or_else(|| attachment.get("id"))
        .and_then(Value::as_str)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::merge_missing_tdlib_metadata;

    #[test]
    fn adds_missing_attachment_metadata_without_downgrading_downloaded_attachments() {
        let merged = merge_missing_tdlib_metadata(
            &json!({
                "attachments": [{
                    "provider_attachment_id": "tdlib:document:1",
                    "attachment_id": "attachment:canonical:1",
                    "download_state": "downloaded"
                }]
            }),
            vec![
                json!({
                    "attachment_id": "tdlib:document:1",
                    "download_state": "remote"
                }),
                json!({
                    "attachment_id": "tdlib:photo:2",
                    "attachment_type": "photo",
                    "download_state": "remote"
                }),
            ],
            Some(json!({ "reactions": [{ "reaction_emoji": "👍", "count": 2 }] })),
        )
        .expect("new attachment metadata");

        assert_eq!(merged["attachments"].as_array().map(Vec::len), Some(2));
        assert_eq!(
            merged["attachments"][0]["attachment_id"],
            json!("attachment:canonical:1")
        );
        assert_eq!(
            merged["attachments"][0]["download_state"],
            json!("downloaded")
        );
        assert_eq!(
            merged["attachments"][1]["attachment_id"],
            json!("tdlib:photo:2")
        );
        assert_eq!(
            merged["reaction_summary"]["reactions"][0]["count"],
            json!(2)
        );
    }
}
