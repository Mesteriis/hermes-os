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
        validate_provider_kind(&self.provider_kind)?;
        validate_non_empty("provider_key", &self.provider_key)?;
        validate_non_empty("display_name", &self.display_name)?;
        if self.provider_kind == "cli" {
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AiProviderPatchRequest {
    pub display_name: Option<String>,
    pub base_url: Option<String>,
    pub config: Option<Value>,
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing)]
    pub api_key: Option<String>,
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
