use super::*;
use crate::app::vault_reconciliation::lifecycle::spawn_host_vault_manifest_reconciliation;
use crate::domains::communications::templates::{
    CommunicationMergePreviewRow, CommunicationTemplateStore, NewCommunicationTemplate,
};

const MAX_MAIL_MERGE_PREVIEW_ROWS: usize = 250;

#[derive(Deserialize)]
pub(crate) struct RenderTemplateRequest {
    pub(super) template_id: String,
    pub(super) variables: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
pub(crate) struct MailMergePreviewRequest {
    pub(super) template_id: String,
    pub(super) rows: Vec<MailMergePreviewRowRequest>,
}

#[derive(Deserialize)]
pub(crate) struct MailMergePreviewRowRequest {
    pub(super) row_id: String,
    pub(super) variables: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
pub(crate) struct UpsertTemplateRequest {
    pub(super) template_id: Option<String>,
    pub(super) name: String,
    pub(super) subject_template: Option<String>,
    pub(super) body_template: Option<String>,
    pub(super) content: Option<String>,
    pub(super) variables: Option<Vec<String>>,
    pub(super) language: Option<String>,
}

pub(crate) async fn get_v1_rich_templates(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let templates = crate::app::api_support::stores::domain_stores::app_store::<
        CommunicationTemplateStore,
    >(pool.clone())
    .list()
    .await?;
    Ok(Json(serde_json::json!({ "templates": templates })))
}

pub(crate) async fn post_v1_rich_template(
    State(state): State<AppState>,
    Json(req): Json<UpsertTemplateRequest>,
) -> Result<Json<Value>, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let template = crate::app::api_support::stores::domain_stores::app_store::<
        CommunicationTemplateStore,
    >(pool.clone())
    .upsert(&NewCommunicationTemplate {
        template_id: req
            .template_id
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| {
                let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or_default();
                format!("mail_template:{timestamp}")
            }),
        name: req.name,
        subject_template: req
            .subject_template
            .or_else(|| req.content.clone())
            .unwrap_or_else(|| "Untitled template".to_owned()),
        body_template: req.body_template.or(req.content).unwrap_or_default(),
        variables: req.variables.unwrap_or_default(),
        language: req.language,
    })
    .await?;
    Ok(Json(
        serde_json::json!({ "saved": true, "template": template }),
    ))
}

pub(crate) async fn delete_v1_rich_template(
    State(state): State<AppState>,
    Path(template_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let template_id = template_id.trim();
    if template_id.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "template_id is required",
        ));
    }
    let deleted = crate::app::api_support::stores::domain_stores::app_store::<
        CommunicationTemplateStore,
    >(pool.clone())
    .delete(template_id)
    .await?;
    if !deleted {
        return Err(ApiError::NotFound);
    }
    Ok(Json(serde_json::json!({
        "template_id": template_id,
        "deleted": true
    })))
}

pub(crate) async fn get_v1_blockers()
-> Result<Json<Vec<crate::domains::communications::blockers::ArchitectureBlocker>>, ApiError> {
    Ok(Json(
        crate::domains::communications::blockers::list_blockers(),
    ))
}

pub(crate) async fn post_v1_render_template(
    State(state): State<AppState>,
    Json(req): Json<RenderTemplateRequest>,
) -> Result<Json<Value>, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let store = crate::app::api_support::stores::domain_stores::app_store::<
        CommunicationTemplateStore,
    >(pool.clone());
    let template_id = req.template_id.trim();
    let Some(template) = store.get(template_id).await? else {
        return Err(ApiError::NotFound);
    };
    let vars = req.variables.unwrap_or_default();
    let rendered = store.render(&template, &vars)?;
    Ok(Json(serde_json::json!({
        "template_id": template.template_id,
        "variables": vars,
        "rendered": rendered
    })))
}

pub(crate) async fn post_v1_rich_template_mail_merge_preview(
    State(state): State<AppState>,
    Json(req): Json<MailMergePreviewRequest>,
) -> Result<Json<Value>, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let template_id = req.template_id.trim();
    if template_id.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "template_id is required",
        ));
    }
    if req.rows.is_empty() {
        return Err(ApiError::InvalidCommunicationQuery(
            "mail merge preview rows are required",
        ));
    }
    if req.rows.len() > MAX_MAIL_MERGE_PREVIEW_ROWS {
        return Err(ApiError::InvalidCommunicationQuery(
            "mail merge preview row limit exceeded",
        ));
    }
    let rows = req
        .rows
        .into_iter()
        .map(|row| {
            let row_id = row.row_id.trim().to_owned();
            if row_id.is_empty() {
                return Err(ApiError::InvalidCommunicationQuery("row_id is required"));
            }
            Ok(CommunicationMergePreviewRow {
                row_id,
                variables: row.variables.unwrap_or_default(),
            })
        })
        .collect::<Result<Vec<_>, ApiError>>()?;
    let store = crate::app::api_support::stores::domain_stores::app_store::<
        CommunicationTemplateStore,
    >(pool.clone());
    let Some(template) = store.get(template_id).await? else {
        return Err(ApiError::NotFound);
    };
    let preview = store.render_mail_merge_preview(&template, rows)?;
    Ok(Json(serde_json::to_value(preview).map_err(|_| {
        ApiError::InvalidCommunicationQuery("mail merge preview response failed")
    })?))
}

#[derive(Deserialize)]
pub(crate) struct PersonaListQuery {
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
            personas: true,
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
    let status = state.vault.create()?;
    spawn_host_vault_manifest_reconciliation(&state);
    Ok(Json(status))
}

pub(crate) async fn post_v1_vault_unlock(
    State(state): State<AppState>,
) -> Result<Json<crate::vault::VaultStatus>, ApiError> {
    let status = state.vault.unlock()?;
    spawn_host_vault_manifest_reconciliation(&state);
    Ok(Json(status))
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
    let status = state.vault.import_recovery(&request.recovery_phrase)?;
    spawn_host_vault_manifest_reconciliation(&state);
    Ok(Json(status))
}
