use axum::Json;
use axum::extract::{Path, Query, State};
use serde_json::json;

use super::models::{DecisionListQuery, DecisionListResponse, DecisionReviewApiRequest};
use crate::app::{ApiError, AppState};
use crate::application::review_transitions::DecisionReviewApplicationService;
use crate::domains::decisions::{Decision, DecisionEntityKind, DecisionReviewState, DecisionStore};
use crate::platform::audit::{ApiAuditLog, NewApiAuditRecord};

const DECISION_API_ACTOR_ID: &str = "hermes-frontend";
const DEFAULT_DECISION_LIMIT: i64 = 50;
const MIN_DECISION_LIMIT: i64 = 1;
const MAX_DECISION_LIMIT: i64 = 100;

pub(crate) async fn get_v1_decisions(
    State(state): State<AppState>,
    Query(query): Query<DecisionListQuery>,
) -> Result<Json<DecisionListResponse>, ApiError> {
    let limit = validate_limit(query.limit)?;
    let store = decision_store(&state)?;
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
            return Err(ApiError::InvalidDecisionQuery(
                "review_state cannot be combined with entity filters",
            ));
        }
        (None, _, _) => {
            return Err(ApiError::InvalidDecisionQuery(
                "missing required decision query field",
            ));
        }
    };

    Ok(Json(DecisionListResponse { items }))
}

pub(crate) async fn put_v1_decision_review(
    State(state): State<AppState>,
    Path(decision_id): Path<String>,
    Json(request): Json<DecisionReviewApiRequest>,
) -> Result<Json<Decision>, ApiError> {
    let decision_id = validate_required_query_value(Some(&decision_id))?;
    let review_state = parse_review_state(&request.review_state)?;
    api_audit_log(&state)?
        .record(&NewApiAuditRecord::decision_review_set(
            DECISION_API_ACTOR_ID,
            &decision_id,
        ))
        .await?;

    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let decision = DecisionReviewApplicationService::new(pool)
        .review_manual(&decision_id, review_state)
        .await?;

    Ok(Json(decision))
}

fn decision_store(state: &AppState) -> Result<DecisionStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(crate::app::api_support::stores::domain_stores::app_store::<
        DecisionStore,
    >(pool.clone()))
}

fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApiAuditLog::new(pool.clone()))
}

fn parse_required_entity_kind(value: Option<&str>) -> Result<DecisionEntityKind, ApiError> {
    let value = validate_required_query_value(value)?;
    DecisionEntityKind::parse(&value).map_err(ApiError::from)
}

fn parse_review_state(value: &str) -> Result<DecisionReviewState, ApiError> {
    DecisionReviewState::parse(value).map_err(ApiError::from)
}

fn validate_required_query_value(value: Option<&str>) -> Result<String, ApiError> {
    let value = value.unwrap_or_default().trim();
    if value.is_empty() {
        return Err(ApiError::InvalidDecisionQuery(
            "missing required decision query field",
        ));
    }

    Ok(value.to_owned())
}

fn validate_limit(limit: Option<i64>) -> Result<i64, ApiError> {
    let limit = limit.unwrap_or(DEFAULT_DECISION_LIMIT);
    if !(MIN_DECISION_LIMIT..=MAX_DECISION_LIMIT).contains(&limit) {
        return Err(ApiError::InvalidDecisionQuery(
            "limit must be between 1 and 100",
        ));
    }

    Ok(limit)
}
