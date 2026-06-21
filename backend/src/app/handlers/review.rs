use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::app::{ApiError, AppState};
use crate::application::review_promotion::ReviewPromotionService;
use crate::domains::review::{
    NewReviewItem, NewReviewItemEvidence, ReviewInboxService, ReviewInboxStore, ReviewItem,
    ReviewItemKind, ReviewItemStatus, ReviewPromotionTarget,
};
use crate::platform::observations::{NewObservation, ObservationOriginKind, ObservationStore};

const DEFAULT_REVIEW_LIMIT: i64 = 50;
const MIN_REVIEW_LIMIT: i64 = 1;
const MAX_REVIEW_LIMIT: i64 = 100;
const REVIEW_STATUS_ACTIVE: &str = "active";
const REVIEW_STATUS_ALL: &str = "all";

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewItemsQuery {
    status: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ReviewItemsResponse {
    items: Vec<ReviewItem>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateReviewItemRequest {
    item_kind: String,
    title: String,
    summary: String,
    confidence: f64,
    metadata: Option<Value>,
    evidence: Vec<CreateReviewItemEvidenceRequest>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateReviewItemEvidenceRequest {
    observation_id: String,
    evidence_role: Option<String>,
    metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PromoteReviewItemRequest {
    target_domain: String,
    target_entity_kind: String,
    target_entity_id: String,
}

pub(crate) async fn get_v1_review_items(
    State(state): State<AppState>,
    Query(query): Query<ReviewItemsQuery>,
) -> Result<Json<ReviewItemsResponse>, ApiError> {
    let status = parse_status_filter(query.status.as_deref())?;
    let limit = validate_limit(query.limit)?;
    let items = match status {
        ReviewItemsStatusFilter::Single(status) => {
            review_store(&state)?.list_by_status(status, limit).await?
        }
        ReviewItemsStatusFilter::Active => review_store(&state)?.list_open(limit).await?,
        ReviewItemsStatusFilter::All => review_store(&state)?.list_all(limit).await?,
    };
    Ok(Json(ReviewItemsResponse { items }))
}

pub(crate) async fn post_v1_review_items(
    State(state): State<AppState>,
    Json(request): Json<CreateReviewItemRequest>,
) -> Result<Json<ReviewItem>, ApiError> {
    let mut item = NewReviewItem::new(
        parse_item_kind(&request.item_kind)?,
        request.title,
        request.summary,
        request.confidence,
    );
    if let Some(metadata) = request.metadata {
        item = item.metadata(metadata);
    }

    let evidence: Vec<NewReviewItemEvidence> = request
        .evidence
        .into_iter()
        .map(|item| {
            let mut evidence = NewReviewItemEvidence::new(item.observation_id);
            if let Some(role) = item.evidence_role {
                evidence = evidence.role(role);
            }
            if let Some(metadata) = item.metadata {
                evidence = evidence.metadata(metadata);
            }
            evidence
        })
        .collect();

    let item = review_store(&state)?
        .create_with_evidence(&item, &evidence)
        .await?;
    Ok(Json(item))
}

pub(crate) async fn post_v1_review_item_approve(
    State(state): State<AppState>,
    Path(review_item_id): Path<String>,
) -> Result<Json<ReviewItem>, ApiError> {
    let item =
        transition_review_item_status(&state, &review_item_id, ReviewItemStatus::Approved).await?;
    Ok(Json(item))
}

pub(crate) async fn post_v1_review_item_dismiss(
    State(state): State<AppState>,
    Path(review_item_id): Path<String>,
) -> Result<Json<ReviewItem>, ApiError> {
    let item =
        transition_review_item_status(&state, &review_item_id, ReviewItemStatus::Dismissed).await?;
    Ok(Json(item))
}

pub(crate) async fn post_v1_review_item_archive(
    State(state): State<AppState>,
    Path(review_item_id): Path<String>,
) -> Result<Json<ReviewItem>, ApiError> {
    let item =
        transition_review_item_status(&state, &review_item_id, ReviewItemStatus::Archived).await?;
    Ok(Json(item))
}

pub(crate) async fn post_v1_review_item_take(
    State(state): State<AppState>,
    Path(review_item_id): Path<String>,
) -> Result<Json<ReviewItem>, ApiError> {
    let item =
        transition_review_item_status(&state, &review_item_id, ReviewItemStatus::InReview).await?;
    Ok(Json(item))
}

pub(crate) async fn post_v1_review_item_promote(
    State(state): State<AppState>,
    Path(review_item_id): Path<String>,
    Json(request): Json<PromoteReviewItemRequest>,
) -> Result<Json<ReviewItem>, ApiError> {
    let target = ReviewPromotionTarget::new(
        request.target_domain,
        request.target_entity_kind,
        request.target_entity_id,
    );
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    let observation = crate::app::api_support::app_store::<ObservationStore>(pool.clone())
        .capture(
            &NewObservation::new(
                "REVIEW_TRANSITION",
                ObservationOriginKind::Manual,
                Utc::now(),
                json!({
                    "review_item_id": review_item_id,
                    "operation": "review_item_promote",
                    "target_domain": target.target_domain,
                    "target_entity_kind": target.target_entity_kind,
                    "target_entity_id": target.target_entity_id,
                }),
                format!("review-item://{review_item_id}/promote"),
            )
            .provenance(json!({
                "captured_by": "review_api.post_v1_review_item_promote",
                "endpoint": "post_v1_review_item_promote",
            })),
        )
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "review item promote observation capture failed");
            ApiError::InvalidReviewQuery("review item promote observation capture failed")
        })?;

    let item = ReviewPromotionService::new(pool.clone())
        .promote_with_observation(
            &review_item_id,
            target,
            Some(&observation.observation_id),
            Some(json!({
                "captured_by": "review_api.post_v1_review_item_promote",
                "endpoint": "post_v1_review_item_promote",
            })),
        )
        .await?;
    Ok(Json(item))
}

async fn transition_review_item_status(
    state: &AppState,
    review_item_id: &str,
    status: ReviewItemStatus,
) -> Result<ReviewItem, ApiError> {
    let item = review_service(state)?
        .transition_status_from_manual(
            review_item_id,
            status,
            "review_api.transition_review_item_status",
            "transition_review_item_status",
        )
        .await?;
    Ok(item)
}

fn review_service(state: &AppState) -> Result<ReviewInboxService, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ReviewInboxService::new(pool.clone()))
}

fn review_store(state: &AppState) -> Result<ReviewInboxStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(crate::app::api_support::app_store::<ReviewInboxStore>(
        pool.clone(),
    ))
}

fn parse_item_kind(value: &str) -> Result<ReviewItemKind, ApiError> {
    ReviewItemKind::parse(value).map_err(ApiError::from)
}

enum ReviewItemsStatusFilter {
    Single(ReviewItemStatus),
    Active,
    All,
}

fn parse_status_filter(value: Option<&str>) -> Result<ReviewItemsStatusFilter, ApiError> {
    match value {
        None => Ok(ReviewItemsStatusFilter::Active),
        Some(value) => match value {
            REVIEW_STATUS_ACTIVE => Ok(ReviewItemsStatusFilter::Active),
            REVIEW_STATUS_ALL => Ok(ReviewItemsStatusFilter::All),
            unknown => ReviewItemStatus::parse(unknown)
                .map(ReviewItemsStatusFilter::Single)
                .map_err(ApiError::from),
        },
    }
}

fn validate_limit(limit: Option<i64>) -> Result<i64, ApiError> {
    let limit = limit.unwrap_or(DEFAULT_REVIEW_LIMIT);
    if !(MIN_REVIEW_LIMIT..=MAX_REVIEW_LIMIT).contains(&limit) {
        return Err(ApiError::InvalidReviewQuery(
            "limit must be between 1 and 100",
        ));
    }

    Ok(limit)
}
