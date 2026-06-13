use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};

use crate::app::{ApiError, AppState};
use crate::platform::audit::{ApiAuditLog, NewApiAuditRecord};

use super::{Relationship, RelationshipEntityKind, RelationshipReviewState, RelationshipStore};

const RELATIONSHIP_API_ACTOR_ID: &str = "hermes-frontend";
const DEFAULT_RELATIONSHIP_LIMIT: i64 = 50;
const MIN_RELATIONSHIP_LIMIT: i64 = 1;
const MAX_RELATIONSHIP_LIMIT: i64 = 100;

#[derive(Debug, Deserialize)]
pub(crate) struct RelationshipListQuery {
    entity_kind: Option<String>,
    entity_id: Option<String>,
    review_state: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RelationshipReviewApiRequest {
    review_state: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct RelationshipListResponse {
    items: Vec<Relationship>,
}

pub(crate) async fn get_v1_relationships(
    State(state): State<AppState>,
    Query(query): Query<RelationshipListQuery>,
) -> Result<Json<RelationshipListResponse>, ApiError> {
    let limit = validate_limit(query.limit)?;
    let store = relationship_store(&state)?;
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
            return Err(ApiError::InvalidRelationshipQuery(
                "review_state cannot be combined with entity filters",
            ));
        }
        (None, _, _) => {
            return Err(ApiError::InvalidRelationshipQuery(
                "missing required relationship query field",
            ));
        }
    };

    Ok(Json(RelationshipListResponse { items }))
}

pub(crate) async fn put_v1_relationship_review(
    State(state): State<AppState>,
    Path(relationship_id): Path<String>,
    Json(request): Json<RelationshipReviewApiRequest>,
) -> Result<Json<Relationship>, ApiError> {
    let relationship_id = validate_required_query_value(Some(&relationship_id))?;
    let review_state = parse_review_state(&request.review_state)?;

    api_audit_log(&state)?
        .record(&NewApiAuditRecord::relationship_review_set(
            RELATIONSHIP_API_ACTOR_ID,
            &relationship_id,
        ))
        .await?;

    let relationship = relationship_store(&state)?
        .set_review_state(&relationship_id, review_state)
        .await?;

    Ok(Json(relationship))
}

fn relationship_store(state: &AppState) -> Result<RelationshipStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(RelationshipStore::new(pool.clone()))
}

fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApiAuditLog::new(pool.clone()))
}

fn parse_required_entity_kind(value: Option<&str>) -> Result<RelationshipEntityKind, ApiError> {
    let value = validate_required_query_value(value)?;
    RelationshipEntityKind::parse(&value).map_err(ApiError::from)
}

fn parse_review_state(value: &str) -> Result<RelationshipReviewState, ApiError> {
    RelationshipReviewState::parse(value).map_err(ApiError::from)
}

fn validate_required_query_value(value: Option<&str>) -> Result<String, ApiError> {
    let value = value.unwrap_or_default().trim();
    if value.is_empty() {
        return Err(ApiError::InvalidRelationshipQuery(
            "missing required relationship query field",
        ));
    }

    Ok(value.to_owned())
}

fn validate_limit(limit: Option<i64>) -> Result<i64, ApiError> {
    let limit = limit.unwrap_or(DEFAULT_RELATIONSHIP_LIMIT);
    if !(MIN_RELATIONSHIP_LIMIT..=MAX_RELATIONSHIP_LIMIT).contains(&limit) {
        return Err(ApiError::InvalidRelationshipQuery(
            "limit must be between 1 and 100",
        ));
    }

    Ok(limit)
}
