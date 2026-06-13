use super::*;

#[derive(Deserialize)]
pub(crate) struct RenderTemplateRequest {
    pub(super) template_id: String,
    pub(super) variables: Option<HashMap<String, String>>,
}
pub(crate) async fn get_v1_rich_templates(
    State(_state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(serde_json::json!({"templates": []})))
}
pub(crate) async fn post_v1_rich_template(
    State(_state): State<AppState>,
    Json(_req): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(serde_json::json!({"saved": true})))
}

pub(crate) async fn get_v1_blockers()
-> Result<Json<Vec<crate::domains::mail::blockers::ArchitectureBlocker>>, ApiError> {
    Ok(Json(crate::domains::mail::blockers::list_blockers()))
}

pub(crate) async fn post_v1_render_template(
    State(_state): State<AppState>,
    Json(req): Json<RenderTemplateRequest>,
) -> Result<Json<Value>, ApiError> {
    let template_id = req.template_id;
    let vars = req.variables.unwrap_or_default();
    Ok(Json(
        serde_json::json!({"rendered": true, "template_id": template_id, "variables": vars}),
    ))
}

#[derive(Deserialize)]
pub(crate) struct PersonListQuery {
    pub(super) favorites_only: Option<bool>,
    pub(super) limit: Option<i64>,
}

pub(crate) async fn get_v1_status(
    State(state): State<AppState>,
) -> Result<Json<V1StatusResponse>, ApiError> {
    let Some(_pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(Json(V1StatusResponse {
        version: "1.0",
        surfaces: V1Surfaces {
            messages: true,
            persons: true,
            search: true,
            documents: true,
            account_setup: true,
        },
        vault_status: state.vault.status()?,
    }))
}

pub(crate) async fn get_v1_vault_status(
    State(state): State<AppState>,
) -> Result<Json<crate::vault::VaultStatus>, ApiError> {
    Ok(Json(state.vault.status()?))
}

#[derive(Deserialize)]
pub(crate) struct VaultEntropyBatchRequest {
    pub(super) events: Vec<EntropyEvent>,
}

pub(crate) async fn post_v1_vault_collect_entropy(
    State(state): State<AppState>,
    Json(request): Json<VaultEntropyBatchRequest>,
) -> Result<Json<crate::vault::VaultStatus>, ApiError> {
    Ok(Json(state.vault.collect_entropy(request.events)?))
}

pub(crate) async fn post_v1_vault_create(
    State(state): State<AppState>,
) -> Result<Json<crate::vault::VaultStatus>, ApiError> {
    Ok(Json(state.vault.create()?))
}

pub(crate) async fn post_v1_vault_unlock(
    State(state): State<AppState>,
) -> Result<Json<crate::vault::VaultStatus>, ApiError> {
    Ok(Json(state.vault.unlock()?))
}

pub(crate) async fn post_v1_vault_recovery_export(
    State(state): State<AppState>,
) -> Result<Json<crate::vault::RecoveryExportResponse>, ApiError> {
    Ok(Json(state.vault.export_recovery()?))
}

#[derive(Deserialize)]
pub(crate) struct VaultRecoveryImportRequest {
    pub(super) recovery_phrase: String,
}

pub(crate) async fn post_v1_vault_recovery_import(
    State(state): State<AppState>,
    Json(request): Json<VaultRecoveryImportRequest>,
) -> Result<Json<crate::vault::VaultStatus>, ApiError> {
    Ok(Json(state.vault.import_recovery(&request.recovery_phrase)?))
}
