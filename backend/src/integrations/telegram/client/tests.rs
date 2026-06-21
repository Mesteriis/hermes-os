use chrono::Utc;

use super::validation::{validate_chat_list_limit, validate_message_list_limit};
use super::*;

fn valid_message(text: &str) -> NewTelegramMessage {
    NewTelegramMessage {
        account_id: "telegram-account".to_owned(),
        provider_chat_id: "12345".to_owned(),
        provider_message_id: "12345:67890".to_owned(),
        chat_kind: TelegramChatKind::Private,
        chat_title: "Private Chat".to_owned(),
        sender_id: "user:12345".to_owned(),
        sender_display_name: "Telegram User".to_owned(),
        text: text.to_owned(),
        import_batch_id: "telegram-tdlib-history:telegram-account:12345".to_owned(),
        occurred_at: Utc::now(),
        delivery_state: TelegramDeliveryState::Received,
    }
}

#[test]
fn fixture_message_validation_rejects_empty_text() {
    let message = valid_message("   ");

    let error = message
        .validate_for_runtime("fixture")
        .expect_err("fixture text validation should reject empty body");

    assert!(matches!(error, TelegramError::InvalidRequest(_)));
    assert!(error.to_string().contains("text must not be empty"));
}

#[test]
fn tdlib_message_validation_allows_empty_text_for_media_snapshots() {
    let message = valid_message("");

    message
        .validate_for_runtime("tdlib")
        .expect("TDLib media snapshots may not have text");
}

#[test]
fn message_list_limit_allows_full_selected_chat_window() {
    assert_eq!(validate_message_list_limit(5000).expect("limit"), 5000);
    assert!(matches!(
        validate_message_list_limit(5001),
        Err(TelegramError::InvalidRequest(_))
    ));
}

#[test]
fn chat_list_limit_allows_full_metadata_window() {
    assert_eq!(validate_chat_list_limit(5000).expect("limit"), 5000);
    assert!(matches!(
        validate_chat_list_limit(5001),
        Err(TelegramError::InvalidRequest(_))
    ));
}
