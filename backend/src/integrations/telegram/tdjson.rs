mod client;
mod identifiers;
mod library_paths;
mod parsing;
mod qr_login;
mod qr_login_support;
mod requests;
mod snapshots;

pub(crate) use self::client::{TdJsonClient, TdJsonLibrary, runtime_available};
pub(crate) use self::parsing::{
    authorization_state, is_tdlib_database_encryption_key_needed_error,
    is_tdlib_parameters_not_specified_error, parse_tdlib_chat_ids, parse_tdlib_chat_snapshot,
    parse_tdlib_file_snapshot, parse_tdlib_message_list, parse_tdlib_message_snapshot,
    tdlib_error_message,
};
pub(crate) use self::qr_login::{cancel_qr_login, start_qr_login, submit_qr_login_password};
pub(crate) use self::qr_login_support::{PendingQrLoginMap, TelegramQrLoginSession};
pub(crate) use self::requests::{
    check_database_encryption_key_request, set_tdlib_parameters_request, tdlib_database_directory,
    tdlib_download_file_request, tdlib_get_chat_history_request, tdlib_get_chat_request,
    tdlib_get_chats_request, tdlib_load_chats_request, tdlib_send_text_message_request,
};
pub(crate) use self::snapshots::{
    TelegramTdlibChatSnapshot, TelegramTdlibFileSnapshot, TelegramTdlibMessageSnapshot,
};

#[cfg(test)]
use self::library_paths::{tdjson_library_candidates_with_context, tdjson_platform_dir};
#[cfg(test)]
use self::qr_login::cancel_existing_qr_logins_for_account;
#[cfg(test)]
use self::qr_login_support::{
    TelegramQrLoginCommand, TelegramQrLoginIdentity, mark_worker_complete, new_worker_completion,
    parse_tdlib_user_identity, password_waiting_response, qr_preparing_response, ready_response,
    render_qr_svg, state_allows_qr_request,
};

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::{Arc, Mutex, mpsc};

    use base64::Engine as _;
    use base64::engine::general_purpose::STANDARD;
    use serde_json::json;

    use crate::integrations::telegram::client::{
        TelegramError, TelegramQrLoginPasswordRequest, TelegramQrLoginStartRequest,
        TelegramQrLoginStatus, TelegramQrLoginStatusResponse,
    };

    use super::{
        TelegramQrLoginCommand, TelegramQrLoginSession, check_database_encryption_key_request,
        render_qr_svg, set_tdlib_parameters_request,
    };

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_tdjson_candidates_prefer_bundled_tauri_resources() {
        let exe_dir = Path::new("/Applications/Hermes Hub.app/Contents/MacOS");
        let cwd = Path::new("/workspace/hermes-hub");
        let candidates =
            super::tdjson_library_candidates_with_context(None, Some(exe_dir), Some(cwd));
        let bundled_resource = Path::new("/Applications/Hermes Hub.app/Contents/Resources")
            .join("tdlib")
            .join(super::tdjson_platform_dir())
            .join("libtdjson.dylib");
        let dev_resource = cwd
            .join("frontend/src-tauri/resources/tdlib")
            .join(super::tdjson_platform_dir())
            .join("libtdjson.dylib");

        assert_eq!(candidates.first(), Some(&bundled_resource));
        assert!(candidates.contains(&dev_resource));
        assert!(
            candidates
                .iter()
                .position(|candidate| candidate == &bundled_resource)
                < candidates.iter().position(
                    |candidate| candidate == Path::new("/opt/homebrew/lib/libtdjson.dylib")
                )
        );
    }

    #[test]
    fn renders_tdlib_qr_link_as_svg() {
        let svg = render_qr_svg("tg://login?token=test-token").expect("QR SVG");

        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.len() > 100);
    }

    #[test]
    fn tdlib_parameters_use_legacy_nested_shape_for_tdlib_1_8_runtime() {
        let request = TelegramQrLoginStartRequest {
            account_id: "telegram-qr".to_owned(),
            display_name: "Telegram QR".to_owned(),
            external_account_id: "qr-login:telegram-qr".to_owned(),
            api_id: Some(12345),
            api_hash: Some("telegram-api-hash".to_owned()),
            session_encryption_key: Some("telegram-session-key".to_owned()),
            tdlib_data_path: Some("docker/data/telegram/telegram-qr".to_owned()),
            transcription_enabled: true,
        };

        let command =
            set_tdlib_parameters_request(&request, Path::new("docker/data/telegram/telegram-qr"))
                .expect("TDLib parameters");

        assert_eq!(command["@type"], "setTdlibParameters");
        assert_eq!(command["parameters"]["api_id"], 12345);
        assert_eq!(command["parameters"]["api_hash"], "telegram-api-hash");
        assert_eq!(command["parameters"]["enable_storage_optimizer"], true);
        assert_eq!(command["parameters"]["ignore_file_names"], false);
        assert_eq!(
            command["parameters"]["database_encryption_key"],
            STANDARD.encode("telegram-session-key")
        );
        assert_eq!(command["database_encryption_key"], serde_json::Value::Null);
    }

    #[test]
    fn tdlib_database_key_check_uses_same_base64_key_without_plaintext_secret() {
        let request = TelegramQrLoginStartRequest {
            account_id: "telegram-qr".to_owned(),
            display_name: "Telegram QR".to_owned(),
            external_account_id: "qr-login:telegram-qr".to_owned(),
            api_id: Some(12345),
            api_hash: Some("telegram-api-hash".to_owned()),
            session_encryption_key: Some("telegram-session-key".to_owned()),
            tdlib_data_path: Some("docker/data/telegram/telegram-qr".to_owned()),
            transcription_enabled: true,
        };

        let command = check_database_encryption_key_request(&request);

        assert_eq!(command["@type"], "checkDatabaseEncryptionKey");
        assert_eq!(
            command["encryption_key"],
            STANDARD.encode("telegram-session-key")
        );
        assert_ne!(command["encryption_key"], "telegram-session-key");
    }

    #[test]
    fn tdlib_send_text_message_request_uses_formatted_text_content() {
        let command = super::tdlib_send_text_message_request(
            123456789,
            "Hello from Hermes",
            "hermes-send-message-1",
        )
        .expect("send message request");

        assert_eq!(command["@type"], "sendMessage");
        assert_eq!(command["chat_id"], 123456789);
        assert_eq!(command["@extra"], "hermes-send-message-1");
        assert_eq!(
            command["input_message_content"]["@type"],
            "inputMessageText"
        );
        assert_eq!(
            command["input_message_content"]["text"]["@type"],
            "formattedText"
        );
        assert_eq!(
            command["input_message_content"]["text"]["text"],
            "Hello from Hermes"
        );
        assert_eq!(
            command["input_message_content"]["text"]["entities"],
            json!([])
        );
        assert_eq!(command["input_message_content"]["clear_draft"], true);
    }

    #[test]
    fn tdlib_get_chat_history_request_caps_limit_to_tdlib_page_size() {
        let command = super::tdlib_get_chat_history_request(
            123456789,
            Some(98765),
            500,
            true,
            "hermes-history-1",
        );

        assert_eq!(command["@type"], "getChatHistory");
        assert_eq!(command["chat_id"], 123456789);
        assert_eq!(command["from_message_id"], 98765);
        assert_eq!(command["offset"], 0);
        assert_eq!(command["limit"], 100);
        assert_eq!(command["only_local"], true);
        assert_eq!(command["@extra"], "hermes-history-1");
    }

    #[test]
    fn tdlib_download_file_request_uses_synchronous_on_demand_download() {
        let command = super::tdlib_download_file_request(42, 16, "hermes-download-file-42");

        assert_eq!(command["@type"], "downloadFile");
        assert_eq!(command["file_id"], 42);
        assert_eq!(command["priority"], 16);
        assert_eq!(command["offset"], 0);
        assert_eq!(command["limit"], 0);
        assert_eq!(command["synchronous"], true);
        assert_eq!(command["@extra"], "hermes-download-file-42");
    }

    #[test]
    fn parses_tdlib_file_snapshot_from_download_file_response() {
        let file = super::parse_tdlib_file_snapshot(&json!({
            "@type": "file",
            "id": 42,
            "size": 2048,
            "expected_size": 4096,
            "local": {
                "@type": "localFile",
                "path": "docker/data/telegram/account/files/document.pdf",
                "can_be_downloaded": true,
                "is_downloading_active": false,
                "is_downloading_completed": true,
                "downloaded_size": 2048
            },
            "remote": {
                "@type": "remoteFile",
                "id": "remote-file-id",
                "unique_id": "remote-unique-id",
                "is_uploading_active": false,
                "is_uploading_completed": false,
                "uploaded_size": 0
            }
        }))
        .expect("file snapshot");

        assert_eq!(file.file_id, 42);
        assert_eq!(file.size_bytes, Some(2048));
        assert_eq!(file.expected_size_bytes, Some(4096));
        assert_eq!(
            file.local_path.as_deref(),
            Some("docker/data/telegram/account/files/document.pdf")
        );
        assert!(file.is_downloading_completed);
        assert!(!file.is_downloading_active);
        assert_eq!(file.remote_unique_id.as_deref(), Some("remote-unique-id"));
        assert_eq!(file.downloaded_size_bytes, Some(2048));
    }

    #[test]
    fn parses_tdlib_chat_snapshot_from_chat_object() {
        let chat = super::parse_tdlib_chat_snapshot(&json!({
            "@type": "chat",
            "id": 123456789,
            "type": {
                "@type": "chatTypeSupergroup",
                "supergroup_id": 555,
                "is_channel": true
            },
            "title": "Release Channel",
            "last_message": {
                "@type": "message",
                "id": 42,
                "date": 1781352000
            },
            "metadata": {"ignored": true}
        }))
        .expect("chat snapshot");

        assert_eq!(chat.provider_chat_id, "123456789");
        assert_eq!(chat.chat_kind.as_str(), "channel");
        assert_eq!(chat.title, "Release Channel");
        assert_eq!(chat.username, None);
        assert_eq!(
            chat.last_message_at.expect("last message").to_rfc3339(),
            "2026-06-13T12:00:00+00:00"
        );
        assert_eq!(chat.raw["@type"], "chat");
    }

    #[test]
    fn parses_tdlib_text_message_snapshot_from_message_object() {
        let message = super::parse_tdlib_message_snapshot(&json!({
            "@type": "message",
            "id": 777,
            "chat_id": 123456789,
            "sender_id": {
                "@type": "messageSenderUser",
                "user_id": 999
            },
            "date": 1781352060,
            "is_outgoing": false,
            "content": {
                "@type": "messageText",
                "text": {
                    "@type": "formattedText",
                    "text": "Incoming TDLib text",
                    "entities": []
                }
            }
        }))
        .expect("message snapshot");

        assert_eq!(message.provider_chat_id, "123456789");
        assert_eq!(message.provider_message_id, "777");
        assert_eq!(message.sender_id, "user:999");
        assert_eq!(message.sender_display_name, "Telegram User 999");
        assert_eq!(message.text, "Incoming TDLib text");
        assert_eq!(message.delivery_state.as_str(), "received");
        assert_eq!(
            message.occurred_at.to_rfc3339(),
            "2026-06-13T12:01:00+00:00"
        );
        assert_eq!(message.raw["@type"], "message");
    }

    #[test]
    fn parses_tdlib_media_message_without_caption_as_empty_text() {
        let message = super::parse_tdlib_message_snapshot(&json!({
            "@type": "message",
            "id": 778,
            "chat_id": 123456789,
            "sender_id": {
                "@type": "messageSenderUser",
                "user_id": 999
            },
            "date": 1781352061,
            "is_outgoing": false,
            "content": {
                "@type": "messagePhoto",
                "photo": {
                    "@type": "photo",
                    "sizes": []
                }
            }
        }))
        .expect("media message snapshot");

        assert_eq!(message.provider_message_id, "778");
        assert_eq!(message.text, "");
        assert_eq!(message.raw["content"]["@type"], "messagePhoto");
    }

    #[test]
    fn recognizes_tdlib_bootstrap_error_events() {
        assert!(super::is_tdlib_parameters_not_specified_error(&json!({
            "@type": "error",
            "code": 400,
            "message": "Parameters aren't specified"
        })));
        assert!(super::is_tdlib_database_encryption_key_needed_error(
            &json!({
                "@type": "error",
                "code": 400,
                "message": "Database encryption key is needed: call checkDatabaseEncryptionKey first"
            })
        ));
    }

    #[test]
    fn parses_tdlib_user_identity_for_qr_account_defaults() {
        let identity = super::parse_tdlib_user_identity(&json!({
            "@type": "user",
            "id": 123456789,
            "usernames": {
                "active_usernames": ["Test_User"]
            }
        }))
        .expect("identity");

        assert_eq!(identity.user_id, "123456789");
        assert_eq!(identity.username.as_deref(), Some("Test_User"));
        assert_eq!(identity.suggested_account_id, "123456789_account_test_user");
        assert_eq!(identity.suggested_display_name, "@Test_User");
        assert_eq!(identity.suggested_external_account_id, "telegram:123456789");
    }

    #[test]
    fn wait_password_state_is_not_a_qr_request_state() {
        assert!(!super::state_allows_qr_request(
            "authorizationStateWaitPassword"
        ));
    }

    #[test]
    fn password_waiting_response_does_not_expose_stale_qr_token() {
        let response = super::password_waiting_response(
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
        let identity = super::TelegramQrLoginIdentity {
            user_id: "123456789".to_owned(),
            username: Some("Test_User".to_owned()),
            suggested_account_id: "123456789_account_test_user".to_owned(),
            suggested_display_name: "@Test_User".to_owned(),
            suggested_external_account_id: "telegram:123456789".to_owned(),
        };

        let response = super::ready_response(
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
        let response = super::qr_preparing_response("setup-id", "telegram-account");

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
            TelegramQrLoginSession {
                response: test_qr_login_response(TelegramQrLoginStatus::WaitingPassword),
                command_tx,
                worker_completion: super::new_worker_completion(),
            },
        )])));

        let login_check_value = "tdlib-check-value".to_owned();

        let response = super::submit_qr_login_password(
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
            TelegramQrLoginCommand::CheckPassword(login_check_value)
        );
    }

    #[test]
    fn qr_password_submission_requires_waiting_password_status() {
        let (command_tx, command_rx) = mpsc::channel();
        let pending = Arc::new(Mutex::new(HashMap::from([(
            "setup-id".to_owned(),
            TelegramQrLoginSession {
                response: test_qr_login_response(TelegramQrLoginStatus::WaitingQrScan),
                command_tx,
                worker_completion: super::new_worker_completion(),
            },
        )])));

        let login_check_value = "tdlib-check-value".to_owned();

        let error = super::submit_qr_login_password(
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
        let worker_completion = super::new_worker_completion();
        super::mark_worker_complete(&worker_completion);
        let pending = Arc::new(Mutex::new(HashMap::from([(
            "setup-id".to_owned(),
            TelegramQrLoginSession {
                response: test_qr_login_response(TelegramQrLoginStatus::WaitingQrScan),
                command_tx,
                worker_completion,
            },
        )])));

        super::cancel_qr_login(Arc::clone(&pending), "setup-id").expect("QR login cancelled");

        assert!(
            !pending
                .lock()
                .expect("pending lock")
                .contains_key("setup-id")
        );
        assert_eq!(
            command_rx.try_recv().expect("cancel command"),
            TelegramQrLoginCommand::Cancel
        );
    }

    #[test]
    fn qr_login_cancel_unknown_setup_returns_not_found() {
        let pending = Arc::new(Mutex::new(HashMap::new()));

        let error = super::cancel_qr_login(pending, "missing-setup")
            .expect_err("unknown QR setup must not be cancelled");

        assert!(matches!(error, TelegramError::QrLoginNotFound));
    }

    #[test]
    fn qr_login_start_cancels_existing_sessions_for_same_account() {
        let (same_account_tx, same_account_rx) = mpsc::channel();
        let (other_account_tx, other_account_rx) = mpsc::channel();
        let same_account_completion = super::new_worker_completion();
        let other_account_completion = super::new_worker_completion();
        super::mark_worker_complete(&same_account_completion);
        super::mark_worker_complete(&other_account_completion);
        let mut other_response = test_qr_login_response(TelegramQrLoginStatus::WaitingQrScan);
        other_response.setup_id = "other-setup-id".to_owned();
        other_response.account_id = "other-account".to_owned();
        let pending = Arc::new(Mutex::new(HashMap::from([
            (
                "setup-id".to_owned(),
                TelegramQrLoginSession {
                    response: test_qr_login_response(TelegramQrLoginStatus::WaitingQrScan),
                    command_tx: same_account_tx,
                    worker_completion: same_account_completion,
                },
            ),
            (
                "other-setup-id".to_owned(),
                TelegramQrLoginSession {
                    response: other_response,
                    command_tx: other_account_tx,
                    worker_completion: other_account_completion,
                },
            ),
        ])));

        super::cancel_existing_qr_logins_for_account(&pending, "telegram-account")
            .expect("same-account sessions cancelled");

        let pending = pending.lock().expect("pending lock");
        assert!(!pending.contains_key("setup-id"));
        assert!(pending.contains_key("other-setup-id"));
        assert_eq!(
            same_account_rx.try_recv().expect("same account cancel"),
            TelegramQrLoginCommand::Cancel
        );
        assert!(other_account_rx.try_recv().is_err());
    }

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
}
