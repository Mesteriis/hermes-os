use super::models::{RealtimeConversationProviderCapabilities, RealtimeConversationProviderKind};

pub trait RealtimeConversationProvider {
    fn provider_kind(&self) -> RealtimeConversationProviderKind;
    fn provider_shape(&self) -> &'static str;
    fn capabilities(&self) -> RealtimeConversationProviderCapabilities;
}

pub fn yandex_telemost_provider_capabilities() -> RealtimeConversationProviderCapabilities {
    RealtimeConversationProviderCapabilities {
        provider_kind: RealtimeConversationProviderKind::YandexTelemost,
        provider_shape: "yandex_telemost_user".to_owned(),
        supports_conference_create: true,
        supports_visible_webview: true,
        supports_audio_capture: true,
        supports_participant_events: false,
        supports_speaker_hints: true,
        supports_chat_capture: false,
        supports_screen_share_detection: false,
        supports_screenshot_hints: true,
        supports_recording: true,
        supports_provider_transcript: false,
        supports_reactions: false,
        evidence: RealtimeConversationProviderCapabilities::evidence_source(
            "yandex_telemost_api_and_visible_webview_runtime",
        ),
    }
}

pub fn zoom_provider_capabilities() -> RealtimeConversationProviderCapabilities {
    RealtimeConversationProviderCapabilities {
        provider_kind: RealtimeConversationProviderKind::Zoom,
        provider_shape: "zoom_user".to_owned(),
        supports_conference_create: true,
        supports_visible_webview: true,
        supports_audio_capture: true,
        supports_participant_events: true,
        supports_speaker_hints: true,
        supports_chat_capture: true,
        supports_screen_share_detection: true,
        supports_screenshot_hints: true,
        supports_recording: true,
        supports_provider_transcript: true,
        supports_reactions: true,
        evidence: RealtimeConversationProviderCapabilities::evidence_source(
            "zoom_provider_runtime_contract",
        ),
    }
}

pub fn generic_webview_provider_capabilities(
    provider_kind: RealtimeConversationProviderKind,
    provider_shape: impl Into<String>,
) -> RealtimeConversationProviderCapabilities {
    RealtimeConversationProviderCapabilities {
        provider_kind,
        provider_shape: provider_shape.into(),
        supports_conference_create: false,
        supports_visible_webview: true,
        supports_audio_capture: true,
        supports_participant_events: false,
        supports_speaker_hints: false,
        supports_chat_capture: false,
        supports_screen_share_detection: false,
        supports_screenshot_hints: true,
        supports_recording: true,
        supports_provider_transcript: false,
        supports_reactions: false,
        evidence: RealtimeConversationProviderCapabilities::evidence_source(
            "generic_visible_webview_runtime",
        ),
    }
}
