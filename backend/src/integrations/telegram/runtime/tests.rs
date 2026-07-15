use crate::integrations::telegram::runtime::models::{
    TelegramHistorySyncMode, TelegramHistorySyncRequest, TelegramHistorySyncResponse,
};
use serde_json::json;

use super::*;

#[test]
fn history_sync_request_accepts_older_cursor() {
    let request: TelegramHistorySyncRequest = serde_json::from_value(json!({
        "account_id": "telegram-primary",
        "provider_chat_id": "-100123456789",
        "from_message_id": 987654321,
        "mode": "older",
        "limit": 100
    }))
    .expect("history request");

    request.validate().expect("valid history request");
    assert_eq!(request.mode(), TelegramHistorySyncMode::Older);
    assert_eq!(request.from_message_id, Some(987654321));
}

#[test]
fn history_sync_response_exposes_next_cursor() {
    let response = TelegramHistorySyncResponse {
        account_id: "telegram-primary".to_owned(),
        provider_chat_id: "-100123456789".to_owned(),
        runtime_kind: "tdlib_qr_authorized".to_owned(),
        status: "synced".to_owned(),
        synced_count: 100,
        has_more: true,
        next_from_message_id: Some(12345),
        items: Vec::new(),
    };

    let value = serde_json::to_value(response).expect("serialized response");
    assert_eq!(value["has_more"], json!(true));
    assert_eq!(value["next_from_message_id"], json!(12345));
}
