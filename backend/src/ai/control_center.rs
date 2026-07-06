mod availability;
mod catalog;
mod errors;
mod evidence;
mod models;
mod presets;
mod prompts;
mod provider_auth;
mod providers;
mod routes;
mod rows;
mod store;
#[cfg(test)]
mod tests;
mod validation;
mod vault;

pub use errors::AiControlCenterError;
pub(crate) use models::AiProviderVaultRestore;
pub use models::{
    AiCapabilitySlot, AiModelAvailabilityUpdateRequest, AiModelCatalogItem, AiModelRoute,
    AiModelRouteUpdateRequest, AiPromptActivateRequest, AiPromptCreateRequest, AiPromptEvalRun,
    AiPromptTemplate, AiPromptTestRequest, AiPromptVersion, AiPromptVersionCreateRequest,
    AiProviderAccount, AiProviderAuthPendingGrant, AiProviderAuthStartRequest,
    AiProviderAuthStartResponse, AiProviderAuthStatusResponse, AiProviderCommandKind,
    AiProviderCommandResponse, AiProviderConsentRequest, AiProviderCreateRequest,
    AiProviderPatchRequest, AiProviderPreset, AiSettingsOverviewResponse,
};
pub use presets::{BUILT_IN_OLLAMA_PROVIDER_ID, OLLAMA_CHAT_MODEL, OLLAMA_EMBEDDING_MODEL};
pub(crate) use provider_auth::{connect_pending_ai_provider_auth, start_local_provider_auth};
pub use store::AiControlCenterStore;
pub use vault::store_api_key_in_host_vault;
