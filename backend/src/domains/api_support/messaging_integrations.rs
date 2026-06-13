use super::*;

#[derive(Serialize)]
pub(crate) struct TelegramCapabilitiesResponse {
    pub(crate) version: &'static str,
    pub(crate) runtime_mode: &'static str,
    pub(crate) telegram_app_credentials_configured: bool,
    pub(crate) tdjson_runtime_available: bool,
    pub(crate) qr_login_ready: bool,
    pub(crate) capabilities: Vec<TelegramCapabilityStatus>,
    pub(crate) unsupported_features: Vec<&'static str>,
}

impl TelegramCapabilitiesResponse {
    pub(crate) fn current(config: &AppConfig) -> Self {
        let telegram_app_credentials_configured =
            config.telegram_api_id().is_some() && config.telegram_api_hash().is_some();
        let tdjson_runtime_available = tdjson::runtime_available(config.tdjson_path());
        let qr_login_ready = telegram_app_credentials_configured && tdjson_runtime_available;

        Self {
            version: "1.0",
            runtime_mode: if qr_login_ready {
                "tdlib_qr"
            } else {
                "fixture"
            },
            telegram_app_credentials_configured,
            tdjson_runtime_available,
            qr_login_ready,
            capabilities: vec![
                TelegramCapabilityStatus::available(
                    "telegram_fixture_runtime",
                    "Fixture Telegram accounts, chats and message projection are available for CI and local smoke validation.",
                    true,
                ),
                if qr_login_ready {
                    TelegramCapabilityStatus::available(
                        "tdlib_live_runtime",
                        "TDLib QR login runtime is configured for local development.",
                        true,
                    )
                } else {
                    TelegramCapabilityStatus::blocked(
                        "tdlib_live_runtime",
                        "Live TDLib sessions require a loadable native TDLib JSON runtime and Telegram app credentials.",
                        false,
                    )
                },
                TelegramCapabilityStatus::blocked(
                    "telegram_bot_live_runtime",
                    "Live bot sends require the Bot API runtime adapter and account-scoped bot token resolution.",
                    false,
                ),
                TelegramCapabilityStatus::available(
                    "automation_dry_run",
                    "Policy/template validation and audited dry-run records are available.",
                    true,
                ),
                TelegramCapabilityStatus::blocked(
                    "automation_live_send",
                    "Live automated sends remain blocked until a live Telegram runtime passes the same policy evaluator and audit contract.",
                    false,
                ),
                TelegramCapabilityStatus::available(
                    "call_state_and_transcript_storage",
                    "1:1 call metadata and transcript artifact storage are available through fixture APIs.",
                    true,
                ),
                TelegramCapabilityStatus::blocked(
                    "desktop_audio_capture",
                    "Desktop audio capture requires a visible recording runtime boundary and explicit platform permissions.",
                    false,
                ),
                TelegramCapabilityStatus::available(
                    "fixture_speech_to_text",
                    "Fixture speech-to-text provider is available for deterministic tests.",
                    true,
                ),
                TelegramCapabilityStatus::blocked(
                    "whisper_rs_speech_to_text",
                    "Real local transcription requires the whisper-rs provider adapter and local model configuration.",
                    false,
                ),
            ],
            unsupported_features: vec![
                "video_calls",
                "group_calls",
                "screen_sharing",
                "hidden_recording",
                "telegram_data_fine_tuning",
                "third_party_plugin_execution",
            ],
        }
    }
}

#[derive(Serialize)]
pub(crate) struct TelegramCapabilityStatus {
    pub(crate) capability: &'static str,
    pub(crate) status: &'static str,
    pub(crate) closure_gate: bool,
    pub(crate) reason: &'static str,
}

impl TelegramCapabilityStatus {
    fn available(capability: &'static str, reason: &'static str, closure_gate: bool) -> Self {
        Self {
            capability,
            status: "available",
            closure_gate,
            reason,
        }
    }

    fn blocked(capability: &'static str, reason: &'static str, closure_gate: bool) -> Self {
        Self {
            capability,
            status: "blocked",
            closure_gate,
            reason,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct WhatsappCapabilitiesResponse {
    pub(crate) version: &'static str,
    pub(crate) runtime_mode: &'static str,
    pub(crate) capabilities: Vec<WhatsappCapabilityStatus>,
    pub(crate) unsupported_features: Vec<&'static str>,
}

impl WhatsappCapabilitiesResponse {
    pub(crate) fn current() -> Self {
        Self {
            version: "1.0",
            runtime_mode: "fixture",
            capabilities: vec![
                WhatsappCapabilityStatus::available(
                    "whatsapp_web_fixture_runtime",
                    "Fixture WhatsApp Web accounts, session metadata and message projection are available for CI and local smoke validation.",
                    true,
                ),
                WhatsappCapabilityStatus::available(
                    "whatsapp_web_manual_session_state",
                    "Manual companion session metadata is stored without session secrets or pairing material in PostgreSQL.",
                    true,
                ),
                WhatsappCapabilityStatus::available(
                    "whatsapp_web_fixture_ingestion",
                    "Fixture WhatsApp Web messages preserve append-only raw provenance and project into canonical communication messages.",
                    true,
                ),
                WhatsappCapabilityStatus::blocked(
                    "whatsapp_web_live_runtime",
                    "Live WhatsApp Web requires a user-visible desktop runtime, explicit session lifecycle and smoke validation.",
                    false,
                ),
                WhatsappCapabilityStatus::blocked(
                    "whatsapp_web_live_send",
                    "Live outbound sends require a WhatsApp-specific policy, audit and visible runtime contract.",
                    false,
                ),
            ],
            unsupported_features: vec![
                "hidden_web_scraping",
                "reverse_engineered_protocol_runtime",
                "bulk_messaging",
                "auto_messaging",
                "auto_dialing",
                "whatsapp_data_fine_tuning",
                "whatsapp_business_cloud_as_personal_provider",
            ],
        }
    }
}

#[derive(Serialize)]
pub(crate) struct WhatsappCapabilityStatus {
    pub(crate) capability: &'static str,
    pub(crate) status: &'static str,
    pub(crate) closure_gate: bool,
    pub(crate) reason: &'static str,
}

impl WhatsappCapabilityStatus {
    fn available(capability: &'static str, reason: &'static str, closure_gate: bool) -> Self {
        Self {
            capability,
            status: "available",
            closure_gate,
            reason,
        }
    }

    fn blocked(capability: &'static str, reason: &'static str, closure_gate: bool) -> Self {
        Self {
            capability,
            status: "blocked",
            closure_gate,
            reason,
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct TelegramListQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct TelegramChatListResponse {
    pub(crate) items: Vec<TelegramChat>,
}

#[derive(Serialize)]
pub(crate) struct TelegramMessageListResponse {
    pub(crate) items: Vec<TelegramMessage>,
}

#[derive(Deserialize)]
pub(crate) struct WhatsappWebListQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) provider_chat_id: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct WhatsappWebSessionListResponse {
    pub(crate) items: Vec<WhatsappWebSession>,
}

#[derive(Serialize)]
pub(crate) struct WhatsappWebMessageListResponse {
    pub(crate) items: Vec<WhatsappWebMessage>,
}
