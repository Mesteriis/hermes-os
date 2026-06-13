mod catalog;
mod errors;
mod models;
mod presets;
mod prompts;
mod providers;
mod routes;
mod rows;
mod store;
#[cfg(test)]
mod tests;
mod validation;
mod vault;

pub use errors::AiControlCenterError;
pub use models::{
    AiCapabilitySlot, AiModelCatalogItem, AiModelRoute, AiModelRouteUpdateRequest,
    AiPromptActivateRequest, AiPromptCreateRequest, AiPromptEvalRun, AiPromptTemplate,
    AiPromptTestRequest, AiPromptVersion, AiPromptVersionCreateRequest, AiProviderAccount,
    AiProviderCommandKind, AiProviderCommandResponse, AiProviderConsentRequest,
    AiProviderCreateRequest, AiProviderPatchRequest, AiProviderPreset, AiSettingsOverviewResponse,
};
pub use presets::{BUILT_IN_OLLAMA_PROVIDER_ID, OLLAMA_CHAT_MODEL, OLLAMA_EMBEDDING_MODEL};
pub use store::AiControlCenterStore;
pub use vault::store_api_key_in_host_vault;
