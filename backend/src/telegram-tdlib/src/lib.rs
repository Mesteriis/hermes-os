//! TDLib adapter boundary. The provider wire is isolated from Telegram policy and storage.

use std::collections::VecDeque;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use hermes_telegram_api::{
    TelegramChat, TelegramChatAvatar, TelegramDownloadFile, TelegramFileSnapshot, TelegramMediaKind,
    TelegramMessageMedia, TelegramMessageObservation, TelegramParticipant, TelegramParticipantFilter,
    TelegramParticipantPage, TelegramProviderCommand, TelegramSendMessage, TelegramProviderEvent,
    TelegramChatFolder, TelegramChatPosition, TelegramForwardOrigin, TelegramMessageReferences,
    TelegramReplyReference, TelegramTopic, TelegramTypingState,
    provider_command_operation_id, validate_provider_command,
};
use hermes_telegram_api::{TelegramChatKind, validate_page_size, validate_text};
use libloading::Library;
use serde_json::{Value, json};
use zeroize::Zeroizing;

pub mod authorization;
pub use authorization::{TdlibAuthorizationDriver, TdlibAuthorizationEvent};

pub const PACKAGE: &str = "hermes-telegram-tdlib";

/// Runtime-owned port for converting an authorized opaque BlobRef into a
/// short-lived TDLib input file. The adapter never reads a filesystem path
/// from the provider contract.
pub trait TelegramMediaMaterializer {
    fn materialize(&mut self, blob_ref: &str) -> Result<String, TdlibError>;
    fn release(&mut self, materialized_path: &str);
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TdlibAuthorizationParameters {
    pub api_id: i64,
    pub api_hash: Zeroizing<String>,
    pub database_directory: PathBuf,
    pub session_encryption_key: Option<Zeroizing<Vec<u8>>>,
}

impl TdlibAuthorizationParameters {
    pub fn from_secret_material(
        api_id: i64,
        api_hash: Zeroizing<Vec<u8>>,
        database_directory: PathBuf,
        session_encryption_key: Option<Zeroizing<Vec<u8>>>,
    ) -> Result<Self, TdlibError> {
        let api_hash = String::from_utf8(api_hash.to_vec())
            .map(Zeroizing::new)
            .map_err(|_| TdlibError::Protocol("Telegram API hash is not UTF-8".to_owned()))?;
        if api_id <= 0 || api_hash.trim().is_empty() {
            return Err(TdlibError::Protocol(
                "Telegram application credentials are invalid".to_owned(),
            ));
        }
        Ok(Self {
            api_id,
            api_hash,
            database_directory,
            session_encryption_key,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TdlibAuthorizationUpdate {
    WaitingParameters,
    WaitingEncryptionKey,
    WaitingQrScan,
    WaitingPassword { hint: Option<String> },
    Ready,
    Closing,
    Closed,
    Error { code: Option<i64>, message: String },
    Other(String),
}

pub fn set_tdlib_parameters_request(
    parameters: &TdlibAuthorizationParameters,
) -> Result<Value, TdlibError> {
    if parameters.api_id <= 0 || parameters.api_hash.trim().is_empty() {
        return Err(TdlibError::Protocol("TDLib application credentials are invalid".to_owned()));
    }
    let database_directory = parameters.database_directory.to_string_lossy().into_owned();
    let files_directory = parameters.database_directory.join("files").to_string_lossy().into_owned();
    let encryption_key = parameters
        .session_encryption_key
        .as_deref()
        .map(|value| STANDARD.encode(value))
        .unwrap_or_default();
    Ok(json!({
        "@type": "setTdlibParameters",
        "parameters": {
            "use_test_dc": false,
            "database_directory": database_directory,
            "files_directory": files_directory,
            "database_encryption_key": encryption_key,
            "use_file_database": true,
            "use_chat_info_database": true,
            "use_message_database": true,
            "use_secret_chats": false,
            "api_id": parameters.api_id,
            "api_hash": parameters.api_hash.as_str(),
            "system_language_code": "en",
            "device_model": "Hermes Hub",
            "system_version": std::env::consts::OS,
            "application_version": env!("CARGO_PKG_VERSION"),
            "enable_storage_optimizer": true,
            "ignore_file_names": false
        },
        "@extra": "hermes-set-tdlib-parameters"
    }))
}

pub fn check_database_encryption_key_request(key: Option<&[u8]>) -> Value {
    json!({
        "@type": "checkDatabaseEncryptionKey",
        "encryption_key": key.map(|value| STANDARD.encode(value)).unwrap_or_default(),
        "@extra": "hermes-check-database-encryption-key"
    })
}

pub fn request_qr_code_authentication() -> Value {
    json!({
        "@type": "requestQrCodeAuthentication",
        "other_user_ids": [],
        "@extra": "hermes-request-qr-code-authentication"
    })
}

pub fn check_authentication_password(password: &str) -> Result<Value, TdlibError> {
    if password.is_empty() {
        return Err(TdlibError::Protocol("Telegram password is empty".to_owned()));
    }
    Ok(json!({
        "@type": "checkAuthenticationPassword",
        "password": password,
        "@extra": "hermes-check-authentication-password"
    }))
}

pub fn close_session_request() -> Value {
    json!({"@type": "close", "@extra": "hermes-close-tdlib-session"})
}

pub fn parse_authorization_update(payload: &Value) -> Result<TdlibAuthorizationUpdate, TdlibError> {
    let state = payload
        .get("authorization_state")
        .unwrap_or(payload)
        .get("@type")
        .and_then(Value::as_str)
        .ok_or_else(|| TdlibError::Protocol("TDLib authorization state is missing".to_owned()))?;
    Ok(match state {
        "authorizationStateWaitTdlibParameters" => TdlibAuthorizationUpdate::WaitingParameters,
        "authorizationStateWaitEncryptionKey" => TdlibAuthorizationUpdate::WaitingEncryptionKey,
        "authorizationStateWaitOtherDeviceConfirmation" => TdlibAuthorizationUpdate::WaitingQrScan,
        "authorizationStateWaitPassword" => TdlibAuthorizationUpdate::WaitingPassword {
            hint: payload.get("password_hint").and_then(Value::as_str).map(ToOwned::to_owned),
        },
        "authorizationStateReady" => TdlibAuthorizationUpdate::Ready,
        "authorizationStateClosing" | "authorizationStateLoggingOut" => TdlibAuthorizationUpdate::Closing,
        "authorizationStateClosed" => TdlibAuthorizationUpdate::Closed,
        "error" => TdlibAuthorizationUpdate::Error {
            code: payload.get("code").and_then(Value::as_i64),
            message: payload.get("message").and_then(Value::as_str).unwrap_or("TDLib error").to_owned(),
        },
        other => TdlibAuthorizationUpdate::Other(other.to_owned()),
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TdlibRequest {
    LoadChats { account_id: String, limit: u32 },
    LoadHistory {
        account_id: String,
        provider_chat_id: String,
        from_message_id: Option<i64>,
        mode: hermes_telegram_api::TelegramHistorySyncMode,
        limit: u32,
    },
    SendMessage(TelegramSendMessage),
    SendMedia(hermes_telegram_api::TelegramSendMedia),
    SendMediaMaterialized {
        command: hermes_telegram_api::TelegramSendMedia,
        materialized_path: String,
    },
    DownloadFile(TelegramDownloadFile),
    ListParticipants {
        account_id: String,
        provider_chat_id: String,
        filter: TelegramParticipantFilter,
        offset: u32,
        limit: u32,
    },
    ListBasicGroupParticipants {
        account_id: String,
        provider_chat_id: String,
        basic_group_id: i64,
    },
    ListTopics {
        account_id: String,
        provider_chat_id: String,
        limit: u32,
    },
    GetChatFolder {
        account_id: String,
        provider_folder_id: i64,
    },
    ProviderCommand(TelegramProviderCommand),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TdlibResponse {
    Chats(Vec<TelegramChat>),
    History(Vec<TelegramMessageObservation>),
    Sent { provider_message_id: String },
    File(TelegramFileSnapshot),
    Participants(TelegramParticipantPage),
    Topics(Vec<TelegramTopic>),
    ChatFolders(Vec<TelegramChatFolder>),
    Accepted { operation_id: String },
}

pub fn get_chats_request(account_id: &str, limit: u32) -> Result<TdlibRequest, TdlibError> {
    validate_page_size(limit).map_err(|_| TdlibError::Protocol("invalid chat page size".to_owned()))?;
    if account_id.trim().is_empty() {
        return Err(TdlibError::Protocol("account id is empty".to_owned()));
    }
    Ok(TdlibRequest::LoadChats {
        account_id: account_id.to_owned(),
        limit,
    })
}

pub fn get_history_request(
    account_id: &str,
    provider_chat_id: &str,
    limit: u32,
) -> Result<TdlibRequest, TdlibError> {
    get_history_request_with_options(
        account_id,
        provider_chat_id,
        None,
        hermes_telegram_api::TelegramHistorySyncMode::Latest,
        limit,
    )
}

pub fn get_history_request_with_options(
    account_id: &str,
    provider_chat_id: &str,
    from_message_id: Option<i64>,
    mode: hermes_telegram_api::TelegramHistorySyncMode,
    limit: u32,
) -> Result<TdlibRequest, TdlibError> {
    validate_page_size(limit).map_err(|_| TdlibError::Protocol("invalid history page size".to_owned()))?;
    provider_chat_id
        .parse::<i64>()
        .map_err(|_| TdlibError::Protocol("provider chat id is not an integer".to_owned()))?;
    if account_id.trim().is_empty() {
        return Err(TdlibError::Protocol("account id is empty".to_owned()));
    }
    if from_message_id.is_some_and(|message_id| message_id <= 0)
        || (matches!(mode, hermes_telegram_api::TelegramHistorySyncMode::Older)
            && from_message_id.is_none())
    {
        return Err(TdlibError::Protocol("history cursor is invalid".to_owned()));
    }
    Ok(TdlibRequest::LoadHistory {
        account_id: account_id.to_owned(),
        provider_chat_id: provider_chat_id.to_owned(),
        from_message_id,
        mode,
        limit,
    })
}

pub fn send_message_request(command: TelegramSendMessage) -> Result<TdlibRequest, TdlibError> {
    validate_text(&command.text)
        .map_err(|_| TdlibError::Protocol("message text is invalid".to_owned()))?;
    command
        .provider_chat_id
        .parse::<i64>()
        .map_err(|_| TdlibError::Protocol("provider chat id is not an integer".to_owned()))?;
    Ok(TdlibRequest::SendMessage(command))
}

pub fn encode_request(request: &TdlibRequest) -> Result<Value, TdlibError> {
    match request {
        TdlibRequest::LoadChats { account_id, limit } => Ok(json!({
            "@type": "getChats",
            "chat_list": {"@type": "chatListMain"},
            "limit": limit,
            "@extra": account_id,
        })),
        TdlibRequest::LoadHistory {
            account_id,
            provider_chat_id,
            from_message_id,
            limit,
            ..
        } => Ok(json!({
            "@type": "getChatHistory",
            "chat_id": provider_chat_id.parse::<i64>().map_err(|_| TdlibError::Protocol("provider chat id is not an integer".to_owned()))?,
            "from_message_id": from_message_id.unwrap_or_default(),
            "offset": 0,
            "limit": limit,
            "only_local": false,
            "@extra": account_id,
        })),
        TdlibRequest::SendMessage(command) => Ok(json!({
            "@type": "sendMessage",
            "chat_id": command.provider_chat_id.parse::<i64>().map_err(|_| TdlibError::Protocol("provider chat id is not an integer".to_owned()))?,
            "reply_to": {"@type": "inputMessageReplyToMessage", "message_id": 0},
            "options": null,
            "reply_markup": null,
            "input_message_content": {
                "@type": "inputMessageText",
                "text": {"@type": "formattedText", "text": command.text},
                "clear_draft": false,
                "link_preview_options": null,
            },
            "@extra": command.operation_id,
        })),
        TdlibRequest::SendMedia(_) => Err(TdlibError::Protocol(
            "Telegram media request requires an authorized Blob materializer".to_owned(),
        )),
        TdlibRequest::SendMediaMaterialized { command, materialized_path } => {
            encode_send_media_materialized(command, materialized_path)
        }
        TdlibRequest::DownloadFile(command) => Ok(json!({
            "@type": "downloadFile",
            "file_id": provider_id(&command.provider_file_id)?,
            "priority": command.priority,
            "offset": 0,
            "limit": 0,
            "synchronous": false,
            "@extra": command.operation_id,
        })),
        TdlibRequest::ListParticipants {
            account_id,
            provider_chat_id,
            filter,
            offset,
            limit,
        } => Ok(json!({
            "@type": "getSupergroupMembers",
            "supergroup_id": provider_id(provider_chat_id)?,
            "filter": {"@type": match filter {
                TelegramParticipantFilter::Recent => "chatMembersFilterMembers",
                TelegramParticipantFilter::Administrators => "chatMembersFilterAdministrators",
            }},
            "offset": offset,
            "limit": limit,
            "@extra": account_id,
        })),
        TdlibRequest::ListBasicGroupParticipants {
            account_id,
            basic_group_id,
            ..
        } => Ok(json!({
            "@type": "getBasicGroup",
            "basic_group_id": basic_group_id,
            "@extra": account_id,
        })),
        TdlibRequest::ListTopics {
            account_id,
            provider_chat_id,
            limit,
        } => Ok(json!({
            "@type": "getForumTopics",
            "chat_id": provider_id(provider_chat_id)?,
            "query": "",
            "offset_date": 0,
            "offset_message_id": 0,
            "limit": limit,
            "@extra": account_id,
        })),
        TdlibRequest::GetChatFolder {
            account_id,
            provider_folder_id,
        } => Ok(json!({
            "@type": "getChatFolder",
            "chat_folder_id": provider_folder_id,
            "@extra": format!("{account_id}:folder:{provider_folder_id}"),
        })),
        TdlibRequest::ProviderCommand(command) => encode_provider_command(command),
    }
}

pub fn encode_provider_command(command: &TelegramProviderCommand) -> Result<Value, TdlibError> {
    validate_provider_command(command)
        .map_err(|_| TdlibError::Protocol("Telegram provider command is invalid".to_owned()))?;
    match command {
        TelegramProviderCommand::SendText(command) => encode_request(&TdlibRequest::SendMessage(command.clone())),
        TelegramProviderCommand::SendMedia(_) => Err(TdlibError::Protocol(
            "Telegram media command requires an authorized Blob materializer".to_owned(),
        )),
        TelegramProviderCommand::DownloadFile(command) => encode_request(&TdlibRequest::DownloadFile(command.clone())),
        TelegramProviderCommand::ListTopics { operation_id, account_id, provider_chat_id, limit } => {
            let request = TdlibRequest::ListTopics {
                account_id: account_id.clone(),
                provider_chat_id: provider_chat_id.clone(),
                limit: *limit,
            };
            let mut encoded = encode_request(&request)?;
            encoded["@extra"] = json!(operation_id);
            Ok(encoded)
        }
        TelegramProviderCommand::CreateTopic { operation_id, provider_chat_id, title, .. } => Ok(json!({
            "@type": "createForumTopic",
            "chat_id": provider_id(provider_chat_id)?,
            "name": title,
            "icon": {"@type": "messageForumTopicIcon", "color": 7322096, "custom_emoji_id": ""},
            "@extra": operation_id,
        })),
        TelegramProviderCommand::SetTopicClosed { operation_id, provider_chat_id, provider_topic_id, is_closed, .. } => Ok(json!({
            "@type": "toggleForumTopicIsClosed",
            "chat_id": provider_id(provider_chat_id)?,
            "message_thread_id": provider_id(provider_topic_id)?,
            "is_closed": is_closed,
            "@extra": operation_id,
        })),
        TelegramProviderCommand::Reply {
            operation_id,
            provider_chat_id,
            reply_to_provider_message_id,
            text,
            ..
        } => Ok(text_command(
            "sendMessage",
            provider_chat_id,
            Some(reply_to_provider_message_id),
            text,
            operation_id,
        )?),
        TelegramProviderCommand::Forward {
            operation_id,
            provider_chat_id,
            from_provider_chat_id,
            from_provider_message_id,
            ..
        } => Ok(json!({
            "@type": "forwardMessages",
            "chat_id": provider_id(provider_chat_id)?,
            "from_chat_id": provider_id(from_provider_chat_id)?,
            "message_ids": [provider_id(from_provider_message_id)?],
            "options": null,
            "send_copy": false,
            "remove_caption": false,
            "@extra": operation_id,
        })),
        TelegramProviderCommand::Edit {
            operation_id,
            provider_chat_id,
            provider_message_id,
            text,
            ..
        } => Ok(json!({
            "@type": "editMessageText",
            "chat_id": provider_id(provider_chat_id)?,
            "message_id": provider_id(provider_message_id)?,
            "input_message_content": formatted_text_content(text, false),
            "@extra": operation_id,
        })),
        TelegramProviderCommand::Delete {
            operation_id,
            provider_chat_id,
            provider_message_id,
            revoke,
            ..
        } => Ok(json!({
            "@type": "deleteMessages",
            "chat_id": provider_id(provider_chat_id)?,
            "message_ids": [provider_id(provider_message_id)?],
            "revoke": revoke,
            "@extra": operation_id,
        })),
        TelegramProviderCommand::RestoreVisibility { .. } => Err(TdlibError::Protocol(
            "Telegram restore visibility is local-only".to_owned(),
        )),
        TelegramProviderCommand::Reaction {
            operation_id,
            provider_chat_id,
            provider_message_id,
            emoji,
            active,
            ..
        } => Ok(json!({
            "@type": if *active { "addMessageReaction" } else { "removeMessageReaction" },
            "chat_id": provider_id(provider_chat_id)?,
            "message_id": provider_id(provider_message_id)?,
            "reaction_type": {"@type": "reactionTypeEmoji", "emoji": emoji},
            "is_big": false,
            "update_recent_reactions": true,
            "@extra": operation_id,
        })),
        TelegramProviderCommand::Pin {
            operation_id,
            provider_chat_id,
            provider_message_id,
            active,
            ..
        } => Ok(if *active {
            json!({
                "@type": "pinChatMessage", "chat_id": provider_id(provider_chat_id)?,
                "message_id": provider_id(provider_message_id)?, "disable_notification": false,
                "only_for_self": false, "@extra": operation_id
            })
        } else {
            json!({
                "@type": "unpinChatMessage", "chat_id": provider_id(provider_chat_id)?,
                "message_id": provider_id(provider_message_id)?, "@extra": operation_id
            })
        }),
        TelegramProviderCommand::MarkUnread {
            operation_id,
            provider_chat_id,
            unread,
            read_through_provider_message_id,
            ..
        } => {
            if !unread {
                if let Some(message_id) = read_through_provider_message_id {
                    return Ok(json!({
                        "@type": "viewMessages", "chat_id": provider_id(provider_chat_id)?,
                        "message_ids": [provider_id(message_id)?], "source": null,
                        "force_read": true, "@extra": operation_id
                    }));
                }
            }
            Ok(json!({
                "@type": "toggleChatIsMarkedAsUnread", "chat_id": provider_id(provider_chat_id)?,
                "is_marked_as_unread": unread, "@extra": operation_id
            }))
        }
        TelegramProviderCommand::Archive { operation_id, provider_chat_id, archived, .. } => Ok(json!({
            "@type": "addChatToList", "chat_id": provider_id(provider_chat_id)?,
            "chat_list": {"@type": if *archived { "chatListArchive" } else { "chatListMain" }},
            "@extra": operation_id
        })),
        TelegramProviderCommand::Mute { operation_id, provider_chat_id, muted, .. } => Ok(json!({
            "@type": "setChatNotificationSettings", "chat_id": provider_id(provider_chat_id)?,
            "notification_settings": {"@type": "chatNotificationSettings", "use_default_mute_for": !muted,
                "mute_for": if *muted { 31_708_800 } else { 0 }, "use_default_sound": true,
                "sound_id": 0, "use_default_show_preview": true, "show_preview": true,
                "use_default_mute_stories": true, "mute_stories": false,
                "use_default_story_sound": true, "story_sound_id": 0,
                "use_default_show_story_poster": true, "show_story_poster": true,
                "use_default_disable_pinned_message_notifications": true,
                "disable_pinned_message_notifications": false,
                "use_default_disable_mention_notifications": true,
                "disable_mention_notifications": false},
            "@extra": operation_id
        })),
        TelegramProviderCommand::Join { operation_id, provider_chat_id, .. } => Ok(json!({
            "@type": "joinChat", "chat_id": provider_id(provider_chat_id)?, "@extra": operation_id
        })),
        TelegramProviderCommand::Leave { operation_id, provider_chat_id, .. } => Ok(json!({
            "@type": "leaveChat", "chat_id": provider_id(provider_chat_id)?, "@extra": operation_id
        })),
        TelegramProviderCommand::AddChatToFolder { operation_id, provider_chat_id, provider_folder_id, .. } => Ok(json!({
            "@type": "addChatToList",
            "chat_id": provider_id(provider_chat_id)?,
            "chat_list": {"@type": "chatListFolder", "chat_folder_id": provider_folder_id},
            "@extra": operation_id
        })),
        TelegramProviderCommand::RemoveChatFromFolder { operation_id, provider_folder_id, .. } => Ok(json!({
            "@type": "getChatFolder",
            "chat_folder_id": provider_folder_id,
            "@extra": format!("{operation_id}:get")
        })),
        TelegramProviderCommand::SearchMessages { operation_id, provider_chat_id, query, limit, .. } => {
            let query = query.trim();
            if let Some(chat_id) = provider_chat_id {
                Ok(json!({
                    "@type": "searchChatMessages", "chat_id": provider_id(chat_id)?,
                    "query": query, "sender_id": null, "from_message_id": 0, "offset": 0,
                    "limit": limit, "filter": {"@type": "searchMessagesFilterEmpty"},
                    "@extra": operation_id
                }))
            } else {
                Ok(json!({
                    "@type": "searchMessages", "chat_list": {"@type": "chatListMain"},
                    "query": query, "offset_date": 0, "offset_chat_id": 0,
                    "offset_message_id": 0, "limit": limit,
                    "filter": {"@type": "searchMessagesFilterEmpty"}, "@extra": operation_id
                }))
            }
        }
        TelegramProviderCommand::ListParticipants { operation_id, account_id, provider_chat_id, filter, offset, limit } => {
            let request = TdlibRequest::ListParticipants {
                account_id: account_id.clone(),
                provider_chat_id: provider_chat_id.clone(),
                filter: *filter,
                offset: *offset,
                limit: *limit,
            };
            let mut encoded = encode_request(&request)?;
            encoded["@extra"] = json!(operation_id);
            Ok(encoded)
        }
    }
}

pub fn encode_send_media_materialized(
    command: &hermes_telegram_api::TelegramSendMedia,
    materialized_path: &str,
) -> Result<Value, TdlibError> {
    let input_type = match command.media_kind {
        TelegramMediaKind::Photo => "inputMessagePhoto",
        TelegramMediaKind::Video => "inputMessageVideo",
        TelegramMediaKind::Audio => "inputMessageAudio",
        TelegramMediaKind::Document => "inputMessageDocument",
        TelegramMediaKind::Animation => "inputMessageAnimation",
        TelegramMediaKind::VoiceNote => "inputMessageVoiceNote",
    };
    let file_field = match command.media_kind {
        TelegramMediaKind::Photo | TelegramMediaKind::Video | TelegramMediaKind::Animation => "photo",
        TelegramMediaKind::Audio => "audio",
        TelegramMediaKind::Document => "document",
        TelegramMediaKind::VoiceNote => "voice_note",
    };
    let caption = command.caption.as_deref().unwrap_or("");
    let mut content = json!({
        "@type": input_type,
        "caption": {"@type": "formattedText", "text": caption, "entities": []},
    });
    if materialized_path.trim().is_empty() {
        return Err(TdlibError::Protocol("Telegram media materialization path is empty".to_owned()));
    }
    content[file_field] = json!({"@type": "inputFileLocal", "path": materialized_path});
    Ok(json!({
        "@type": "sendMessage",
        "chat_id": provider_id(&command.provider_chat_id)?,
        "reply_to": {"@type": "inputMessageReplyToMessage", "message_id": 0},
        "options": null,
        "reply_markup": null,
        "input_message_content": content,
        "@extra": command.operation_id,
    }))
}

fn text_command(
    command_type: &str,
    provider_chat_id: &str,
    reply_to_provider_message_id: Option<&str>,
    text: &str,
    operation_id: &str,
) -> Result<Value, TdlibError> {
    let mut request = json!({
        "@type": command_type,
        "chat_id": provider_id(provider_chat_id)?,
        "input_message_content": formatted_text_content(text, true),
        "@extra": operation_id,
    });
    if let Some(message_id) = reply_to_provider_message_id {
        request["reply_to"] = json!({
            "@type": "inputMessageReplyToMessage",
            "message_id": provider_id(message_id)?,
        });
    }
    Ok(request)
}

fn formatted_text_content(text: &str, clear_draft: bool) -> Value {
    json!({
        "@type": "inputMessageText",
        "text": {"@type": "formattedText", "text": text, "entities": []},
        "clear_draft": clear_draft,
        "link_preview_options": null,
    })
}

fn provider_id(value: &str) -> Result<i64, TdlibError> {
    value
        .parse::<i64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| TdlibError::Protocol("Telegram provider id is invalid".to_owned()))
}

pub fn parse_chat(account_id: &str, payload: &Value) -> Result<TelegramChat, TdlibError> {
    let provider_chat_id = required_string(payload, "id")?;
    let title = required_string(payload, "title")?;
    let kind = payload
        .get("type")
        .and_then(|value| value.get("@type"))
        .and_then(Value::as_str)
        .map(chat_kind)
        .ok_or_else(|| TdlibError::Protocol("TDLib chat type is missing".to_owned()))??;
    let username = payload
        .get("usernames")
        .and_then(|value| value.get("editable_username"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    Ok(TelegramChat {
        account_id: account_id.to_owned(),
        provider_chat_id,
        kind,
        title,
        username,
    })
}

fn parse_chat_avatar(account_id: &str, payload: &Value) -> Result<TelegramChatAvatar, TdlibError> {
    let photo = payload.get("photo").filter(|value| !value.is_null());
    let file = photo
        .and_then(|value| value.get("small").or_else(|| value.get("big")));
    Ok(TelegramChatAvatar {
        account_id: account_id.to_owned(),
        provider_chat_id: required_string(payload, "chat_id")?,
        provider_file_id: file
            .and_then(|value| value.get("id"))
            .and_then(value_id_optional),
        provider_unique_id: file
            .and_then(|value| value.get("remote"))
            .and_then(|value| value.get("unique_id"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
    })
}

pub fn parse_message_observation(
    account_id: &str,
    payload: &Value,
) -> Result<TelegramMessageObservation, TdlibError> {
    let provider_chat_id = required_string(payload, "chat_id")?;
    let provider_message_id = required_string(payload, "id")?;
    let sender = payload
        .get("sender_id")
        .and_then(|value| value.get("user_id").or_else(|| value.get("chat_id")))
        .and_then(Value::as_i64)
        .ok_or_else(|| TdlibError::Protocol("TDLib message sender is missing".to_owned()))?;
    let text = message_text(payload.get("content"));
    let media = parse_message_media(payload.get("content"));
    let references = parse_message_references(payload)?;
    Ok(TelegramMessageObservation {
        account_id: account_id.to_owned(),
        provider_chat_id,
        provider_message_id,
        provider_topic_id: payload
            .get("message_thread_id")
            .and_then(value_id_optional),
        sender_id: sender.to_string(),
        sender_display_name: None,
        text,
        media,
        references,
        observed_at_unix_seconds: payload.get("date").and_then(Value::as_i64).unwrap_or_default(),
    })
}

fn parse_message_references(payload: &Value) -> Result<TelegramMessageReferences, TdlibError> {
    let reply_to = match payload.get("reply_to") {
        None => None,
        Some(reply) => {
            if reply.get("@type").and_then(Value::as_str) != Some("messageReplyToMessage") {
                return Err(TdlibError::Protocol("TDLib reply reference type is unsupported".to_owned()));
            }
            Some(TelegramReplyReference {
                provider_chat_id: required_string(reply, "chat_id")?,
                provider_message_id: required_string(reply, "message_id")?,
            })
        }
    };
    let forward_origin = match payload.get("forward_info") {
        None => None,
        Some(forward_info) => {
            let origin = forward_info
                .get("origin")
                .ok_or_else(|| TdlibError::Protocol("TDLib forward origin is missing".to_owned()))?;
            let origin_type = origin
                .get("@type")
                .and_then(Value::as_str)
                .ok_or_else(|| TdlibError::Protocol("TDLib forward origin type is missing".to_owned()))?;
            let provider_sender_id = match origin_type {
                "messageOriginUser" => Some(value_id(origin.get("sender_user_id").ok_or_else(|| {
                    TdlibError::Protocol("TDLib forward user origin is missing".to_owned())
                })?)?),
                "messageOriginChat" => Some(value_id(origin.get("sender_chat_id").ok_or_else(|| {
                    TdlibError::Protocol("TDLib forward chat origin is missing".to_owned())
                })?)?),
                "messageOriginHiddenUser" | "messageOriginMessageImport" => None,
                other => {
                    return Err(TdlibError::Protocol(format!(
                        "TDLib forward origin type is unsupported: {other}"
                    )))
                }
            };
            let sender_name = origin
                .get("sender_name")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let provider_chat_id = forward_info
                .get("chat_id")
                .map(value_id)
                .transpose()?;
            let provider_message_id = forward_info
                .get("message_id")
                .map(value_id)
                .transpose()?;
            Some(TelegramForwardOrigin {
                provider_chat_id,
                provider_message_id,
                provider_sender_id,
                sender_name,
                observed_at_unix_seconds: forward_info.get("date").and_then(Value::as_i64),
            })
        }
    };
    Ok(TelegramMessageReferences { reply_to, forward_origin })
}

pub fn parse_file_snapshot(
    account_id: &str,
    payload: &Value,
) -> Result<TelegramFileSnapshot, TdlibError> {
    if payload.get("@type").and_then(Value::as_str) != Some("file") {
        return Err(TdlibError::Protocol("TDLib file payload is invalid".to_owned()));
    }
    let provider_file_id = required_string(payload, "id")?;
    let local = payload.get("local");
    let remote = payload.get("remote");
    Ok(TelegramFileSnapshot {
        account_id: account_id.to_owned(),
        provider_file_id,
        provider_unique_id: remote
            .and_then(|value| value.get("unique_id"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        media_kind: None,
        size_bytes: integer_field(payload, "size"),
        expected_size_bytes: integer_field(payload, "expected_size"),
        downloaded_size_bytes: local.and_then(|value| integer_field(value, "downloaded_size")),
        is_downloading: local
            .and_then(|value| value.get("is_downloading_active"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        is_downloaded: local
            .and_then(|value| value.get("is_downloading_completed"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

pub fn parse_participant_page(
    account_id: &str,
    provider_chat_id: &str,
    filter: TelegramParticipantFilter,
    offset: u32,
    payload: &Value,
) -> Result<TelegramParticipantPage, TdlibError> {
    let members = payload
        .get("members")
        .and_then(Value::as_array)
        .ok_or_else(|| TdlibError::Protocol("TDLib participant list is missing members".to_owned()))?;
    let items = members
        .iter()
        .map(|member| {
            let member_id = member
                .get("member_id")
                .ok_or_else(|| TdlibError::Protocol("TDLib participant id is missing".to_owned()))?;
            let (member_kind, member_id) = if let Some(user_id) = member_id.get("user_id") {
                ("user", value_id(user_id)?)
            } else if let Some(chat_id) = member_id.get("chat_id") {
                ("chat", value_id(chat_id)?)
            } else {
                return Err(TdlibError::Protocol("TDLib participant sender kind is unsupported".to_owned()));
            };
            let status_kind = member
                .get("status")
                .and_then(|value| value.get("@type"))
                .and_then(Value::as_str)
                .unwrap_or("chatMemberStatusUnknown")
                .to_owned();
            let is_admin = matches!(status_kind.as_str(), "chatMemberStatusAdministrator" | "chatMemberStatusCreator");
            let is_owner = status_kind == "chatMemberStatusCreator";
            Ok(TelegramParticipant {
                account_id: account_id.to_owned(),
                provider_chat_id: provider_chat_id.to_owned(),
                provider_member_id: format!("{member_kind}:{member_id}"),
                display_name: optional_trimmed(member.get("display_name")),
                username: optional_trimmed(member.get("username")),
                role: participant_role(&status_kind).to_owned(),
                status: status_kind
                    .strip_prefix("chatMemberStatus")
                    .unwrap_or(&status_kind)
                    .to_lowercase(),
                is_admin,
                is_owner,
                permissions: participant_permissions(member.get("status")),
            })
        })
        .collect::<Result<Vec<_>, TdlibError>>()?;
    let next_offset = (!items.is_empty()).then_some(offset + items.len() as u32);
    Ok(TelegramParticipantPage {
        account_id: account_id.to_owned(),
        provider_chat_id: provider_chat_id.to_owned(),
        filter,
        items,
        next_offset,
    })
}

pub fn parse_topic_list(
    account_id: &str,
    provider_chat_id: &str,
    payload: &Value,
) -> Result<Vec<TelegramTopic>, TdlibError> {
    payload
        .get("topics")
        .and_then(Value::as_array)
        .ok_or_else(|| TdlibError::Protocol("TDLib topic list is missing topics".to_owned()))?
        .iter()
        .map(|topic| {
            let info = topic.get("info").unwrap_or(topic);
            let mut parsed = parse_topic_info(account_id, provider_chat_id, info)?;
            parsed.unread_count = topic
                .get("unread_count")
                .and_then(Value::as_i64)
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or_default();
            parsed.last_message_at_unix_seconds = topic
                .get("last_message")
                .and_then(|message| message.get("date"))
                .and_then(Value::as_i64);
            Ok(parsed)
        })
        .collect()
}

fn parse_topic_info(
    account_id: &str,
    provider_chat_id: &str,
    info: &Value,
) -> Result<TelegramTopic, TdlibError> {
    let provider_topic_id = required_string(info, "message_thread_id")?;
    Ok(TelegramTopic {
        account_id: account_id.to_owned(),
        provider_chat_id: provider_chat_id.to_owned(),
        provider_topic_id,
        title: info
            .get("name")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("Telegram topic")
            .to_owned(),
        icon_emoji: info
            .get("icon")
            .and_then(|icon| icon.get("custom_emoji_id"))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty() && *value != "0")
            .map(ToOwned::to_owned),
        is_pinned: info.get("is_pinned").and_then(Value::as_bool).unwrap_or(false),
        is_closed: info.get("is_closed").and_then(Value::as_bool).unwrap_or(false),
        unread_count: 0,
        last_message_at_unix_seconds: None,
    })
}

fn parse_typing_state(account_id: &str, payload: &Value) -> Result<TelegramTypingState, TdlibError> {
    let sender = payload
        .get("sender_id")
        .ok_or_else(|| TdlibError::Protocol("TDLib typing sender is missing".to_owned()))?;
    let sender_id = match sender.get("@type").and_then(Value::as_str) {
        Some("messageSenderUser") => format!("user:{}", required_string(sender, "user_id")?),
        Some("messageSenderChat") => format!("chat:{}", required_string(sender, "chat_id")?),
        _ => return Err(TdlibError::Protocol("TDLib typing sender kind is unsupported".to_owned())),
    };
    let action = payload
        .get("action")
        .and_then(|value| value.get("@type"))
        .and_then(Value::as_str)
        .ok_or_else(|| TdlibError::Protocol("TDLib typing action is missing".to_owned()))?;
    Ok(TelegramTypingState {
        account_id: account_id.to_owned(),
        provider_chat_id: required_string(payload, "chat_id")?,
        provider_thread_id: payload.get("message_thread_id").and_then(value_id_optional),
        sender_id,
        action: action.to_owned(),
        is_active: action != "chatActionCancel",
    })
}

fn parse_chat_position(account_id: &str, payload: &Value) -> Result<TelegramChatPosition, TdlibError> {
    let position = payload
        .get("position")
        .ok_or_else(|| TdlibError::Protocol("TDLib chat position is missing".to_owned()))?;
    let list = position
        .get("list")
        .ok_or_else(|| TdlibError::Protocol("TDLib chat position list is missing".to_owned()))?;
    let list_type = list
        .get("@type")
        .and_then(Value::as_str)
        .ok_or_else(|| TdlibError::Protocol("TDLib chat list type is missing".to_owned()))?;
    let (list_kind, provider_folder_id) = match list_type {
        "chatListMain" => ("main".to_owned(), None),
        "chatListArchive" => ("archive".to_owned(), None),
        "chatListFolder" => (
            "folder".to_owned(),
            list.get("chat_folder_id").and_then(Value::as_i64),
        ),
        _ => return Err(TdlibError::Protocol("unsupported Telegram chat list".to_owned())),
    };
    Ok(TelegramChatPosition {
        account_id: account_id.to_owned(),
        provider_chat_id: required_string(payload, "chat_id")?,
        list_kind,
        provider_folder_id,
        order: position.get("order").and_then(Value::as_i64).unwrap_or_default(),
        is_pinned: position.get("is_pinned").and_then(Value::as_bool).unwrap_or(false),
    })
}

fn parse_chat_folders(account_id: &str, payload: &Value) -> Result<Vec<TelegramChatFolder>, TdlibError> {
    payload
        .get("chat_folders")
        .and_then(Value::as_array)
        .ok_or_else(|| TdlibError::Protocol("TDLib chat folders are missing".to_owned()))?
        .iter()
        .map(|folder| {
            Ok(TelegramChatFolder {
                account_id: account_id.to_owned(),
                provider_folder_id: required_string(folder, "id")?.parse().map_err(|_| TdlibError::Protocol("Telegram folder id is invalid".to_owned()))?,
                title: folder
                    .get("name")
                    .and_then(|name| name.get("text"))
                    .and_then(Value::as_str)
                    .unwrap_or("Telegram folder")
                    .to_owned(),
                icon_name: folder
                    .get("icon")
                    .and_then(|icon| icon.get("name"))
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                color_id: folder.get("color_id").and_then(Value::as_i64),
                pinned_chat_ids: folder_id_list(folder, "pinned_chat_ids")?,
                included_chat_ids: folder_id_list(folder, "included_chat_ids")?,
                excluded_chat_ids: folder_id_list(folder, "excluded_chat_ids")?,
            })
        })
        .collect()
}

fn folder_id_list(folder: &Value, field: &str) -> Result<Vec<String>, TdlibError> {
    let Some(values) = folder.get(field).and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    values
        .iter()
        .map(|value| {
            value
                .as_i64()
                .map(|id| id.to_string())
                .or_else(|| value.as_str().map(ToOwned::to_owned))
                .ok_or_else(|| TdlibError::Protocol("Telegram folder chat id is invalid".to_owned()))
        })
        .collect()
}

pub fn parse_provider_events(
    account_id: &str,
    payload: &Value,
) -> Result<Vec<TelegramProviderEvent>, TdlibError> {
    let event_type = payload
        .get("@type")
        .and_then(Value::as_str)
        .ok_or_else(|| TdlibError::Protocol("TDLib update type is missing".to_owned()))?;
    let event = match event_type {
        "updateNewMessage" => {
            let message = payload
                .get("message")
                .ok_or_else(|| TdlibError::Protocol("updateNewMessage has no message".to_owned()))?;
            TelegramProviderEvent::MessageCreated(parse_message_observation(account_id, message)?)
        }
        "updateUserChatAction" => TelegramProviderEvent::TypingChanged(parse_typing_state(account_id, payload)?),
        "updateForumTopicInfo" => {
            let chat_id = required_string(payload, "chat_id")?;
            let info = payload
                .get("info")
                .ok_or_else(|| TdlibError::Protocol("updateForumTopicInfo has no info".to_owned()))?;
            TelegramProviderEvent::TopicChanged(parse_topic_info(account_id, &chat_id, info)?)
        }
        "updateMessageSendFailed" => TelegramProviderEvent::MessageSendFailed {
            account_id: account_id.to_owned(),
            provider_chat_id: required_string(payload, "chat_id")?,
            old_provider_message_id: required_string(payload, "old_message_id")?,
            error_code: payload
                .get("error")
                .and_then(|error| error.get("code"))
                .and_then(Value::as_i64),
        },
        "updateMessageSendSucceeded" => {
            let message = payload
                .get("message")
                .ok_or_else(|| TdlibError::Protocol("updateMessageSendSucceeded has no message".to_owned()))?;
            TelegramProviderEvent::MessageSendSucceeded {
                account_id: account_id.to_owned(),
                provider_chat_id: required_string(payload, "chat_id")?,
                old_provider_message_id: required_string(payload, "old_message_id")?,
                provider_message_id: required_string(message, "id")?,
            }
        }
        "updateChatPosition" => TelegramProviderEvent::ChatPositionChanged(parse_chat_position(account_id, payload)?),
        "updateChatFolders" => TelegramProviderEvent::ChatFoldersChanged {
            account_id: account_id.to_owned(),
            folders: parse_chat_folders(account_id, payload)?,
        },
        "updateChatNotificationSettings" => {
            let settings = payload
                .get("notification_settings")
                .ok_or_else(|| TdlibError::Protocol("TDLib notification settings are missing".to_owned()))?;
            TelegramProviderEvent::ChatNotificationChanged {
                account_id: account_id.to_owned(),
                provider_chat_id: required_string(payload, "chat_id")?,
                use_default_mute_for: settings
                    .get("use_default_mute_for")
                    .and_then(Value::as_bool)
                    .unwrap_or(true),
                mute_for_seconds: settings.get("mute_for").and_then(Value::as_i64).unwrap_or_default(),
            }
        }
        "updateChatPhoto" => TelegramProviderEvent::ChatAvatarChanged(parse_chat_avatar(account_id, payload)?),
        "updateChatMember" => {
            let provider_chat_id = required_string(payload, "chat_id")?;
            let member = payload
                .get("new_chat_member")
                .ok_or_else(|| TdlibError::Protocol("updateChatMember has no new_chat_member".to_owned()))?;
            let page = parse_participant_page(
                account_id,
                &provider_chat_id,
                TelegramParticipantFilter::Recent,
                0,
                &json!({"members": [member]}),
            )?;
            let participant = page
                .items
                .into_iter()
                .next()
                .ok_or_else(|| TdlibError::Protocol("updateChatMember has no participant".to_owned()))?;
            TelegramProviderEvent::ParticipantChanged(participant)
        }
        "updateFile" => TelegramProviderEvent::FileChanged(parse_file_snapshot(
            account_id,
            payload.get("file").ok_or_else(|| TdlibError::Protocol("updateFile has no file".to_owned()))?,
        )?),
        "updateMessageContent" => TelegramProviderEvent::MessageEdited {
            account_id: account_id.to_owned(),
            provider_chat_id: required_string(payload, "chat_id")?,
            provider_message_id: required_string(payload, "message_id")?,
            text: message_text(payload.get("new_content")),
            observed_at_unix_seconds: 0,
        },
        "updateMessageEdited" => TelegramProviderEvent::MessageEdited {
            account_id: account_id.to_owned(),
            provider_chat_id: required_string(payload, "chat_id")?,
            provider_message_id: required_string(payload, "message_id")?,
            text: None,
            observed_at_unix_seconds: payload
                .get("edit_date")
                .and_then(Value::as_i64)
                .unwrap_or_default(),
        },
        "updateMessageIsPinned" => TelegramProviderEvent::MessagePinned {
            account_id: account_id.to_owned(),
            provider_chat_id: required_string(payload, "chat_id")?,
            provider_message_id: required_string(payload, "message_id")?,
            is_pinned: payload
                .get("is_pinned")
                .and_then(Value::as_bool)
                .ok_or_else(|| TdlibError::Protocol("pinned state is missing".to_owned()))?,
        },
        "updateMessageInteractionInfo" => TelegramProviderEvent::ReactionsObserved {
            account_id: account_id.to_owned(),
            provider_chat_id: required_string(payload, "chat_id")?,
            provider_message_id: required_string(payload, "message_id")?,
            reactions: parse_reaction_observations(payload)?,
        },
        "updateDeleteMessages" => {
            let chat_id = required_string(payload, "chat_id")?;
            let is_permanent = payload
                .get("is_permanent")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let message_ids = payload
                .get("message_ids")
                .and_then(Value::as_array)
                .ok_or_else(|| TdlibError::Protocol("deleted message ids are missing".to_owned()))?;
            return message_ids
                .iter()
                .map(|message_id| {
                    Ok(TelegramProviderEvent::MessageDeleted {
                        account_id: account_id.to_owned(),
                        provider_chat_id: chat_id.clone(),
                        provider_message_id: value_id(message_id)?,
                        is_permanent,
                    })
                })
                .collect();
        }
        "updateChatReadInbox" => TelegramProviderEvent::ChatUnreadChanged {
            account_id: account_id.to_owned(),
            provider_chat_id: required_string(payload, "chat_id")?,
            unread_count: payload.get("unread_count").and_then(Value::as_i64),
            unread_mention_count: None,
            last_read_inbox_message_id: payload
                .get("last_read_inbox_message_id")
                .map(value_id)
                .transpose()?,
        },
        "updateChatUnreadMentionCount" => TelegramProviderEvent::ChatUnreadChanged {
            account_id: account_id.to_owned(),
            provider_chat_id: required_string(payload, "chat_id")?,
            unread_count: None,
            unread_mention_count: payload.get("unread_mention_count").and_then(Value::as_i64),
            last_read_inbox_message_id: None,
        },
        "updateChatIsMarkedAsUnread" => TelegramProviderEvent::ChatMarkedUnreadChanged {
            account_id: account_id.to_owned(),
            provider_chat_id: required_string(payload, "chat_id")?,
            is_marked_as_unread: payload
                .get("is_marked_as_unread")
                .and_then(Value::as_bool)
                .ok_or_else(|| TdlibError::Protocol("marked unread state is missing".to_owned()))?,
        },
        _ => return Ok(Vec::new()),
    };
    Ok(vec![event])
}

fn message_text(content: Option<&Value>) -> Option<String> {
    content?
        .get("text")
        .or_else(|| content?.get("caption"))
        .and_then(|value| value.get("text"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn parse_message_media(content: Option<&Value>) -> Option<TelegramMessageMedia> {
    let content = content?;
    let content_type = content.get("@type").and_then(Value::as_str)?;
    let (kind, file) = match content_type {
        "messagePhoto" => (
            TelegramMediaKind::Photo,
            content.get("photo")?.get("sizes")?.as_array()?.last()?.get("photo"),
        ),
        "messageVideo" => (
            TelegramMediaKind::Video,
            content.get("video").and_then(|value| value.get("video")),
        ),
        "messageAudio" => (
            TelegramMediaKind::Audio,
            content.get("audio").and_then(|value| value.get("audio")),
        ),
        "messageDocument" => (
            TelegramMediaKind::Document,
            content.get("document").and_then(|value| value.get("document")),
        ),
        "messageAnimation" => (
            TelegramMediaKind::Animation,
            content.get("animation").and_then(|value| value.get("animation")),
        ),
        "messageVoiceNote" => (
            TelegramMediaKind::VoiceNote,
            content.get("voice_note").and_then(|value| value.get("voice")),
        ),
        _ => return None,
    };
    Some(TelegramMessageMedia {
        kind,
        provider_file_id: file
            .and_then(|value| value.get("id"))
            .and_then(value_id_optional),
        caption: content
            .get("caption")
            .and_then(|value| value.get("text"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        filename: content
            .get("document")
            .and_then(|value| value.get("file_name"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
    })
}

fn value_id_optional(value: &Value) -> Option<String> {
    value
        .as_i64()
        .map(|id| id.to_string())
        .or_else(|| value.as_str().map(ToOwned::to_owned))
}

fn optional_trimmed(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn participant_role(status_kind: &str) -> &'static str {
    match status_kind {
        "chatMemberStatusCreator" => "owner",
        "chatMemberStatusAdministrator" => "admin",
        "chatMemberStatusRestricted" => "restricted",
        "chatMemberStatusBanned" => "banned",
        "chatMemberStatusLeft" => "left",
        "chatMemberStatusMember" => "member",
        _ => "unknown",
    }
}

fn participant_permissions(status: Option<&Value>) -> Vec<String> {
    let Some(object) = status.and_then(Value::as_object) else {
        return Vec::new();
    };
    let mut permissions = Vec::new();
    for (key, value) in object {
        if key == "@type" {
            continue;
        }
        if value.as_bool() == Some(true) {
            permissions.push(key.clone());
        } else if let Some(text) = value.as_str().filter(|value| !value.trim().is_empty()) {
            permissions.push(format!("{key}={text}"));
        } else if let Some(nested) = value.as_object() {
            for (nested_key, nested_value) in nested {
                if nested_value.as_bool() == Some(true) {
                    permissions.push(format!("{key}.{nested_key}"));
                }
            }
        }
    }
    permissions.sort();
    permissions
}

fn integer_field(payload: &Value, field: &str) -> Option<u64> {
    payload
        .get(field)
        .and_then(Value::as_i64)
        .and_then(|value| u64::try_from(value).ok())
        .or_else(|| payload.get(field).and_then(Value::as_u64))
}

fn parse_reaction_observations(
    payload: &Value,
) -> Result<Vec<hermes_telegram_api::TelegramReactionObservation>, TdlibError> {
    let values = payload
        .get("interaction_info")
        .and_then(|value| value.get("reactions"))
        .and_then(|value| value.get("recent_reactions"))
        .and_then(Value::as_array)
        .ok_or_else(|| TdlibError::Protocol("TDLib reaction list is missing".to_owned()))?;
    values
        .iter()
        .map(|reaction| {
            let sender = reaction
                .get("sender_id")
                .and_then(|sender| sender.get("user_id").or_else(|| sender.get("chat_id")))
                .ok_or_else(|| TdlibError::Protocol("TDLib reaction sender is missing".to_owned()))?;
            let emoji = reaction
                .get("type")
                .and_then(|kind| kind.get("emoji"))
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| TdlibError::Protocol("TDLib reaction emoji is missing".to_owned()))?;
            Ok(hermes_telegram_api::TelegramReactionObservation {
                sender_id: value_id(sender)?,
                emoji: emoji.to_owned(),
                is_outgoing: reaction
                    .get("is_outgoing")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                is_active: true,
            })
        })
        .collect()
}

fn value_id(value: &Value) -> Result<String, TdlibError> {
    value
        .as_i64()
        .map(|id| id.to_string())
        .or_else(|| value.as_str().map(ToOwned::to_owned))
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(|| TdlibError::Protocol("TDLib provider id is invalid".to_owned()))
}

fn required_string(payload: &Value, field: &str) -> Result<String, TdlibError> {
    payload
        .get(field)
        .and_then(Value::as_i64)
        .map(|value| value.to_string())
        .or_else(|| payload.get(field).and_then(Value::as_str).map(ToOwned::to_owned))
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| TdlibError::Protocol(format!("TDLib payload field `{field}` is missing")))
}

fn chat_kind(value: &str) -> Result<TelegramChatKind, TdlibError> {
    match value {
        "chatTypePrivate" => Ok(TelegramChatKind::Private),
        "chatTypeBasicGroup" | "chatTypeSupergroup" => Ok(TelegramChatKind::Group),
        "chatTypeSecret" => Ok(TelegramChatKind::Private),
        other => Err(TdlibError::Protocol(format!("unsupported TDLib chat type `{other}`"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_command_encodes_tdlib_message_operation_without_domain_fields() {
        let command = TelegramProviderCommand::Edit {
            operation_id: "op-edit".to_owned(),
            account_id: "account".to_owned(),
            provider_chat_id: "100".to_owned(),
            provider_message_id: "200".to_owned(),
            text: "updated".to_owned(),
        };
        let encoded = encode_provider_command(&command).expect("valid edit command");
        assert_eq!(encoded["@type"], "editMessageText");
        assert_eq!(encoded["chat_id"], 100);
        assert_eq!(encoded["message_id"], 200);
        assert_eq!(encoded["@extra"], "op-edit");
    }

    #[test]
    fn mark_read_uses_view_messages_when_cursor_is_present() {
        let command = TelegramProviderCommand::MarkUnread {
            operation_id: "op-read".to_owned(),
            account_id: "account".to_owned(),
            provider_chat_id: "100".to_owned(),
            unread: false,
            read_through_provider_message_id: Some("200".to_owned()),
        };
        let encoded = encode_provider_command(&command).expect("valid read command");
        assert_eq!(encoded["@type"], "viewMessages");
        assert_eq!(encoded["message_ids"][0], 200);
    }

    #[test]
    fn parses_content_update_into_message_edit_event() {
        let events = parse_provider_events(
            "account",
            &json!({
                "@type": "updateMessageContent",
                "chat_id": 100,
                "message_id": 200,
                "new_content": {
                    "@type": "messageText",
                    "text": {"@type": "formattedText", "text": "edited"}
                }
            }),
        )
        .expect("content update");
        assert!(matches!(
            &events[0],
            TelegramProviderEvent::MessageEdited { text: Some(text), .. } if text == "edited"
        ));
    }

    #[test]
    fn parses_delete_update_as_one_event_per_provider_message() {
        let events = parse_provider_events(
            "account",
            &json!({
                "@type": "updateDeleteMessages",
                "chat_id": 100,
                "message_ids": [200, 201],
                "is_permanent": true
            }),
        )
        .expect("delete update");
        assert_eq!(events.len(), 2);
        assert!(matches!(
            &events[0],
            TelegramProviderEvent::MessageDeleted { provider_message_id, is_permanent: true, .. }
                if provider_message_id == "200"
        ));
    }

    #[test]
    fn encodes_media_download_and_participant_requests_at_provider_boundary() {
        let media = TelegramProviderCommand::SendMedia(hermes_telegram_api::TelegramSendMedia {
            operation_id: "op-media".to_owned(),
            account_id: "account".to_owned(),
            provider_chat_id: "100".to_owned(),
            media_kind: TelegramMediaKind::Document,
            blob_ref: "blob:report".to_owned(),
            caption: Some("report".to_owned()),
            filename: Some("report.pdf".to_owned()),
        });
        let encoded = encode_send_media_materialized(
            match &media {
                TelegramProviderCommand::SendMedia(command) => command,
                _ => unreachable!(),
            },
            "/tmp/report.pdf",
        )
        .expect("valid materialized media command");
        assert_eq!(encoded["input_message_content"]["@type"], "inputMessageDocument");
        assert_eq!(encoded["input_message_content"]["document"]["path"], "/tmp/report.pdf");
        assert!(encode_provider_command(&media).is_err());

        let participants = TelegramProviderCommand::ListParticipants {
            operation_id: "op-members".to_owned(),
            account_id: "account".to_owned(),
            provider_chat_id: "100".to_owned(),
            filter: TelegramParticipantFilter::Administrators,
            offset: 10,
            limit: 50,
        };
        let encoded = encode_provider_command(&participants).expect("valid participant command");
        assert_eq!(encoded["@type"], "getSupergroupMembers");
        assert_eq!(encoded["filter"]["@type"], "chatMembersFilterAdministrators");
        assert_eq!(encoded["@extra"], "op-members");
    }

    #[test]
    fn parses_provider_file_snapshot_without_exposing_raw_tdlib_payload() {
        let file = parse_file_snapshot(
            "account",
            &json!({
                "@type": "file",
                "id": 42,
                "size": 100,
                "expected_size": 100,
                "local": {"path": "/tmp/file", "downloaded_size": 100, "is_downloading_completed": true},
                "remote": {"unique_id": "remote-42"}
            }),
        )
        .expect("valid file snapshot");
        assert_eq!(file.provider_file_id, "42");
        assert_eq!(file.provider_unique_id.as_deref(), Some("remote-42"));
        assert!(file.is_downloaded);
    }

    #[test]
    fn parses_provider_participant_page_with_provider_roles() {
        let page = parse_participant_page(
            "account",
            "100",
            TelegramParticipantFilter::Recent,
            0,
            &json!({
                "members": [
                    {"member_id": {"user_id": 7}, "status": {"@type": "chatMemberStatusCreator"}},
                    {"member_id": {"user_id": 8}, "status": {"@type": "chatMemberStatusMember"}}
                ]
            }),
        )
        .expect("valid participant page");
        assert_eq!(page.items.len(), 2);
        assert!(page.items[0].is_owner);
        assert!(!page.items[1].is_admin);
        assert_eq!(page.next_offset, Some(2));
    }

    #[test]
    fn parses_message_media_file_and_caption_without_business_mapping() {
        let message = parse_message_observation(
            "account",
            &json!({
                "chat_id": 100,
                "id": 200,
                "date": 10,
                "sender_id": {"user_id": 7},
                "content": {
                    "@type": "messageDocument",
                    "document": {"file_name": "report.pdf", "document": {"id": 42}},
                    "caption": {"text": "report"}
                }
            }),
        )
        .expect("document message");
        let media = message.media.expect("media snapshot");
        assert_eq!(media.kind, TelegramMediaKind::Document);
        assert_eq!(media.provider_file_id.as_deref(), Some("42"));
        assert_eq!(media.caption.as_deref(), Some("report"));
        assert_eq!(media.filename.as_deref(), Some("report.pdf"));
        assert_eq!(message.text.as_deref(), Some("report"));
    }

    #[test]
    fn parses_typing_and_topic_updates_as_provider_events() {
        let typing = parse_provider_events(
            "account",
            &json!({
                "@type": "updateUserChatAction",
                "chat_id": 100,
                "message_thread_id": 7,
                "sender_id": {"@type": "messageSenderUser", "user_id": 42},
                "action": {"@type": "chatActionTyping"}
            }),
        )
        .expect("typing update");
        assert!(matches!(
            &typing[0],
            TelegramProviderEvent::TypingChanged(state)
                if state.sender_id == "user:42" && state.provider_thread_id.as_deref() == Some("7")
        ));

        let topic = parse_provider_events(
            "account",
            &json!({
                "@type": "updateForumTopicInfo",
                "chat_id": 100,
                "info": {"message_thread_id": 7, "name": "Release", "is_pinned": true, "is_closed": false}
            }),
        )
        .expect("topic update");
        assert!(matches!(
            &topic[0],
            TelegramProviderEvent::TopicChanged(topic)
                if topic.provider_topic_id == "7" && topic.title == "Release" && topic.is_pinned
        ));
    }

    #[test]
    fn parses_provider_chat_position_folder_and_notification_updates() {
        let position = parse_provider_events(
            "account",
            &json!({
                "@type": "updateChatPosition",
                "chat_id": 100,
                "position": {
                    "list": {"@type": "chatListFolder", "chat_folder_id": 7},
                    "order": 9,
                    "is_pinned": true
                }
            }),
        )
        .expect("chat position update");
        assert!(matches!(
            &position[0],
            TelegramProviderEvent::ChatPositionChanged(position)
                if position.list_kind == "folder" && position.provider_folder_id == Some(7) && position.is_pinned
        ));

        let folders = parse_provider_events(
            "account",
            &json!({
                "@type": "updateChatFolders",
                "chat_folders": [{"id": 7, "name": {"text": "Projects"}, "icon": {"name": "briefcase"}, "color_id": 3}]
            }),
        )
        .expect("chat folders update");
        assert!(matches!(
            &folders[0],
            TelegramProviderEvent::ChatFoldersChanged { folders, .. }
                if folders[0].title == "Projects" && folders[0].provider_folder_id == 7
        ));

        let notification = parse_provider_events(
            "account",
            &json!({
                "@type": "updateChatNotificationSettings",
                "chat_id": 100,
                "notification_settings": {"use_default_mute_for": false, "mute_for": 3600}
            }),
        )
        .expect("notification update");
        assert!(matches!(
            &notification[0],
            TelegramProviderEvent::ChatNotificationChanged { mute_for_seconds: 3600, .. }
        ));
    }

    #[test]
    fn preserves_tdlib_file_progress_as_provider_event() {
        let events = parse_provider_events(
            "account",
            &json!({
                "@type": "updateFile",
                "file": {
                    "@type": "file",
                    "id": 42,
                    "size": 100,
                    "local": {"downloaded_size": 40, "is_downloading_active": true, "is_downloading_completed": false}
                }
            }),
        )
        .expect("file update");
        assert!(matches!(
            &events[0],
            TelegramProviderEvent::FileChanged(file)
                if file.provider_file_id == "42" && file.downloaded_size_bytes == Some(40) && file.is_downloading
        ));
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TdlibError {
    Transport(String),
    Protocol(String),
    AuthenticationRequired,
    RuntimeUnavailable,
}

type TdJsonClientCreate = unsafe extern "C" fn() -> *mut c_void;
type TdJsonClientSend = unsafe extern "C" fn(*mut c_void, *const c_char);
type TdJsonClientReceive = unsafe extern "C" fn(*mut c_void, f64) -> *const c_char;
type TdJsonClientExecute = unsafe extern "C" fn(*mut c_void, *const c_char) -> *const c_char;
type TdJsonClientDestroy = unsafe extern "C" fn(*mut c_void);

/// Loaded libtdjson handle. The unsafe C ABI is isolated to this adapter file.
pub struct TdJsonLibrary {
    create: TdJsonClientCreate,
    send: TdJsonClientSend,
    receive: TdJsonClientReceive,
    execute: TdJsonClientExecute,
    destroy: TdJsonClientDestroy,
    _library: Library,
}

impl TdJsonLibrary {
    pub fn load(configured_path: Option<&Path>) -> Result<Self, TdlibError> {
        let candidates = library_candidates(configured_path);
        let mut errors = Vec::new();
        for candidate in candidates {
            let library = unsafe { Library::new(&candidate) };
            match library {
                Ok(library) => return Self::from_library(library, &candidate),
                Err(error) => errors.push(format!("{}: {error}", candidate.display())),
            }
        }
        Err(TdlibError::Transport(format!(
            "unable to load libtdjson: {}",
            errors.join("; ")
        )))
    }

    fn from_library(library: Library, candidate: &Path) -> Result<Self, TdlibError> {
        Ok(Self {
            create: load_symbol(&library, b"td_json_client_create\0", candidate)?,
            send: load_symbol(&library, b"td_json_client_send\0", candidate)?,
            receive: load_symbol(&library, b"td_json_client_receive\0", candidate)?,
            execute: load_symbol(&library, b"td_json_client_execute\0", candidate)?,
            destroy: load_symbol(&library, b"td_json_client_destroy\0", candidate)?,
            _library: library,
        })
    }

    pub fn create_client(self) -> Result<TdJsonClient, TdlibError> {
        let client = unsafe { (self.create)() };
        if client.is_null() {
            return Err(TdlibError::Transport("td_json_client_create returned null".to_owned()));
        }
        Ok(TdJsonClient { client, library: self })
    }
}

pub struct TdJsonClient {
    client: *mut c_void,
    library: TdJsonLibrary,
}

impl TdJsonClient {
    pub fn send_json(&self, request: &Value) -> Result<(), TdlibError> {
        let request = CString::new(request.to_string())
            .map_err(|_| TdlibError::Protocol("TDLib request contains NUL".to_owned()))?;
        unsafe { (self.library.send)(self.client, request.as_ptr()) };
        Ok(())
    }

    pub fn receive_json(&self, timeout_seconds: f64) -> Result<Option<Value>, TdlibError> {
        let response = unsafe { (self.library.receive)(self.client, timeout_seconds) };
        parse_response(response)
    }

    pub fn execute_json(&self, request: &Value) -> Result<Option<Value>, TdlibError> {
        let request = CString::new(request.to_string())
            .map_err(|_| TdlibError::Protocol("TDLib request contains NUL".to_owned()))?;
        let response = unsafe { (self.library.execute)(self.client, request.as_ptr()) };
        parse_response(response)
    }
}

impl Drop for TdJsonClient {
    fn drop(&mut self) {
        if !self.client.is_null() {
            unsafe { (self.library.destroy)(self.client) };
            self.client = std::ptr::null_mut();
        }
    }
}

fn parse_response(response: *const c_char) -> Result<Option<Value>, TdlibError> {
    if response.is_null() {
        return Ok(None);
    }
    let text = unsafe { CStr::from_ptr(response) }
        .to_str()
        .map_err(|error| TdlibError::Protocol(format!("invalid TDLib UTF-8: {error}")))?;
    serde_json::from_str(text)
        .map(Some)
        .map_err(|error| TdlibError::Protocol(format!("invalid TDLib JSON: {error}")))
}

fn load_symbol<T: Copy>(library: &Library, name: &'static [u8], candidate: &Path) -> Result<T, TdlibError> {
    let symbol = unsafe { library.get::<T>(name) }.map_err(|error| {
        TdlibError::Transport(format!("libtdjson `{}` is missing symbol: {error}", candidate.display()))
    })?;
    Ok(*symbol)
}

fn library_candidates(configured_path: Option<&Path>) -> Vec<PathBuf> {
    if let Some(path) = configured_path {
        return vec![path.to_path_buf()];
    }
    #[cfg(target_os = "macos")]
    {
        return vec![
            PathBuf::from("libtdjson.dylib"),
            PathBuf::from("/opt/homebrew/opt/tdlib/lib/libtdjson.dylib"),
            PathBuf::from("/usr/local/opt/tdlib/lib/libtdjson.dylib"),
        ];
    }
    #[cfg(target_os = "linux")]
    {
        return vec![
            PathBuf::from("libtdjson.so"),
            PathBuf::from("/usr/local/lib/libtdjson.so"),
            PathBuf::from("/usr/lib/libtdjson.so"),
        ];
    }
    #[cfg(target_os = "windows")]
    {
        vec![PathBuf::from("tdjson.dll")]
    }
}

/// The runtime owns this port; TDLib transport implementations own sockets/processes.
pub trait TdlibTransport {
    fn request(&mut self, request: TdlibRequest) -> Result<TdlibResponse, TdlibError>;
    fn poll_events(&mut self) -> Result<Vec<TelegramProviderEvent>, TdlibError>;
}

/// Real libtdjson execution port. It owns correlation, while provider parsing stays here.
pub struct TdJsonTransport {
    client: TdJsonClient,
    account_id: String,
    receive_timeout_seconds: f64,
    request_timeout: Duration,
    pending_updates: VecDeque<Value>,
}

impl TdJsonTransport {
    pub fn new(client: TdJsonClient, account_id: impl Into<String>) -> Result<Self, TdlibError> {
        let account_id = account_id.into();
        if account_id.trim().is_empty() {
            return Err(TdlibError::Protocol("Telegram account id is empty".to_owned()));
        }
        Ok(Self {
            client,
            account_id,
            receive_timeout_seconds: 0.25,
            request_timeout: Duration::from_secs(30),
            pending_updates: VecDeque::new(),
        })
    }

    pub fn with_timeouts(
        mut self,
        receive_timeout_seconds: f64,
        request_timeout: Duration,
    ) -> Result<Self, TdlibError> {
        if !(0.0..=10.0).contains(&receive_timeout_seconds) || request_timeout.is_zero() {
            return Err(TdlibError::Protocol("TDLib transport timeout is invalid".to_owned()));
        }
        self.receive_timeout_seconds = receive_timeout_seconds;
        self.request_timeout = request_timeout;
        Ok(self)
    }

    pub fn poll_events(&mut self) -> Result<Vec<TelegramProviderEvent>, TdlibError> {
        let mut events = Vec::new();
        while let Some(payload) = self.pending_updates.pop_front() {
            events.extend(parse_provider_events(&self.account_id, &payload)?);
        }
        while let Some(payload) = self.client.receive_json(0.0)? {
            events.extend(parse_provider_events(&self.account_id, &payload)?);
        }
        Ok(events)
    }

    fn receive_correlated(&mut self, expected_extra: &str) -> Result<Value, TdlibError> {
        let started = Instant::now();
        while started.elapsed() < self.request_timeout {
            let Some(payload) = self.client.receive_json(self.receive_timeout_seconds)? else {
                continue;
            };
            if payload.get("@extra").and_then(Value::as_str) == Some(expected_extra) {
                return Ok(payload);
            }
            if payload.get("@type").and_then(Value::as_str) == Some("error") {
                return Err(tdlib_error(&payload));
            }
            self.pending_updates.push_back(payload);
        }
        Err(TdlibError::Protocol(format!(
            "TDLib request `{expected_extra}` timed out"
        )))
    }

    fn request_once(&mut self, request: &TdlibRequest) -> Result<TdlibResponse, TdlibError> {
        let payload = encode_request(request)?;
        let expected_extra = request_extra(request);
        self.client.send_json(&payload)?;
        let response = self.receive_correlated(&expected_extra)?;
        parse_response_for_request(&self.account_id, request, response)
    }
}

impl TdlibTransport for TdJsonTransport {
    fn request(&mut self, request: TdlibRequest) -> Result<TdlibResponse, TdlibError> {
        if let TdlibRequest::ProviderCommand(TelegramProviderCommand::RemoveChatFromFolder {
            operation_id,
            provider_chat_id,
            provider_folder_id,
            ..
        }) = &request
        {
            let get_extra = format!("{operation_id}:get");
            self.client.send_json(&json!({
                "@type": "getChatFolder",
                "chat_folder_id": provider_folder_id,
                "@extra": get_extra,
            }))?;
            let folder = self.receive_correlated(&get_extra)?;
            let edit = encode_remove_chat_from_folder(
                *provider_folder_id,
                provider_id(provider_chat_id)?,
                &folder,
                operation_id,
            )?;
            self.client.send_json(&edit)?;
            let response = self.receive_correlated(operation_id)?;
            return parse_response_for_request(&self.account_id, &request, response);
        }
        if let TdlibRequest::ListBasicGroupParticipants {
            account_id,
            basic_group_id,
            ..
        } = &request
        {
            let group_extra = format!("telegram-basic-group-{account_id}-{basic_group_id}");
            self.client.send_json(&json!({
                "@type": "getBasicGroup",
                "basic_group_id": basic_group_id,
                "@extra": group_extra,
            }))?;
            let _group = self.receive_correlated(&group_extra)?;
            let full_info_extra = format!("telegram-basic-group-full-info-{account_id}-{basic_group_id}");
            self.client.send_json(&json!({
                "@type": "getBasicGroupFullInfo",
                "basic_group_id": basic_group_id,
                "@extra": full_info_extra,
            }))?;
            let response = self.receive_correlated(&full_info_extra)?;
            return parse_response_for_request(&self.account_id, &request, response);
        }
        if let TdlibRequest::LoadChats { account_id, limit } = &request {
            let list_request = TdlibRequest::LoadChats {
                account_id: account_id.clone(),
                limit: *limit,
            };
            let payload = encode_request(&list_request)?;
            let expected_extra = request_extra(&list_request);
            self.client.send_json(&payload)?;
            let response = self.receive_correlated(&expected_extra)?;
            let ids = response
                .get("chat_ids")
                .and_then(Value::as_array)
                .ok_or_else(|| TdlibError::Protocol("TDLib getChats response is missing chat_ids".to_owned()))?;
            let mut chats = Vec::with_capacity(ids.len());
            for id in ids.iter().take(*limit as usize) {
                let provider_chat_id = value_id(id)?;
                let extra = format!("telegram-get-chat-{account_id}-{provider_chat_id}");
                self.client.send_json(&json!({
                    "@type": "getChat",
                    "chat_id": provider_id(&provider_chat_id)?,
                    "@extra": extra,
                }))?;
                let chat_payload = self.receive_correlated(&extra)?;
                chats.push(parse_chat(account_id, &chat_payload)?);
            }
            return Ok(TdlibResponse::Chats(chats));
        }
        self.request_once(&request)
    }

    fn poll_events(&mut self) -> Result<Vec<TelegramProviderEvent>, TdlibError> {
        TdJsonTransport::poll_events(self)
    }
}

fn encode_remove_chat_from_folder(
    folder_id: i64,
    chat_id: i64,
    folder: &Value,
    extra: &str,
) -> Result<Value, TdlibError> {
    if folder.get("@type").and_then(Value::as_str) != Some("chatFolder") {
        return Err(TdlibError::Protocol(
            "TDLib getChatFolder response is missing chatFolder payload".to_owned(),
        ));
    }
    let unique_ids = |key: &str| {
        let mut ids = Vec::new();
        if let Some(values) = folder.get(key).and_then(Value::as_array) {
            for value in values.iter().filter_map(Value::as_i64) {
                if !ids.contains(&value) {
                    ids.push(value);
                }
            }
        }
        ids
    };
    let mut pinned = unique_ids("pinned_chat_ids");
    pinned.retain(|value| *value != chat_id);
    let mut included = unique_ids("included_chat_ids");
    included.retain(|value| *value != chat_id);
    let mut excluded = unique_ids("excluded_chat_ids");
    if !excluded.contains(&chat_id) {
        excluded.push(chat_id);
    }
    let text = |parent: &str, key: &str| {
        folder
            .get(parent)
            .and_then(|value| value.get(key))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("")
    };
    let bool_field = |key: &str| folder.get(key).and_then(Value::as_bool).unwrap_or(false);
    Ok(json!({
        "@type": "editChatFolder",
        "chat_folder_id": folder_id,
        "folder": {
            "@type": "chatFolder",
            "name": {"@type": "chatFolderName", "text": text("name", "text"), "animate_custom_emoji": folder.get("name").and_then(|value| value.get("animate_custom_emoji")).and_then(Value::as_bool).unwrap_or(false)},
            "icon": {"@type": "chatFolderIcon", "name": text("icon", "name")},
            "color_id": folder.get("color_id").and_then(Value::as_i64).unwrap_or_default(),
            "is_shareable": bool_field("is_shareable"),
            "pinned_chat_ids": pinned,
            "included_chat_ids": included,
            "excluded_chat_ids": excluded,
            "exclude_muted": bool_field("exclude_muted"),
            "exclude_read": bool_field("exclude_read"),
            "exclude_archived": bool_field("exclude_archived"),
            "include_contacts": bool_field("include_contacts"),
            "include_non_contacts": bool_field("include_non_contacts"),
            "include_bots": bool_field("include_bots"),
            "include_groups": bool_field("include_groups"),
            "include_channels": bool_field("include_channels")
        },
        "@extra": extra.trim()
    }))
}

fn request_extra(request: &TdlibRequest) -> String {
    match request {
        TdlibRequest::LoadChats { account_id, .. }
        | TdlibRequest::LoadHistory { account_id, .. }
        | TdlibRequest::ListTopics { account_id, .. }
        | TdlibRequest::ListParticipants { account_id, .. }
        | TdlibRequest::ListBasicGroupParticipants { account_id, .. } => account_id.clone(),
        TdlibRequest::GetChatFolder {
            account_id,
            provider_folder_id,
        } => format!("{account_id}:folder:{provider_folder_id}"),
        TdlibRequest::SendMessage(command) => command.operation_id.clone(),
        TdlibRequest::SendMedia(command) => command.operation_id.clone(),
        TdlibRequest::SendMediaMaterialized { command, .. } => command.operation_id.clone(),
        TdlibRequest::DownloadFile(command) => command.operation_id.clone(),
        TdlibRequest::ProviderCommand(command) => provider_command_operation_id(command).to_owned(),
    }
}

fn parse_response_for_request(
    account_id: &str,
    request: &TdlibRequest,
    response: Value,
) -> Result<TdlibResponse, TdlibError> {
    match request {
        TdlibRequest::LoadHistory { .. } => {
            let messages = response
                .get("messages")
                .and_then(Value::as_array)
                .ok_or_else(|| TdlibError::Protocol("TDLib history response is missing messages".to_owned()))?;
            Ok(TdlibResponse::History(
                messages
                    .iter()
                    .map(|message| parse_message_observation(account_id, message))
                    .collect::<Result<Vec<_>, _>>()?,
            ))
        }
        TdlibRequest::ListParticipants { provider_chat_id, filter, .. } => Ok(TdlibResponse::Participants(
            parse_participant_page(account_id, provider_chat_id, *filter, 0, &response)?,
        )),
        TdlibRequest::ListBasicGroupParticipants { provider_chat_id, .. } => Ok(TdlibResponse::Participants(
            parse_participant_page(account_id, provider_chat_id, TelegramParticipantFilter::Recent, 0, &response)?,
        )),
        TdlibRequest::ListTopics { provider_chat_id, .. } => {
            Ok(TdlibResponse::Topics(parse_topic_list(account_id, provider_chat_id, &response)?))
        }
        TdlibRequest::GetChatFolder { .. } => Ok(TdlibResponse::ChatFolders(
            parse_chat_folders(account_id, &json!({"chat_folders": [response]}))?,
        )),
        TdlibRequest::DownloadFile { .. } => Ok(TdlibResponse::File(parse_file_snapshot(account_id, &response)?)),
        TdlibRequest::SendMessage(command) => sent_response(&command.operation_id, &response),
        TdlibRequest::SendMedia(command) => sent_response(&command.operation_id, &response),
        TdlibRequest::SendMediaMaterialized { command, .. } => sent_response(&command.operation_id, &response),
        TdlibRequest::ProviderCommand(TelegramProviderCommand::SearchMessages { .. }) => {
            Ok(TdlibResponse::History(parse_message_list_response(account_id, &response)?))
        }
        TdlibRequest::ProviderCommand(command) => {
            if response.get("@type").and_then(Value::as_str) == Some("message") {
                sent_response(provider_command_operation_id(command), &response)
            } else {
                Ok(TdlibResponse::Accepted {
                    operation_id: provider_command_operation_id(command).to_owned(),
                })
            }
        }
        TdlibRequest::LoadChats { .. } => unreachable!("LoadChats is handled by TdJsonTransport::request"),
    }
}

fn parse_message_list_response(
    account_id: &str,
    response: &Value,
) -> Result<Vec<TelegramMessageObservation>, TdlibError> {
    let messages = response
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| TdlibError::Protocol("TDLib search response is missing messages".to_owned()))?;
    messages
        .iter()
        .map(|message| parse_message_observation(account_id, message))
        .collect()
}

fn sent_response(operation_id: &str, response: &Value) -> Result<TdlibResponse, TdlibError> {
    if response.get("@type").and_then(Value::as_str) != Some("message") {
        return Ok(TdlibResponse::Accepted {
            operation_id: operation_id.to_owned(),
        });
    }
    Ok(TdlibResponse::Sent {
        provider_message_id: required_string(response, "id")?,
    })
}

fn tdlib_error(payload: &Value) -> TdlibError {
    let code = payload.get("code").and_then(Value::as_i64).map(|value| value.to_string()).unwrap_or_else(|| "unknown".to_owned());
    let message = payload.get("message").and_then(Value::as_str).unwrap_or("TDLib returned an error");
    TdlibError::Protocol(format!("TDLib error {code}: {message}"))
}

#[cfg(test)]
mod message_reference_tests {
    use super::*;

    #[test]
    fn parses_reply_and_forward_references_as_provider_data() {
        let observation = parse_message_observation(
            "account",
            &json!({
                "chat_id": 100,
                "id": 200,
                "sender_id": {"user_id": 42},
                "date": 10,
                "reply_to": {
                    "@type": "messageReplyToMessage",
                    "chat_id": 100,
                    "message_id": 150
                },
                "forward_info": {
                    "date": 5,
                    "origin": {
                        "@type": "messageOriginUser",
                        "sender_user_id": 7
                    }
                },
                "content": {"@type": "messageText", "text": {"text": "forwarded reply"}}
            }),
        )
        .expect("message references");

        assert_eq!(
            observation.references.reply_to,
            Some(TelegramReplyReference {
                provider_chat_id: "100".to_owned(),
                provider_message_id: "150".to_owned(),
            })
        );
        assert_eq!(
            observation.references.forward_origin,
            Some(TelegramForwardOrigin {
                provider_chat_id: None,
                provider_message_id: None,
                provider_sender_id: Some("7".to_owned()),
                sender_name: None,
                observed_at_unix_seconds: Some(5),
            })
        );
    }
}

pub struct TdlibClient<T> {
    transport: T,
}

#[cfg(test)]
mod folder_command_tests {
    use super::*;

    #[test]
    fn add_chat_to_folder_uses_provider_folder_chat_list() {
        let command = TelegramProviderCommand::AddChatToFolder {
            operation_id: "op-folder-add".to_owned(),
            account_id: "account".to_owned(),
            provider_chat_id: "100".to_owned(),
            provider_folder_id: 7,
        };
        let encoded = encode_provider_command(&command).expect("valid folder add");
        assert_eq!(encoded["@type"], "addChatToList");
        assert_eq!(encoded["chat_id"], 100);
        assert_eq!(encoded["chat_list"]["@type"], "chatListFolder");
        assert_eq!(encoded["chat_list"]["chat_folder_id"], 7);
    }

    #[test]
    fn remove_chat_from_folder_preserves_folder_policy_and_updates_membership() {
        let folder = json!({
            "@type": "chatFolder",
            "name": {"text": " Projects ", "animate_custom_emoji": true},
            "icon": {"name": "briefcase"},
            "color_id": 3,
            "is_shareable": true,
            "pinned_chat_ids": [100, 101],
            "included_chat_ids": [100, 102],
            "excluded_chat_ids": [103],
            "exclude_muted": true,
            "exclude_read": false,
            "exclude_archived": true,
            "include_contacts": true,
            "include_non_contacts": false,
            "include_bots": true,
            "include_groups": true,
            "include_channels": false
        });
        let encoded = encode_remove_chat_from_folder(7, 100, &folder, "op-folder-remove")
            .expect("valid folder removal");
        assert_eq!(encoded["@type"], "editChatFolder");
        assert_eq!(encoded["folder"]["name"]["text"], "Projects");
        assert_eq!(encoded["folder"]["name"]["animate_custom_emoji"], true);
        assert_eq!(encoded["folder"]["pinned_chat_ids"], json!([101]));
        assert_eq!(encoded["folder"]["included_chat_ids"], json!([102]));
        assert_eq!(encoded["folder"]["excluded_chat_ids"], json!([103, 100]));
        assert_eq!(encoded["folder"]["exclude_muted"], true);
        assert_eq!(encoded["folder"]["include_bots"], true);
    }

    #[test]
    fn parses_search_message_list_into_provider_observations() {
        let messages = parse_message_list_response(
            "account",
            &json!({
                "@type": "messages",
                "messages": [{
                    "chat_id": 100,
                    "id": 200,
                    "sender_id": {"user_id": 42},
                    "date": 10,
                    "content": {"@type": "messageText", "text": {"text": "release"}}
                }]
            }),
        )
        .expect("search messages");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].provider_chat_id, "100");
        assert_eq!(messages[0].provider_message_id, "200");
        assert_eq!(messages[0].text.as_deref(), Some("release"));
    }
}

impl<T> TdlibClient<T>
where
    T: TdlibTransport,
{
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn request(&mut self, request: TdlibRequest) -> Result<TdlibResponse, TdlibError> {
        self.transport.request(request)
    }
}
