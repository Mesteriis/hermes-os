use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::app::error::types::ApiError;
use crate::app::state::AppState;
use crate::domains::personas::enrichment_engine::EnrichmentResultStore;
use crate::domains::personas::expertise::PersonaExpertiseStore;
use crate::domains::personas::trust::promises::PersonaPromiseStore;
use crate::domains::personas::trust::risks::PersonaRiskStore;
// ── Persona Enrichment ─────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct EnrichmentResultsResponse {
    items: Vec<crate::domains::personas::enrichment_engine::EnrichmentResult>,
}

pub(crate) async fn get_persona_enrichment(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<EnrichmentResultsResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items =
        crate::app::api_support::stores::domain_stores::app_store::<EnrichmentResultStore>(pool)
            .list(&persona_id)
            .await
            .map_err(ApiError::from)?;
    Ok(Json(EnrichmentResultsResponse { items }))
}

pub(crate) async fn post_persona_enrichment_apply(
    State(state): State<AppState>,
    Path((persona_id, result_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::domains::personas::command_service::PersonaCommandService::new(pool)
        .apply_enrichment_manual(&persona_id, &result_id)
        .await?;
    Ok(Json(json!({"applied": true})))
}

pub(crate) async fn post_persona_enrichment_reject(
    State(state): State<AppState>,
    Path((persona_id, result_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::domains::personas::command_service::PersonaCommandService::new(pool)
        .reject_enrichment_manual(&persona_id, &result_id)
        .await?;
    Ok(Json(json!({"rejected": true})))
}

// ── Persona Expertise ──────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonaExpertiseResponse {
    items: Vec<crate::domains::personas::expertise::PersonaExpertise>,
}

pub(crate) async fn get_persona_expertise(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<PersonaExpertiseResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items =
        crate::app::api_support::stores::domain_stores::app_store::<PersonaExpertiseStore>(pool)
            .list(&persona_id)
            .await
            .map_err(ApiError::from)?;
    Ok(Json(PersonaExpertiseResponse { items }))
}

#[derive(Deserialize)]
pub(crate) struct ExpertiseSearchQuery {
    skill: String,
    limit: Option<i64>,
}

pub(crate) async fn get_persona_expertise_search(
    State(state): State<AppState>,
    Query(query): Query<ExpertiseSearchQuery>,
) -> Result<Json<PersonaExpertiseResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items =
        crate::app::api_support::stores::domain_stores::app_store::<PersonaExpertiseStore>(pool)
            .search_by_skill(&query.skill, query.limit.unwrap_or(20))
            .await
            .map_err(ApiError::from)?;
    Ok(Json(PersonaExpertiseResponse { items }))
}

// ── Persona Promises ───────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonaPromisesResponse {
    items: Vec<crate::domains::personas::trust::models::PersonaPromise>,
}

pub(crate) async fn get_persona_promises(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<PersonaPromisesResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items =
        crate::app::api_support::stores::domain_stores::app_store::<PersonaPromiseStore>(pool)
            .list(&persona_id)
            .await
            .map_err(ApiError::from)?;
    Ok(Json(PersonaPromisesResponse { items }))
}

// ── Persona Risks ───────────────────────────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct PersonaRisksResponse {
    items: Vec<crate::domains::personas::trust::models::PersonaRisk>,
}

pub(crate) async fn get_persona_risks(
    State(state): State<AppState>,
    Path(persona_id): Path<String>,
) -> Result<Json<PersonaRisksResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::stores::domain_stores::app_store::<PersonaRiskStore>(pool)
        .list(&persona_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(PersonaRisksResponse { items }))
}
