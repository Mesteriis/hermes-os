use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use crate::ai::core::{AiModelRouteTarget, AiModelRouting};
use crate::platform::ai_runtime::{
    AiChatResult, AiEmbedResult, AiRuntimePortError, SharedAiRuntimePort,
};

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

fn local_language(language: &str, confidence: f32, script: Option<&str>) -> LocalLanguageDetection {
    LocalLanguageDetection {
        language: language.to_owned(),
        confidence,
        script: script.map(str::to_owned),
        source: LOCAL_AGENT_SOURCE.to_owned(),
    }
}

fn usage_event_id() -> String {
    format!("ai_hub_usage_{}", Uuid::now_v7())
}

fn elapsed_ms(started: Instant) -> i64 {
    i64::try_from(started.elapsed().as_millis()).unwrap_or(i64::MAX)
}

fn char_count_i32(value: &str) -> i32 {
    usize_to_i32(value.chars().count())
}

fn usize_to_i32(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

fn estimate_tokens(chars: i32) -> i32 {
    if chars <= 0 {
        return 0;
    }
    (chars + 3) / 4
}

fn u64_to_i64(value: u64) -> Option<i64> {
    i64::try_from(value).ok()
}

fn truncate_error(error: &str) -> String {
    const LIMIT: usize = 240;
    let mut value = error.trim().replace(['\n', '\r'], " ");
    if value.chars().count() > LIMIT {
        value = value.chars().take(LIMIT).collect();
    }
    value
}

fn detect_pem_blocks(text: &str, findings: &mut Vec<SensitiveFinding>) {
    let upper = text.to_ascii_uppercase();
    if upper.contains("-----BEGIN ")
        && (upper.contains("PRIVATE KEY-----")
            || upper.contains("RSA PRIVATE KEY-----")
            || upper.contains("OPENSSH PRIVATE KEY-----"))
    {
        push_finding(
            findings,
            "private_key_pem",
            SensitivityLevel::Critical,
            0.99,
            "PEM private key marker",
        );
    }
    if upper.contains("-----BEGIN ") && upper.contains("CERTIFICATE-----") {
        push_finding(
            findings,
            "certificate_pem",
            SensitivityLevel::Public,
            0.95,
            "PEM certificate marker",
        );
    }
    if text.contains("ssh-ed25519") || text.contains("ssh-rsa") {
        push_finding(
            findings,
            "ssh_public_key",
            SensitivityLevel::Public,
            0.90,
            "SSH public key marker",
        );
    }
}

fn detect_secret_assignments(text: &str, findings: &mut Vec<SensitiveFinding>) {
    for line in text.lines() {
        let Some((label, value)) = explicit_secret_assignment(line) else {
            continue;
        };
        if is_placeholder_secret_value(value) {
            continue;
        }
        push_finding(
            findings,
            "secret_assignment",
            SensitivityLevel::Critical,
            0.85,
            &bounded_evidence(&label),
        );
    }
}

fn detect_token_shapes(text: &str, findings: &mut Vec<SensitiveFinding>) {
    for line in text.lines() {
        let has_secret_context = has_secret_context(line);
        for raw_token in line.split(|c: char| c.is_whitespace() || "'\"`<>()[]{};,".contains(c)) {
            let token = raw_token.trim_matches(|c: char| {
                !(c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | '+' | '='))
            });
            if token.len() < 8 {
                continue;
            }

            if looks_like_github_token(token) {
                push_finding(
                    findings,
                    "github_token",
                    SensitivityLevel::Critical,
                    0.95,
                    &masked_token(token),
                );
                continue;
            }
            if looks_like_aws_access_key(token) {
                push_finding(
                    findings,
                    "aws_access_key_id",
                    SensitivityLevel::Critical,
                    0.95,
                    &masked_token(token),
                );
                continue;
            }
            if looks_like_google_api_key(token) {
                push_finding(
                    findings,
                    "google_api_key",
                    SensitivityLevel::Critical,
                    0.90,
                    &masked_token(token),
                );
                continue;
            }
            if looks_like_jwt(token) {
                push_finding(
                    findings,
                    "jwt",
                    SensitivityLevel::High,
                    0.85,
                    &masked_token(token),
                );
                continue;
            }
            if has_secret_context && looks_like_high_entropy_secret(token) {
                push_finding(
                    findings,
                    "high_entropy_token",
                    SensitivityLevel::High,
                    0.70,
                    &masked_token(token),
                );
            }
        }
    }
}

fn has_secret_context(line: &str) -> bool {
    if explicit_secret_assignment(line)
        .is_some_and(|(_, value)| !is_placeholder_secret_value(value))
    {
        return true;
    }
    line.to_ascii_lowercase().contains("authorization: bearer ")
}

fn explicit_secret_assignment(line: &str) -> Option<(String, &str)> {
    let (label, value) = line.split_once('=').or_else(|| line.split_once(':'))?;
    if value.trim().is_empty() {
        return None;
    }
    let label = label.trim().strip_prefix("export ").unwrap_or(label.trim());
    if label.bytes().any(|byte| byte.is_ascii_whitespace()) {
        return None;
    }
    let normalized = label
        .chars()
        .filter_map(|character| match character {
            'A'..='Z' => Some(character.to_ascii_lowercase()),
            'a'..='z' | '0'..='9' | '_' => Some(character),
            '-' => Some('_'),
            _ => None,
        })
        .collect::<String>();
    matches!(
        normalized.as_str(),
        "password"
            | "passwd"
            | "secret"
            | "token"
            | "api_key"
            | "apikey"
            | "private_key"
            | "client_secret"
            | "access_token"
    )
    .then_some((normalized, value.trim()))
}

fn is_placeholder_secret_value(value: &str) -> bool {
    let value = value
        .trim()
        .trim_matches(|character| matches!(character, '\'' | '"' | '`' | '<' | '>'))
        .to_ascii_lowercase();
    matches!(
        value.as_str(),
        "example"
            | "example-value"
            | "example_value"
            | "changeme"
            | "change-me"
            | "redacted"
            | "placeholder"
            | "replace-me"
            | "replace_me"
            | "your-api-key"
            | "your_api_key"
            | "your-token"
            | "your_token"
    )
}

fn detect_financial_identifiers(text: &str, findings: &mut Vec<SensitiveFinding>) {
    let mut card_candidate = String::new();
    let mut separator_before_next_digit = false;
    let mut has_internal_separator = false;
    for ch in text.chars() {
        if ch.is_ascii_digit() {
            if separator_before_next_digit && !card_candidate.is_empty() {
                has_internal_separator = true;
            }
            card_candidate.push(ch);
            separator_before_next_digit = false;
            continue;
        }
        if matches!(ch, ' ' | '-') && !card_candidate.is_empty() {
            separator_before_next_digit = true;
            continue;
        }
        flush_payment_card_candidate(&card_candidate, has_internal_separator, findings);
        card_candidate.clear();
        separator_before_next_digit = false;
        has_internal_separator = false;
    }
    flush_payment_card_candidate(&card_candidate, has_internal_separator, findings);

    for raw_token in text.split_whitespace() {
        let compact: String = raw_token
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_uppercase();
        if compact.len() >= 15
            && compact.len() <= 34
            && looks_like_iban(&compact)
            && iban_checksum_valid(&compact)
        {
            push_finding(
                findings,
                "iban",
                SensitivityLevel::High,
                0.80,
                &masked_token(&compact),
            );
        }
    }
}

fn flush_payment_card_candidate(
    digits: &str,
    has_internal_separator: bool,
    findings: &mut Vec<SensitiveFinding>,
) {
    if has_internal_separator && (13..=19).contains(&digits.len()) && luhn_valid(digits) {
        push_finding(
            findings,
            "payment_card_number",
            SensitivityLevel::High,
            0.85,
            &masked_token(digits),
        );
    }
}

fn looks_like_github_token(token: &str) -> bool {
    ["ghp_", "gho_", "ghu_", "ghs_", "ghr_", "github_pat_"]
        .iter()
        .any(|prefix| token.starts_with(prefix))
        && token.len() >= 20
}

fn looks_like_aws_access_key(token: &str) -> bool {
    (token.starts_with("AKIA") || token.starts_with("ASIA"))
        && token.len() == 20
        && token
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

fn looks_like_google_api_key(token: &str) -> bool {
    token.starts_with("AIza") && token.len() >= 24
}

fn looks_like_jwt(token: &str) -> bool {
    let parts: Vec<&str> = token.split('.').collect();
    parts.len() == 3
        && parts[0].len() >= 8
        && parts[1].len() >= 8
        && parts.iter().all(|part| {
            part.chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '='))
        })
}

fn looks_like_high_entropy_secret(token: &str) -> bool {
    if token.len() < 24 || token.len() > 256 {
        return false;
    }
    let has_lower = token.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = token.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = token.chars().any(|c| c.is_ascii_digit());
    let has_symbol = token
        .chars()
        .any(|c| matches!(c, '_' | '-' | '.' | '/' | '+' | '='));
    let class_count = [has_lower, has_upper, has_digit, has_symbol]
        .into_iter()
        .filter(|value| *value)
        .count();
    class_count >= 3 && shannon_entropy(token) >= 4.0
}

fn looks_like_iban(value: &str) -> bool {
    value.len() >= 15
        && value.len() <= 34
        && value.chars().take(2).all(|c| c.is_ascii_uppercase())
        && value.chars().skip(2).take(2).all(|c| c.is_ascii_digit())
        && value.chars().skip(4).all(|c| c.is_ascii_alphanumeric())
}

fn iban_checksum_valid(value: &str) -> bool {
    let mut remainder = 0u32;
    for byte in value[4..].bytes().chain(value[..4].bytes()) {
        let numeric = match byte {
            b'0'..=b'9' => u32::from(byte - b'0'),
            b'A'..=b'Z' => u32::from(byte - b'A') + 10,
            _ => return false,
        };
        if numeric >= 10 {
            remainder = (remainder * 10 + numeric / 10) % 97;
            remainder = (remainder * 10 + numeric % 10) % 97;
        } else {
            remainder = (remainder * 10 + numeric) % 97;
        }
    }
    remainder == 1
}

fn luhn_valid(digits: &str) -> bool {
    let mut sum = 0u32;
    let mut double = false;
    for ch in digits.chars().rev() {
        let Some(mut digit) = ch.to_digit(10) else {
            return false;
        };
        if double {
            digit *= 2;
            if digit > 9 {
                digit -= 9;
            }
        }
        sum += digit;
        double = !double;
    }
    sum > 0 && sum.is_multiple_of(10)
}

fn shannon_entropy(value: &str) -> f32 {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return 0.0;
    }
    let mut counts = [0usize; 256];
    for byte in bytes {
        counts[*byte as usize] += 1;
    }
    let len = bytes.len() as f32;
    counts
        .into_iter()
        .filter(|count| *count > 0)
        .map(|count| {
            let p = count as f32 / len;
            -p * p.log2()
        })
        .sum()
}

fn sensitivity_for(findings: &[SensitiveFinding]) -> SensitivityLevel {
    findings
        .iter()
        .map(|finding| finding.severity)
        .max()
        .unwrap_or(SensitivityLevel::Public)
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn push_finding(
    findings: &mut Vec<SensitiveFinding>,
    kind: &str,
    severity: SensitivityLevel,
    confidence: f32,
    evidence: &str,
) {
    if findings
        .iter()
        .any(|finding| finding.kind == kind && finding.evidence == evidence)
    {
        return;
    }
    findings.push(SensitiveFinding {
        kind: kind.to_owned(),
        severity,
        confidence,
        evidence: evidence.to_owned(),
    });
}

fn masked_token(token: &str) -> String {
    let chars: Vec<char> = token.chars().collect();
    if chars.len() <= 12 {
        return "<redacted>".to_owned();
    }
    let head: String = chars.iter().take(4).collect();
    let tail: String = chars
        .iter()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{head}…{tail}")
}

fn bounded_evidence(value: &str) -> String {
    value.chars().take(80).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_pem_private_key_without_exposing_value() {
        let text = [
            "-----BEGIN ",
            "RSA PRIVATE KEY-----\nredacted\n-----END RSA PRIVATE KEY-----",
        ]
        .concat();
        let findings = AiHub::detect_sensitive_content(&text);
        assert!(
            findings
                .iter()
                .any(|finding| finding.kind == "private_key_pem")
        );
    }

    #[test]
    fn detects_credit_card_with_luhn_check() {
        let findings = AiHub::detect_sensitive_content("card 4111 1111 1111 1111");
        assert!(
            findings
                .iter()
                .any(|finding| finding.kind == "payment_card_number")
        );
    }

    #[test]
    fn does_not_classify_an_unlabelled_luhn_tracking_number_as_a_payment_card() {
        let findings = AiHub::detect_sensitive_content("Shipment reference 4111111111111111");

        assert!(
            !findings
                .iter()
                .any(|finding| finding.kind == "payment_card_number")
        );
    }

    #[test]
    fn does_not_classify_an_iban_shaped_tracking_value_without_a_valid_checksum() {
        let findings = AiHub::detect_sensitive_content("Tracking reference DE89370400440532013001");

        assert!(!findings.iter().any(|finding| finding.kind == "iban"));
    }

    #[test]
    fn detects_a_checksum_valid_iban() {
        let findings = AiHub::detect_sensitive_content("IBAN: DE89370400440532013000");

        assert!(findings.iter().any(|finding| finding.kind == "iban"));
    }

    #[test]
    fn does_not_classify_unlabelled_tracking_identifier_as_secret() {
        let findings = AiHub::detect_sensitive_content(
            "https://example.test/click?tracking=Abc9-Def8_Ghi7.Jkl6/Mno5Pqr4",
        );

        assert!(
            !findings
                .iter()
                .any(|finding| finding.kind == "high_entropy_token")
        );
    }

    #[test]
    fn does_not_classify_password_reset_copy_as_a_secret_assignment() {
        let findings = AiHub::detect_sensitive_content(
            "Reset your password: use the link in this email to choose a new one.",
        );

        assert!(
            !findings
                .iter()
                .any(|finding| finding.kind == "secret_assignment")
        );
    }

    #[test]
    fn detects_explicit_secret_assignment_keys() {
        let findings = AiHub::detect_sensitive_content("API_KEY=actual-secret-value-123");

        assert!(
            findings
                .iter()
                .any(|finding| finding.kind == "secret_assignment")
        );
    }

    #[test]
    fn ignores_documented_placeholder_secret_assignments() {
        let inspection = AiHub::inspect_text(
            "Configuration example:\nAPI_KEY=example-value\nTOKEN=<your-token>",
        );

        assert_eq!(inspection.sensitivity, SensitivityLevel::Public);
        assert!(inspection.sensitive_findings.is_empty());
    }

    #[test]
    fn does_not_escalate_public_certificate_or_ssh_key_markers() {
        let inspection = AiHub::inspect_text(
            "-----BEGIN CERTIFICATE-----\npublic certificate\n-----END CERTIFICATE-----\nssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIexample",
        );

        assert_eq!(inspection.sensitivity, SensitivityLevel::Public);
        assert!(
            inspection
                .sensitive_findings
                .iter()
                .all(|finding| finding.severity == SensitivityLevel::Public)
        );
    }

    #[test]
    fn detects_language_locally() {
        let detection = AiHub::detect_language("Привет, как дела?");
        assert_eq!(detection.language, "ru");
    }
}
