use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use crate::ai::core::types::{AiModelRouteTarget, AiModelRouting};
use crate::platform::ai_runtime::{
    AiChatResult, AiEmbedResult, AiRuntimePortError, SharedAiRuntimePort,
};

#[path = "sensitivity_guard.rs"]
mod sensitivity_guard;
use sensitivity_guard::*;

pub type SharedAiHub = Arc<AiHub>;

const LOCAL_AGENT_SOURCE: &str = "local_rust_ai_guard";

#[derive(Clone)]
pub struct AiHub {
    runtime: SharedAiRuntimePort,
    model_routing: AiModelRouting,
    usage_recorder: Option<SharedAiHubUsageRecorder>,
}

impl AiHub {
    pub fn new(runtime: SharedAiRuntimePort, model_routing: AiModelRouting) -> Self {
        Self {
            runtime,
            model_routing,
            usage_recorder: None,
        }
    }

    pub fn new_with_usage_recorder(
        runtime: SharedAiRuntimePort,
        model_routing: AiModelRouting,
        usage_recorder: SharedAiHubUsageRecorder,
    ) -> Self {
        Self {
            runtime,
            model_routing,
            usage_recorder: Some(usage_recorder),
        }
    }

    pub fn shared(runtime: SharedAiRuntimePort, model_routing: AiModelRouting) -> SharedAiHub {
        Arc::new(Self::new(runtime, model_routing))
    }

    pub fn shared_with_usage_recorder(
        runtime: SharedAiRuntimePort,
        model_routing: AiModelRouting,
        usage_recorder: SharedAiHubUsageRecorder,
    ) -> SharedAiHub {
        Arc::new(Self::new_with_usage_recorder(
            runtime,
            model_routing,
            usage_recorder,
        ))
    }

    pub fn model_routing(&self) -> &AiModelRouting {
        &self.model_routing
    }

    pub fn runtime_name(&self) -> &'static str {
        self.runtime.runtime_name()
    }

    pub fn model_for_route(&self, route: AiModelRoute) -> &str {
        match route {
            AiModelRoute::DefaultChat => &self.model_routing.default_chat,
            AiModelRoute::Reasoning => &self.model_routing.reasoning,
            AiModelRoute::Summarization => &self.model_routing.summarization,
            AiModelRoute::MailIntelligence => &self.model_routing.mail_intelligence,
            AiModelRoute::ReplyDraft => &self.model_routing.reply_draft,
            AiModelRoute::Extraction => &self.model_routing.extraction,
            AiModelRoute::Embeddings => &self.model_routing.embeddings,
            AiModelRoute::MeetingPrep => &self.model_routing.meeting_prep,
        }
    }

    pub fn target_for_route(&self, route: AiModelRoute) -> Option<&AiModelRouteTarget> {
        self.model_routing
            .targets
            .iter()
            .find(|target| target.capability_slot == route.as_str())
    }

    pub async fn chat(
        &self,
        route: AiModelRoute,
        prompt: &str,
    ) -> Result<AiChatResult, AiHubError> {
        let model = self.model_for_route(route);
        let started = Instant::now();
        let result = self.runtime.chat_with_model(prompt, model).await;
        let latency_ms = elapsed_ms(started);
        match result {
            Ok(result) => {
                self.record_usage(AiHubUsageEvent::completed_chat(
                    route,
                    self.target_for_route(route),
                    model,
                    prompt,
                    &result,
                    latency_ms,
                ))
                .await;
                Ok(result)
            }
            Err(error) => {
                self.record_usage(AiHubUsageEvent::failed(
                    route,
                    self.target_for_route(route),
                    model,
                    "chat",
                    prompt.chars().count(),
                    latency_ms,
                    &error.to_string(),
                ))
                .await;
                Err(AiHubError::from(error))
            }
        }
    }

    pub async fn chat_json(
        &self,
        route: AiModelRoute,
        prompt: &str,
    ) -> Result<AiChatResult, AiHubError> {
        let model = self.model_for_route(route);
        let started = Instant::now();
        let result = self.runtime.chat_json_with_model(prompt, model).await;
        let latency_ms = elapsed_ms(started);
        match result {
            Ok(result) => {
                self.record_usage(AiHubUsageEvent::completed_chat(
                    route,
                    self.target_for_route(route),
                    model,
                    prompt,
                    &result,
                    latency_ms,
                ))
                .await;
                Ok(result)
            }
            Err(error) => {
                self.record_usage(AiHubUsageEvent::failed(
                    route,
                    self.target_for_route(route),
                    model,
                    "chat",
                    prompt.chars().count(),
                    latency_ms,
                    &error.to_string(),
                ))
                .await;
                Err(AiHubError::from(error))
            }
        }
    }

    pub async fn translate_text(
        &self,
        text: &str,
        target_language: &str,
    ) -> Result<AiChatResult, AiHubError> {
        let prompt = format!(
            "Translate the following text to {target_language}. Return ONLY the translated text, no explanations:\n\n{text}"
        );
        self.chat(AiModelRoute::Summarization, &prompt).await
    }

    pub async fn embed(&self, input: &str) -> Result<AiEmbedResult, AiHubError> {
        let route = AiModelRoute::Embeddings;
        let model = self.model_for_route(route);
        let started = Instant::now();
        let result = self.runtime.embed_with_model(input, model).await;
        let latency_ms = elapsed_ms(started);
        match result {
            Ok(result) => {
                self.record_usage(AiHubUsageEvent::completed_embed(
                    self.target_for_route(route),
                    model,
                    input,
                    &result,
                    latency_ms,
                ))
                .await;
                Ok(result)
            }
            Err(error) => {
                self.record_usage(AiHubUsageEvent::failed(
                    route,
                    self.target_for_route(route),
                    model,
                    "embed",
                    input.chars().count(),
                    latency_ms,
                    &error.to_string(),
                ))
                .await;
                Err(AiHubError::from(error))
            }
        }
    }

    pub async fn version(&self) -> Result<Option<String>, AiHubError> {
        self.runtime.version().await.map_err(AiHubError::from)
    }

    pub async fn models(&self) -> Result<Vec<String>, AiHubError> {
        self.runtime.models().await.map_err(AiHubError::from)
    }

    async fn record_usage(&self, event: AiHubUsageEvent) {
        let Some(recorder) = &self.usage_recorder else {
            tracing::info!(
                route_slot = %event.route_slot,
                provider_id = event.provider_id.as_deref().unwrap_or("unknown"),
                model_key = %event.model_key,
                operation = %event.operation,
                status = %event.status,
                latency_ms = event.latency_ms,
                estimated_input_tokens = event.estimated_input_tokens,
                estimated_output_tokens = event.estimated_output_tokens,
                "AI Hub runtime call"
            );
            return;
        };

        if let Err(error) = recorder.record_ai_hub_usage(event).await {
            tracing::warn!(error = %error, "AI Hub usage trace persistence failed");
        }
    }

    pub fn inspect_text(text: &str) -> LocalAiInspection {
        let language = Self::detect_language(text);
        let sensitive_findings = Self::detect_sensitive_content(text);
        let sensitivity = sensitivity_for(&sensitive_findings);

        LocalAiInspection {
            source: LOCAL_AGENT_SOURCE.to_owned(),
            language,
            sensitivity,
            sensitive_findings,
        }
    }

    pub fn detect_language(text: &str) -> LocalLanguageDetection {
        let text = text.trim();
        if text.is_empty() {
            return LocalLanguageDetection {
                language: "unknown".to_owned(),
                confidence: 0.0,
                script: None,
                source: LOCAL_AGENT_SOURCE.to_owned(),
            };
        }

        let lower = text.to_lowercase();
        if text.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c)) {
            if lower.contains('ї') || lower.contains('є') {
                return local_language("uk", 0.85, Some("cyrillic"));
            }
            return local_language("ru", 0.90, Some("cyrillic"));
        }
        if text.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c)) {
            return local_language("zh", 0.70, Some("cjk"));
        }
        if lower.contains('ñ')
            || contains_any(
                &lower,
                &[
                    "hola",
                    "gracias",
                    "para",
                    "como",
                    "que",
                    "por favor",
                    "saludos",
                    "adjunto",
                ],
            )
        {
            return local_language("es", 0.85, Some("latin"));
        }
        if contains_any(&lower, &["privet", "spasibo", "pozhaluysta"]) {
            return local_language("ru", 0.55, Some("latin"));
        }
        if contains_any(
            &lower,
            &[
                "mit", "und", "der", "die", "das", "ist", "von", "für", "danke", "bitte",
            ],
        ) {
            return local_language("de", 0.65, Some("latin"));
        }
        if text.chars().any(|c| c.is_ascii_alphabetic()) {
            return local_language("en", 0.50, Some("latin"));
        }

        local_language("unknown", 0.0, None)
    }

    pub fn detect_sensitive_content(text: &str) -> Vec<SensitiveFinding> {
        let mut findings = Vec::new();
        detect_pem_blocks(text, &mut findings);
        detect_secret_assignments(text, &mut findings);
        detect_token_shapes(text, &mut findings);
        detect_financial_identifiers(text, &mut findings);
        findings
    }
}

pub type SharedAiHubUsageRecorder = Arc<dyn AiHubUsageRecorder>;

#[async_trait]
pub trait AiHubUsageRecorder: Send + Sync {
    async fn record_ai_hub_usage(&self, event: AiHubUsageEvent) -> Result<(), AiHubUsageError>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct AiHubUsageEvent {
    pub usage_event_id: String,
    pub provider_id: Option<String>,
    pub model_key: String,
    pub route_slot: String,
    pub operation: String,
    pub status: String,
    pub prompt_chars: i32,
    pub output_chars: Option<i32>,
    pub estimated_input_tokens: i32,
    pub estimated_output_tokens: Option<i32>,
    pub total_duration_ns: Option<i64>,
    pub latency_ms: i64,
    pub error_summary: Option<String>,
}

impl AiHubUsageEvent {
    fn completed_chat(
        route: AiModelRoute,
        target: Option<&AiModelRouteTarget>,
        requested_model: &str,
        prompt: &str,
        result: &AiChatResult,
        latency_ms: i64,
    ) -> Self {
        let prompt_chars = char_count_i32(prompt);
        let output_chars = char_count_i32(&result.content);
        Self {
            usage_event_id: usage_event_id(),
            provider_id: target.map(|target| target.provider_id.clone()),
            model_key: target
                .map(|target| target.model_key.clone())
                .unwrap_or_else(|| requested_model.to_owned()),
            route_slot: route.as_str().to_owned(),
            operation: "chat".to_owned(),
            status: "completed".to_owned(),
            prompt_chars,
            output_chars: Some(output_chars),
            estimated_input_tokens: estimate_tokens(prompt_chars),
            estimated_output_tokens: Some(estimate_tokens(output_chars)),
            total_duration_ns: result.total_duration_ns.and_then(u64_to_i64),
            latency_ms,
            error_summary: None,
        }
    }

    fn completed_embed(
        target: Option<&AiModelRouteTarget>,
        requested_model: &str,
        input: &str,
        result: &AiEmbedResult,
        latency_ms: i64,
    ) -> Self {
        let input_chars = char_count_i32(input);
        Self {
            usage_event_id: usage_event_id(),
            provider_id: target.map(|target| target.provider_id.clone()),
            model_key: target
                .map(|target| target.model_key.clone())
                .unwrap_or_else(|| requested_model.to_owned()),
            route_slot: AiModelRoute::Embeddings.as_str().to_owned(),
            operation: "embed".to_owned(),
            status: "completed".to_owned(),
            prompt_chars: input_chars,
            output_chars: None,
            estimated_input_tokens: estimate_tokens(input_chars),
            estimated_output_tokens: None,
            total_duration_ns: result.total_duration_ns.and_then(u64_to_i64),
            latency_ms,
            error_summary: None,
        }
    }

    fn failed(
        route: AiModelRoute,
        target: Option<&AiModelRouteTarget>,
        requested_model: &str,
        operation: &str,
        prompt_chars: usize,
        latency_ms: i64,
        error: &str,
    ) -> Self {
        let prompt_chars = usize_to_i32(prompt_chars);
        Self {
            usage_event_id: usage_event_id(),
            provider_id: target.map(|target| target.provider_id.clone()),
            model_key: target
                .map(|target| target.model_key.clone())
                .unwrap_or_else(|| requested_model.to_owned()),
            route_slot: route.as_str().to_owned(),
            operation: operation.to_owned(),
            status: "failed".to_owned(),
            prompt_chars,
            output_chars: None,
            estimated_input_tokens: estimate_tokens(prompt_chars),
            estimated_output_tokens: None,
            total_duration_ns: None,
            latency_ms,
            error_summary: Some(truncate_error(error)),
        }
    }
}

#[derive(Debug, Error)]
#[error("AI Hub usage recorder failed: {0}")]
pub struct AiHubUsageError(pub String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiModelRoute {
    DefaultChat,
    Reasoning,
    Summarization,
    MailIntelligence,
    ReplyDraft,
    Extraction,
    Embeddings,
    MeetingPrep,
}

impl AiModelRoute {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DefaultChat => "default_chat",
            Self::Reasoning => "reasoning",
            Self::Summarization => "summarization",
            Self::MailIntelligence => "mail_intelligence",
            Self::ReplyDraft => "reply_draft",
            Self::Extraction => "extraction",
            Self::Embeddings => "embeddings",
            Self::MeetingPrep => "meeting_prep",
        }
    }
}

#[derive(Debug, Error)]
pub enum AiHubError {
    #[error(transparent)]
    Runtime(#[from] AiRuntimePortError),
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LocalAiInspection {
    pub source: String,
    pub language: LocalLanguageDetection,
    pub sensitivity: SensitivityLevel,
    pub sensitive_findings: Vec<SensitiveFinding>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LocalLanguageDetection {
    pub language: String,
    pub confidence: f32,
    pub script: Option<String>,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SensitiveFinding {
    pub kind: String,
    pub severity: SensitivityLevel,
    pub confidence: f32,
    pub evidence: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SensitivityLevel {
    Public,
    Low,
    Medium,
    High,
    Critical,
}
