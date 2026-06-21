use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};

use crate::integrations::telegram::client::{
    TelegramError, TelegramQrLoginPasswordRequest, TelegramQrLoginStatus,
};

#[test]
fn wait_password_state_is_not_a_qr_request_state() {
    assert!(!super::super::state_allows_qr_request(
        "authorizationStateWaitPassword"
    ));
}

#[test]
fn password_waiting_response_does_not_expose_stale_qr_token() {
    let response = super::super::password_waiting_response(
        "setup-id",
        "telegram-account",
        "Telegram requires your 2-step verification password.",
    );

    assert_eq!(response.status, TelegramQrLoginStatus::WaitingPassword);
    assert_eq!(response.qr_link, None);
    assert_eq!(response.qr_svg, None);
    assert_eq!(response.poll_after_ms, 2_000);
}

#[test]
fn ready_response_for_existing_tdlib_session_does_not_expose_qr_token() {
    let identity = super::super::TelegramQrLoginIdentity {
        user_id: "123456789".to_owned(),
        username: Some("Test_User".to_owned()),
        suggested_account_id: "123456789_account_test_user".to_owned(),
        suggested_display_name: "@Test_User".to_owned(),
        suggested_external_account_id: "telegram:123456789".to_owned(),
    };

    let response = super::super::ready_response(
        "setup-id",
        "telegram-account",
        "Telegram TDLib session is already authorized.",
        Some(&identity),
    );

    assert_eq!(response.status, TelegramQrLoginStatus::Ready);
    assert_eq!(response.qr_link, None);
    assert_eq!(response.qr_svg, None);
    assert_eq!(
        response.suggested_account_id.as_deref(),
        Some("123456789_account_test_user")
    );
    assert_eq!(
        response.suggested_display_name.as_deref(),
        Some("@Test_User")
    );
    assert_eq!(
        response.suggested_external_account_id.as_deref(),
        Some("telegram:123456789")
    );
}

#[test]
fn qr_preparing_response_is_cancellable_without_exposing_qr_token() {
    let response = super::super::qr_preparing_response("setup-id", "telegram-account");

    assert_eq!(response.setup_id, "setup-id");
    assert_eq!(response.status, TelegramQrLoginStatus::WaitingQrScan);
    assert_eq!(response.qr_link, None);
    assert_eq!(response.qr_svg, None);
    assert_eq!(response.poll_after_ms, 1_000);
}

#[test]
fn qr_password_submission_sends_command_to_pending_session() {
    let (command_tx, command_rx) = mpsc::channel();
    let pending = Arc::new(Mutex::new(HashMap::from([(
        "setup-id".to_owned(),
        super::super::TelegramQrLoginSession {
            response: super::test_qr_login_response(TelegramQrLoginStatus::WaitingPassword),
            command_tx,
            worker_completion: super::super::new_worker_completion(),
        },
    )])));

    let login_check_value = "tdlib-check-value".to_owned();

    let response = super::super::submit_qr_login_password(
        pending,
        "setup-id",
        TelegramQrLoginPasswordRequest {
            password: login_check_value.clone(),
        },
    )
    .expect("password accepted");

    assert_eq!(response.status, TelegramQrLoginStatus::WaitingPassword);
    assert_eq!(
        response.message.as_deref(),
        Some("Checking Telegram password.")
    );
    assert_eq!(
        command_rx.try_recv().expect("password command"),
        super::super::TelegramQrLoginCommand::CheckPassword(login_check_value)
    );
}

#[test]
fn qr_password_submission_requires_waiting_password_status() {
    let (command_tx, command_rx) = mpsc::channel();
    let pending = Arc::new(Mutex::new(HashMap::from([(
        "setup-id".to_owned(),
        super::super::TelegramQrLoginSession {
            response: super::test_qr_login_response(TelegramQrLoginStatus::WaitingQrScan),
            command_tx,
            worker_completion: super::super::new_worker_completion(),
        },
    )])));

    let login_check_value = "tdlib-check-value".to_owned();

    let error = super::super::submit_qr_login_password(
        pending,
        "setup-id",
        TelegramQrLoginPasswordRequest {
            password: login_check_value,
        },
    )
    .expect_err("password must not be accepted before TDLib asks for it");

    assert!(matches!(error, TelegramError::InvalidRequest(_)));
    assert!(command_rx.try_recv().is_err());
}

#[test]
fn qr_login_cancel_removes_pending_session_and_notifies_worker() {
    let (command_tx, command_rx) = mpsc::channel();
    let worker_completion = super::super::new_worker_completion();
    super::super::mark_worker_complete(&worker_completion);
    let pending = Arc::new(Mutex::new(HashMap::from([(
        "setup-id".to_owned(),
        super::super::TelegramQrLoginSession {
            response: super::test_qr_login_response(TelegramQrLoginStatus::WaitingQrScan),
            command_tx,
            worker_completion,
        },
    )])));

    super::super::cancel_qr_login(Arc::clone(&pending), "setup-id").expect("QR login cancelled");

    assert!(
        !pending
            .lock()
            .expect("pending lock")
            .contains_key("setup-id")
    );
    assert_eq!(
        command_rx.try_recv().expect("cancel command"),
        super::super::TelegramQrLoginCommand::Cancel
    );
}

#[test]
fn qr_login_cancel_unknown_setup_returns_not_found() {
    let pending = Arc::new(Mutex::new(HashMap::new()));

    let error = super::super::cancel_qr_login(pending, "missing-setup")
        .expect_err("unknown QR setup must not be cancelled");

    assert!(matches!(error, TelegramError::QrLoginNotFound));
}

#[test]
fn qr_login_start_cancels_existing_sessions_for_same_account() {
    let (same_account_tx, same_account_rx) = mpsc::channel();
    let (other_account_tx, other_account_rx) = mpsc::channel();
    let same_account_completion = super::super::new_worker_completion();
    let other_account_completion = super::super::new_worker_completion();
    super::super::mark_worker_complete(&same_account_completion);
    super::super::mark_worker_complete(&other_account_completion);
    let mut other_response = super::test_qr_login_response(TelegramQrLoginStatus::WaitingQrScan);
    other_response.setup_id = "other-setup-id".to_owned();
    other_response.account_id = "other-account".to_owned();
    let pending = Arc::new(Mutex::new(HashMap::from([
        (
            "setup-id".to_owned(),
            super::super::TelegramQrLoginSession {
                response: super::test_qr_login_response(TelegramQrLoginStatus::WaitingQrScan),
                command_tx: same_account_tx,
                worker_completion: same_account_completion,
            },
        ),
        (
            "other-setup-id".to_owned(),
            super::super::TelegramQrLoginSession {
                response: other_response,
                command_tx: other_account_tx,
                worker_completion: other_account_completion,
            },
        ),
    ])));

    super::super::cancel_existing_qr_logins_for_account(&pending, "telegram-account")
        .expect("same-account sessions cancelled");

    let pending = pending.lock().expect("pending lock");
    assert!(!pending.contains_key("setup-id"));
    assert!(pending.contains_key("other-setup-id"));
    assert_eq!(
        same_account_rx.try_recv().expect("same account cancel"),
        super::super::TelegramQrLoginCommand::Cancel
    );
    assert!(other_account_rx.try_recv().is_err());
}
