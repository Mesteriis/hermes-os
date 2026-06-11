use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::domains::mail::messages::{
    MessageProjectionError, MessageProjectionStore, ProjectedMessage,
};
#[cfg(test)]
#[allow(unused_imports)]
use serde_json::Value;

/// Pin/snooze/label operations on messages stored in message_metadata JSONB.
pub struct MessageFlags;

impl MessageFlags {
    const PINNED_KEY: &'static str = "pinned";
    const SNOOZE_UNTIL_KEY: &'static str = "snooze_until";
    const LABELS_KEY: &'static str = "labels";
    const IS_MUTED_KEY: &'static str = "muted";

    pub fn is_pinned(message: &ProjectedMessage) -> bool {
        message
            .message_metadata
            .get(Self::PINNED_KEY)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub fn snooze_until(message: &ProjectedMessage) -> Option<DateTime<Utc>> {
        message
            .message_metadata
            .get(Self::SNOOZE_UNTIL_KEY)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
    }

    pub fn labels(message: &ProjectedMessage) -> Vec<String> {
        message
            .message_metadata
            .get(Self::LABELS_KEY)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn is_muted(message: &ProjectedMessage) -> bool {
        message
            .message_metadata
            .get(Self::IS_MUTED_KEY)
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub async fn toggle_pin(
        store: &MessageProjectionStore,
        message_id: &str,
    ) -> Result<bool, MessageFlagsError> {
        let msg = store
            .message(message_id)
            .await?
            .ok_or(MessageFlagsError::NotFound)?;
        let currently = Self::is_pinned(&msg);
        let mut meta = msg.message_metadata.clone();
        meta[Self::PINNED_KEY] = serde_json::Value::Bool(!currently);
        store.set_message_metadata(message_id, &meta).await?;
        Ok(!currently)
    }

    pub async fn snooze(
        store: &MessageProjectionStore,
        message_id: &str,
        until: DateTime<Utc>,
    ) -> Result<(), MessageFlagsError> {
        let msg = store
            .message(message_id)
            .await?
            .ok_or(MessageFlagsError::NotFound)?;
        let mut meta = msg.message_metadata.clone();
        meta[Self::SNOOZE_UNTIL_KEY] = serde_json::Value::String(until.to_rfc3339());
        store.set_message_metadata(message_id, &meta).await?;
        Ok(())
    }

    pub async fn add_label(
        store: &MessageProjectionStore,
        message_id: &str,
        label: &str,
    ) -> Result<(), MessageFlagsError> {
        let msg = store
            .message(message_id)
            .await?
            .ok_or(MessageFlagsError::NotFound)?;
        let mut labels = Self::labels(&msg);
        if !labels.contains(&label.to_owned()) {
            labels.push(label.to_owned());
        }
        let mut meta = msg.message_metadata.clone();
        meta[Self::LABELS_KEY] = serde_json::to_value(&labels).unwrap_or_default();
        store.set_message_metadata(message_id, &meta).await?;
        Ok(())
    }

    pub async fn remove_label(
        store: &MessageProjectionStore,
        message_id: &str,
        label: &str,
    ) -> Result<(), MessageFlagsError> {
        let msg = store
            .message(message_id)
            .await?
            .ok_or(MessageFlagsError::NotFound)?;
        let mut labels = Self::labels(&msg);
        labels.retain(|l| l != label);
        let mut meta = msg.message_metadata.clone();
        meta[Self::LABELS_KEY] = serde_json::to_value(&labels).unwrap_or_default();
        store.set_message_metadata(message_id, &meta).await?;
        Ok(())
    }

    pub async fn toggle_mute(
        store: &MessageProjectionStore,
        message_id: &str,
    ) -> Result<bool, MessageFlagsError> {
        let msg = store
            .message(message_id)
            .await?
            .ok_or(MessageFlagsError::NotFound)?;
        let currently = Self::is_muted(&msg);
        let mut meta = msg.message_metadata.clone();
        meta[Self::IS_MUTED_KEY] = serde_json::Value::Bool(!currently);
        store.set_message_metadata(message_id, &meta).await?;
        Ok(!currently)
    }
}

#[derive(Debug, Error)]
pub enum MessageFlagsError {
    #[error(transparent)]
    MessageProjection(#[from] MessageProjectionError),
    #[error("message not found")]
    NotFound,
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::domains::mail::messages::{LocalMessageState, WorkflowState};
    use chrono::Utc;
    use serde_json::json;

    fn test_message(meta: Value) -> ProjectedMessage {
        ProjectedMessage {
            message_id: "m:1".into(),
            raw_record_id: "r:1".into(),
            account_id: "a:1".into(),
            provider_record_id: "p:1".into(),
            subject: "S".into(),
            sender: "s@e.com".into(),
            recipients: vec!["r@e.com".into()],
            body_text: "B".into(),
            occurred_at: Some(Utc::now()),
            projected_at: Utc::now(),
            channel_kind: "email".into(),
            conversation_id: None,
            sender_display_name: None,
            delivery_state: "received".into(),
            message_metadata: meta,
            workflow_state: WorkflowState::New,
            importance_score: None,
            ai_category: None,
            ai_summary: None,
            ai_summary_generated_at: None,
            local_state: LocalMessageState::Active,
            local_state_changed_at: None,
            local_state_reason: None,
        }
    }

    #[test]
    fn is_pinned_detects_flag() {
        let msg = test_message(serde_json::json!({"pinned": true}));
        assert!(MessageFlags::is_pinned(&msg));
        let msg2 = test_message(serde_json::json!({}));
        assert!(!MessageFlags::is_pinned(&msg2));
    }

    #[test]
    fn labels_extracts_array() {
        let msg = test_message(serde_json::json!({"labels": ["finance", "urgent"]}));
        assert_eq!(MessageFlags::labels(&msg), vec!["finance", "urgent"]);
    }

    #[test]
    fn is_muted_detects_flag() {
        let msg = test_message(serde_json::json!({"muted": true}));
        assert!(MessageFlags::is_muted(&msg));
    }

    #[test]
    fn snooze_until_parses_datetime() {
        let msg = test_message(serde_json::json!({"snooze_until": "2026-06-08T10:00:00+00:00"}));
        assert!(MessageFlags::snooze_until(&msg).is_some());
    }
}
