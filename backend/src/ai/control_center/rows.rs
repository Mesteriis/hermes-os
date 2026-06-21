use sqlx::Row;
use sqlx::postgres::PgRow;

use super::errors::AiControlCenterError;
use super::models::{
    AiModelCatalogItem, AiModelRoute, AiPromptEvalRun, AiPromptTemplate, AiPromptVersion,
    AiProviderAccount,
};
use super::validation::{json_array, json_string_array};

pub(super) fn row_to_provider(row: PgRow) -> Result<AiProviderAccount, AiControlCenterError> {
    Ok(AiProviderAccount {
        provider_id: row.try_get("provider_id")?,
        provider_kind: row.try_get("provider_kind")?,
        provider_key: row.try_get("provider_key")?,
        display_name: row.try_get("display_name")?,
        status: row.try_get("status")?,
        consent_state: row.try_get("consent_state")?,
        consented_at: row.try_get("consented_at")?,
        config: row.try_get("config")?,
        capabilities: json_string_array(row.try_get("capabilities")?)?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_model(row: PgRow) -> Result<AiModelCatalogItem, AiControlCenterError> {
    Ok(AiModelCatalogItem {
        provider_id: row.try_get("provider_id")?,
        model_key: row.try_get("model_key")?,
        display_name: row.try_get("display_name")?,
        category: row.try_get("category")?,
        privacy: row.try_get("privacy")?,
        capabilities: json_string_array(row.try_get("capabilities")?)?,
        context_window: row.try_get("context_window")?,
        embedding_dimension: row.try_get("embedding_dimension")?,
        is_available: row.try_get("is_available")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_route(row: PgRow) -> Result<AiModelRoute, AiControlCenterError> {
    Ok(AiModelRoute {
        capability_slot: row.try_get("capability_slot")?,
        provider_id: row.try_get("provider_id")?,
        model_key: row.try_get("model_key")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_prompt(row: PgRow) -> Result<AiPromptTemplate, AiControlCenterError> {
    Ok(AiPromptTemplate {
        prompt_id: row.try_get("prompt_id")?,
        name: row.try_get("name")?,
        entity_scope: row.try_get("entity_scope")?,
        capability_slot: row.try_get("capability_slot")?,
        description: row.try_get("description")?,
        is_system: row.try_get("is_system")?,
        active_version_id: row.try_get("active_version_id")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_prompt_version(row: PgRow) -> Result<AiPromptVersion, AiControlCenterError> {
    Ok(AiPromptVersion {
        prompt_version_id: row.try_get("prompt_version_id")?,
        prompt_id: row.try_get("prompt_id")?,
        version_label: row.try_get("version_label")?,
        body_template: row.try_get("body_template")?,
        variables: json_string_array(row.try_get("variables")?)?,
        status: row.try_get("status")?,
        created_by_actor_id: row.try_get("created_by_actor_id")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn row_to_eval_run(row: PgRow) -> Result<AiPromptEvalRun, AiControlCenterError> {
    Ok(AiPromptEvalRun {
        eval_run_id: row.try_get("eval_run_id")?,
        prompt_id: row.try_get("prompt_id")?,
        prompt_version_id: row.try_get("prompt_version_id")?,
        provider_id: row.try_get("provider_id")?,
        model_key: row.try_get("model_key")?,
        source_refs: json_array(row.try_get("source_refs")?)?,
        variables: row.try_get("variables")?,
        output_text: row.try_get("output_text")?,
        score: row.try_get("score")?,
        notes: row.try_get("notes")?,
        actor_id: row.try_get("actor_id")?,
        created_at: row.try_get("created_at")?,
    })
}
