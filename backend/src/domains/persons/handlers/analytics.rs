use crate::app::handlers::{ApiError, AppState};
use crate::domains::persons::analytics::PersonAnalyticsService;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;

pub async fn analytics(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        serde_json::to_value(PersonAnalyticsService::new(pool).compute(&id).await?)
            .unwrap_or_default(),
    ))
}
