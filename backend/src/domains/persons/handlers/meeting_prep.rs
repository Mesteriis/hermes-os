use crate::app::handlers::{ApiError, AppState};
use crate::domains::persons::investigator::PersonInvestigator;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;

pub(crate) async fn meeting_prep(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pool = s
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        serde_json::to_value(PersonInvestigator::new(pool).meeting_prep(&id).await?)
            .unwrap_or_default(),
    ))
}
