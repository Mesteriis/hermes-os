use super::*;

// ---------------------------------------------------------------------------
// WhatsApp capability model (unchanged)
// ---------------------------------------------------------------------------

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

#[derive(Deserialize)]
pub(crate) struct TelegramReactionDeleteQuery {
    pub(crate) account_id: String,
    pub(crate) provider_chat_id: String,
    pub(crate) provider_message_id: String,
    pub(crate) reaction_emoji: String,
    pub(crate) sender_id: String,
    pub(crate) sender_display_name: Option<String>,
    pub(crate) command_id: Option<String>,
}
