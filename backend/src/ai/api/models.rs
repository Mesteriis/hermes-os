use serde::Serialize;

use crate::ai::control_center::models::{AiModelCatalogItem, AiPromptTemplate, AiProviderAccount};

#[derive(Serialize)]
pub(crate) struct AiProviderListResponse {
    pub(crate) items: Vec<AiProviderAccount>,
}

#[derive(Serialize)]
pub(crate) struct AiModelListResponse {
    pub(crate) items: Vec<AiModelCatalogItem>,
}

#[derive(Serialize)]
pub(crate) struct AiPromptListResponse {
    pub(crate) items: Vec<AiPromptTemplate>,
}
