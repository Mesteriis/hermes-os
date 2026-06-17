use crate::integrations::telegram::client::{TelegramQrLoginStatus, TelegramQrLoginStatusResponse};

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
