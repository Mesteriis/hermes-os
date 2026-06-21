use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::integrations::telegram::client::{
    TelegramError, TelegramMessage, TelegramStore, derive_tdlib_reaction_summary_metadata,
};
use crate::integrations::telegram::tdjson::{
    TelegramTdlibMessageContentSnapshot, TelegramTdlibMessageEditedSnapshot,
};

pub(super) async fn update_message_reaction_summary(
    store: &TelegramStore,
    message: &TelegramMessage,
    raw: &Value,
) -> Result<Option<TelegramMessage>, TelegramError> {
    let mut metadata = message.metadata.clone();
    let Some(metadata_map) = metadata.as_object_mut() else {
        return Err(TelegramError::InvalidRequest(
            "telegram message metadata must be a JSON object".to_owned(),
        ));
    };

    if let Some(summary) = derive_tdlib_reaction_summary_metadata(raw) {
        metadata_map.insert("reaction_summary".to_owned(), summary);
    } else {
        metadata_map.remove("reaction_summary");
    }

    store
        .apply_message_metadata(&message.message_id, &metadata)
        .await
}

pub(super) async fn apply_provider_message_content_update(
    store: &TelegramStore,
    message: &TelegramMessage,
    snapshot: &TelegramTdlibMessageContentSnapshot,
    observed_at: DateTime<Utc>,
) -> Result<Option<TelegramMessage>, TelegramError> {
    let mut metadata = message.metadata.clone();
    let Some(metadata_map) = metadata.as_object_mut() else {
        return Err(TelegramError::InvalidRequest(
            "telegram message metadata must be a JSON object".to_owned(),
        ));
    };
    metadata_map.insert("text".to_owned(), Value::String(snapshot.text.clone()));
    metadata_map.insert("tdlib_content".to_owned(), snapshot.new_content.clone());
    metadata_map.insert(
        "last_provider_content_update_source".to_owned(),
        Value::String(snapshot.source_event.clone()),
    );

    store
        .apply_message_projection_update(
            &message.message_id,
            &snapshot.text,
            &metadata,
            observed_at,
        )
        .await
}

pub(super) async fn apply_provider_message_edit_metadata(
    store: &TelegramStore,
    message: &TelegramMessage,
    snapshot: &TelegramTdlibMessageEditedSnapshot,
) -> Result<Option<TelegramMessage>, TelegramError> {
    let mut metadata = message.metadata.clone();
    let Some(metadata_map) = metadata.as_object_mut() else {
        return Err(TelegramError::InvalidRequest(
            "telegram message metadata must be a JSON object".to_owned(),
        ));
    };
    metadata_map.insert(
        "provider_edit_timestamp".to_owned(),
        Value::String(snapshot.edit_timestamp.to_rfc3339()),
    );
    metadata_map.insert(
        "last_provider_edit_source".to_owned(),
        Value::String(snapshot.source_event.clone()),
    );
    if let Some(reply_markup) = &snapshot.reply_markup {
        metadata_map.insert("tdlib_reply_markup".to_owned(), reply_markup.clone());
    }

    store
        .apply_message_metadata(&message.message_id, &metadata)
        .await
}

pub(super) fn observed_edit_timestamp(
    message: &TelegramMessage,
    fallback: DateTime<Utc>,
) -> DateTime<Utc> {
    message
        .metadata
        .get("provider_edit_timestamp")
        .and_then(Value::as_str)
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&Utc))
        .unwrap_or(fallback)
}
