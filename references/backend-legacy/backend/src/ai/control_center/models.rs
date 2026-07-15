use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::AiControlCenterError;
use super::validation::{
    string_array_value, validate_capability_slot, validate_cli_preset, validate_entity_scope,
    validate_non_empty, validate_provider_kind,
};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiSettingsOverviewResponse {
    pub providers: Vec<AiProviderAccount>,
    pub models: Vec<AiModelCatalogItem>,
    pub routes: Vec<AiModelRoute>,
    pub prompts: Vec<AiPromptTemplate>,
    pub eval_runs: Vec<AiPromptEvalRun>,
    pub capability_slots: Vec<AiCapabilitySlot>,
    pub provider_presets: Vec<AiProviderPreset>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiHubUsageStatsResponse {
    pub generated_at: DateTime<Utc>,
    pub window_hours: i32,
    pub totals: AiHubUsageTotals,
    pub providers: Vec<AiHubProviderUsageStats>,
    pub hourly: Vec<AiHubHourlyUsageStats>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiHubUsageTotals {
    pub request_count: i64,
    pub completed_count: i64,
    pub failed_count: i64,
    pub estimated_tokens: i64,
    pub estimated_cost_usd: Option<f64>,
    pub avg_latency_ms: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiHubProviderUsageStats {
    pub provider_id: String,
    pub provider_kind: String,
    pub provider_key: String,
    pub display_name: String,
    pub status: String,
    pub request_count: i64,
    pub completed_count: i64,
    pub failed_count: i64,
    pub estimated_tokens: i64,
    pub estimated_cost_usd: Option<f64>,
    pub avg_latency_ms: Option<f64>,
    pub balance_remaining_usd: Option<f64>,
    pub token_quota_remaining: Option<i64>,
    pub last_request_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiHubHourlyUsageStats {
    pub hour: DateTime<Utc>,
    pub provider_id: String,
    pub request_count: i64,
    pub failed_count: i64,
    pub estimated_tokens: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiCapabilitySlot {
    pub slot: String,
    pub label: String,
    pub description: String,
    pub requires_embedding_dimension: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderPreset {
    pub provider_kind: String,
    pub provider_key: String,
    pub display_name: String,
    pub privacy: String,
    pub base_url: Option<String>,
    pub command_preset: Option<String>,
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderAuthStartRequest {
    pub provider_kind: String,
    pub provider_key: String,
    pub display_name: Option<String>,
    pub callback_url: String,
}

impl AiProviderAuthStartRequest {
    pub(super) fn validate(&self) -> Result<(), AiControlCenterError> {
        let provider_kind = self.provider_kind.trim();
        validate_provider_kind(provider_kind)?;
        if provider_kind == "api" {
            return Err(AiControlCenterError::InvalidRequest(
                "API providers use API-token setup, not local callback authorization".to_owned(),
            ));
        }
        validate_non_empty("provider_key", &self.provider_key)?;
        validate_non_empty("callback_url", &self.callback_url)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiProviderAuthPendingGrant {
    pub setup_id: String,
    pub state: String,
    pub provider_id: String,
    pub provider_kind: String,
    pub provider_key: String,
    pub display_name: String,
    pub callback_url: String,
    pub login_command: Option<String>,
    pub status: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl AiProviderAuthPendingGrant {
    pub fn response(&self, provider: Option<AiProviderAccount>) -> AiProviderAuthStartResponse {
        AiProviderAuthStartResponse {
            setup_id: self.setup_id.clone(),
            provider_id: self.provider_id.clone(),
            provider_kind: self.provider_kind.clone(),
            provider_key: self.provider_key.clone(),
            display_name: self.display_name.clone(),
            callback_url: self.callback_url.clone(),
            login_command: self.login_command.clone(),
            status: self.status.clone(),
            message: self.message.clone(),
            expires_at: self.expires_at,
            provider,
        }
    }

    pub fn status_response(
        &self,
        provider: Option<AiProviderAccount>,
    ) -> AiProviderAuthStatusResponse {
        AiProviderAuthStatusResponse {
            setup_id: self.setup_id.clone(),
            provider_id: self.provider_id.clone(),
            provider_kind: self.provider_kind.clone(),
            provider_key: self.provider_key.clone(),
            display_name: self.display_name.clone(),
            callback_url: self.callback_url.clone(),
            login_command: self.login_command.clone(),
            status: self.status.clone(),
            message: self.message.clone(),
            expires_at: self.expires_at,
            provider,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AiProviderAuthStartResponse {
    pub setup_id: String,
    pub provider_id: String,
    pub provider_kind: String,
    pub provider_key: String,
    pub display_name: String,
    pub callback_url: String,
    pub login_command: Option<String>,
    pub status: String,
    pub message: String,
    pub expires_at: DateTime<Utc>,
    pub provider: Option<AiProviderAccount>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AiProviderAuthStatusResponse {
    pub setup_id: String,
    pub provider_id: String,
    pub provider_kind: String,
    pub provider_key: String,
    pub display_name: String,
    pub callback_url: String,
    pub login_command: Option<String>,
    pub status: String,
    pub message: String,
    pub expires_at: DateTime<Utc>,
    pub provider: Option<AiProviderAccount>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderAccount {
    pub provider_id: String,
    pub provider_kind: String,
    pub provider_key: String,
    pub display_name: String,
    pub status: String,
    pub consent_state: String,
    pub consented_at: Option<DateTime<Utc>>,
    pub config: Value,
    pub capabilities: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderCreateRequest {
    pub provider_id: Option<String>,
    pub provider_kind: String,
    pub provider_key: String,
    pub display_name: String,
    pub base_url: Option<String>,
    pub command_preset: Option<String>,
    pub config: Option<Value>,
    pub capabilities: Option<Vec<String>>,
    pub enabled: Option<bool>,
    pub remote_context_consent: Option<bool>,
    #[serde(default, skip_serializing)]
    pub api_key: Option<String>,
}

impl AiProviderCreateRequest {
    pub(super) fn validate(&self) -> Result<(), AiControlCenterError> {
        let provider_kind = self.provider_kind.trim();
        validate_provider_kind(provider_kind)?;
        validate_non_empty("provider_key", &self.provider_key)?;
        validate_non_empty("display_name", &self.display_name)?;
        if provider_kind != "api" && has_api_key(&self.api_key) {
            return Err(AiControlCenterError::InvalidRequest(
                "API keys can only be configured for API providers".to_owned(),
            ));
        }
        if provider_kind == "cli" {
            let preset = self.command_preset.as_deref().ok_or_else(|| {
                AiControlCenterError::InvalidRequest(
                    "CLI provider requires command_preset".to_owned(),
                )
            })?;
            validate_cli_preset(preset)?;
        }
        Ok(())
    }
}

fn has_api_key(api_key: &Option<String>) -> bool {
    api_key
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderPatchRequest {
    pub display_name: Option<String>,
    pub base_url: Option<String>,
    pub config: Option<Value>,
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing)]
    pub api_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AiProviderVaultRestore {
    pub(crate) provider_id: String,
    pub(crate) provider_kind: String,
    pub(crate) provider_key: String,
    pub(crate) display_name: String,
    pub(crate) status: String,
    pub(crate) consent_state: String,
    pub(crate) config: Value,
    pub(crate) capabilities: Vec<String>,
    pub(crate) secret_ref: String,
    pub(crate) secret_purpose: String,
    pub(crate) secret_metadata: Value,
    pub(crate) secret_label: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderConsentRequest {
    pub consented: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderCommandResponse {
    pub provider_id: String,
    pub command: String,
    pub status: String,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiProviderCommandKind {
    Test,
    SyncModels,
}

impl AiProviderCommandKind {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Test => "test",
            Self::SyncModels => "sync_models",
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiModelCatalogItem {
    pub provider_id: String,
    pub model_key: String,
    pub display_name: String,
    pub category: String,
    pub privacy: String,
    pub capabilities: Vec<String>,
    pub context_window: Option<i32>,
    pub embedding_dimension: Option<i32>,
    pub is_available: bool,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiModelAvailabilityUpdateRequest {
    pub provider_id: String,
    pub model_key: String,
    pub is_available: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiModelDownloadRequest {
    pub provider_id: String,
    pub model_key: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiModelRoute {
    pub capability_slot: String,
    pub provider_id: String,
    pub model_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiModelRouteUpdateRequest {
    pub provider_id: String,
    pub model_key: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptTemplate {
    pub prompt_id: String,
    pub name: String,
    pub entity_scope: String,
    pub capability_slot: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub active_version_id: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptCreateRequest {
    pub prompt_id: Option<String>,
    pub name: String,
    pub entity_scope: String,
    pub capability_slot: String,
    pub description: Option<String>,
    pub metadata: Option<Value>,
}

impl AiPromptCreateRequest {
    pub(super) fn validate(&self) -> Result<(), AiControlCenterError> {
        validate_non_empty("name", &self.name)?;
        validate_entity_scope(&self.entity_scope)?;
        validate_capability_slot(&self.capability_slot)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptVersion {
    pub prompt_version_id: String,
    pub prompt_id: String,
    pub version_label: String,
    pub body_template: String,
    pub variables: Vec<String>,
    pub status: String,
    pub created_by_actor_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptVersionCreateRequest {
    pub prompt_version_id: Option<String>,
    pub version_label: Option<String>,
    pub body_template: String,
    pub variables: Vec<String>,
}

impl AiPromptVersionCreateRequest {
    pub(super) fn validate(&self) -> Result<(), AiControlCenterError> {
        validate_non_empty("body_template", &self.body_template)?;
        let _ = string_array_value(self.variables.clone(), "variables")?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptActivateRequest {
    pub prompt_version_id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptTestRequest {
    pub prompt_version_id: String,
    pub provider_id: String,
    pub model_key: String,
    pub variables: Value,
    pub source_refs: Option<Vec<Value>>,
    pub score: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiPromptEvalRun {
    pub eval_run_id: String,
    pub prompt_id: String,
    pub prompt_version_id: String,
    pub provider_id: String,
    pub model_key: String,
    pub source_refs: Vec<Value>,
    pub variables: Value,
    pub output_text: String,
    pub score: Option<i32>,
    pub notes: Option<String>,
    pub actor_id: String,
    pub created_at: DateTime<Utc>,
}
