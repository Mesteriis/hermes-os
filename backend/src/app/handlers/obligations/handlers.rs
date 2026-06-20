use axum::Json;
use axum::extract::{Path, Query, State};
use serde_json::json;

use super::models::{ObligationListQuery, ObligationListResponse, ObligationReviewApiRequest};
use crate::app::{ApiError, AppState};
use crate::domains::obligations::{
    Obligation, ObligationCommandService, ObligationEntityKind, ObligationReviewState,
    ObligationStore,
};
use crate::platform::audit::{ApiAuditLog, NewApiAuditRecord};

const OBLIGATION_API_ACTOR_ID: &str = "hermes-frontend";
const DEFAULT_OBLIGATION_LIMIT: i64 = 50;
const MIN_OBLIGATION_LIMIT: i64 = 1;
const MAX_OBLIGATION_LIMIT: i64 = 100;

pub(crate) async fn get_v1_obligations(
    State(state): State<AppState>,
    Query(query): Query<ObligationListQuery>,
) -> Result<Json<ObligationListResponse>, ApiError> {
    let limit = validate_limit(query.limit)?;
    let store = obligation_store(&state)?;
    let items = match (
        query.review_state.as_deref(),
        query.entity_kind.as_deref(),
        query.entity_id.as_deref(),
    ) {
        (Some(review_state), None, None) => {
            let review_state = parse_review_state(review_state)?;
            store.list_by_review_state(review_state, limit).await?
        }
        (None, Some(entity_kind), Some(entity_id)) => {
            let entity_kind = parse_required_entity_kind(Some(entity_kind))?;
            let entity_id = validate_required_query_value(Some(entity_id))?;
            store
                .list_for_entity(entity_kind, &entity_id, limit)
                .await?
        }
        (Some(_), _, _) => {
            return Err(ApiError::InvalidObligationQuery(
                "review_state cannot be combined with entity filters",
            ));
        }
        (None, _, _) => {
            return Err(ApiError::InvalidObligationQuery(
                "missing required obligation query field",
            ));
        }
    };

    Ok(Json(ObligationListResponse { items }))
}

pub(crate) async fn put_v1_obligation_review(
    State(state): State<AppState>,
    Path(obligation_id): Path<String>,
    Json(request): Json<ObligationReviewApiRequest>,
) -> Result<Json<Obligation>, ApiError> {
    let obligation_id = validate_required_query_value(Some(&obligation_id))?;
    let review_state = parse_review_state(&request.review_state)?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::obligation_review_set(
            OBLIGATION_API_ACTOR_ID,
            &obligation_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let obligation = ObligationCommandService::new(pool)
        .review_manual(&obligation_id, review_state)
        .await?;

    Ok(Json(obligation))
}

fn obligation_store(state: &AppState) -> Result<ObligationStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ObligationStore::new(pool.clone()))
}

fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApiAuditLog::new(pool.clone()))
}

fn parse_required_entity_kind(value: Option<&str>) -> Result<ObligationEntityKind, ApiError> {
    let value = validate_required_query_value(value)?;
    ObligationEntityKind::parse(&value).map_err(ApiError::from)
}

fn parse_review_state(value: &str) -> Result<ObligationReviewState, ApiError> {
    ObligationReviewState::parse(value).map_err(ApiError::from)
}

fn validate_required_query_value(value: Option<&str>) -> Result<String, ApiError> {
    let value = value.unwrap_or_default().trim();
    if value.is_empty() {
        return Err(ApiError::InvalidObligationQuery(
            "missing required obligation query field",
        ));
    }

    Ok(value.to_owned())
}

fn validate_limit(limit: Option<i64>) -> Result<i64, ApiError> {
    let limit = limit.unwrap_or(DEFAULT_OBLIGATION_LIMIT);
    if !(MIN_OBLIGATION_LIMIT..=MAX_OBLIGATION_LIMIT).contains(&limit) {
        return Err(ApiError::InvalidObligationQuery(
            "limit must be between 1 and 100",
        ));
    }

    Ok(limit)
}
