use crate::integrations::telegram::client::{
    TelegramQrLoginStartRequest, TelegramQrLoginStatus, TelegramQrLoginStatusResponse,
};

mod environment;
mod parsing_snapshots;
mod qr_login_flows;
mod request_builders;

fn test_qr_login_response(status: TelegramQrLoginStatus) -> TelegramQrLoginStatusResponse {
    TelegramQrLoginStatusResponse {
        setup_id: "setup-id".to_owned(),
        account_id: "telegram-account".to_owned(),
        status,
        qr_link: Some("tg://login?token=test-token".to_owned()),
        qr_svg: Some("<svg></svg>".to_owned()),
        telegram_user_id: None,
        telegram_username: None,
        suggested_account_id: None,
        suggested_display_name: None,
        suggested_external_account_id: None,
        expires_at: None,
        poll_after_ms: 2_000,
        message: Some("Waiting".to_owned()),
    }
}

fn test_qr_login_request() -> TelegramQrLoginStartRequest {
    TelegramQrLoginStartRequest {
        account_id: "telegram-account".to_owned(),
        display_name: "Telegram Account".to_owned(),
        external_account_id: "telegram-account".to_owned(),
        api_id: Some(12345),
        api_hash: Some("telegram-api-hash".to_owned()),
        session_encryption_key: Some("telegram-session-key".to_owned()),
        tdlib_data_path: Some("docker/data/telegram/telegram-account".to_owned()),
        transcription_enabled: true,
    }
}
