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
        .append_message_metadata_observation(message, &metadata)
        .await?;

    let mut observed = message.clone();
    observed.metadata = metadata;
    Ok(Some(observed))
}

pub(super) async fn project_provider_message_content_observation(
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
        .append_message_content_observation(message, &snapshot.text, &metadata, observed_at)
        .await?;

    let mut observed = message.clone();
    observed.text = snapshot.text.clone();
    observed.metadata = metadata;
    Ok(Some(observed))
}

pub(super) async fn project_provider_message_edit_observation(
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
        .append_message_metadata_observation(message, &metadata)
        .await?;

    let mut observed = message.clone();
    observed.metadata = metadata;
    Ok(Some(observed))
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
