use crate::app::handlers::{ApiError, AppState};
use crate::domains::documents::processing::{DocumentProcessingRecord, DocumentProcessingStore};
use axum::Json;
use axum::extract::{Path, State};

pub(crate) async fn get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DocumentProcessingRecord>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    Ok(Json(
        DocumentProcessingStore::new(pool)
            .document_processing(&id)
            .await?,
    ))
}
