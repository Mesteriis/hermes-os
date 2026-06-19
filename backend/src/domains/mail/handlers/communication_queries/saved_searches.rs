use super::super::*;
use crate::domains::mail::saved_searches::{
    MailSavedSearch, MailSavedSearchListQuery, MailSavedSearchStore, NewMailSavedSearch,
    UpdateMailSavedSearch,
};
use crate::domains::mail::service::MailCommandService;

#[derive(Deserialize)]
pub(crate) struct SavedSearchesQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) smart_folder: Option<bool>,
    pub(crate) cursor: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct SavedSearchListResponse {
    pub(crate) items: Vec<MailSavedSearch>,
    pub(crate) next_cursor: Option<String>,
    pub(crate) has_more: bool,
}

#[derive(Serialize)]
pub(crate) struct SavedSearchDeleteResponse {
    pub(crate) deleted: bool,
}

pub(crate) async fn get_v1_saved_searches(
    State(state): State<AppState>,
    Query(query): Query<SavedSearchesQuery>,
) -> Result<Json<SavedSearchListResponse>, ApiError> {
    let page = saved_search_store(&state)?
        .list(MailSavedSearchListQuery {
            account_id: query.account_id.as_deref(),
            is_smart_folder: query.smart_folder,
            cursor: query.cursor.as_deref(),
            limit: query.limit.unwrap_or(500),
        })
        .await?;
    Ok(Json(SavedSearchListResponse {
        items: page.items,
        next_cursor: page.next_cursor,
        has_more: page.has_more,
    }))
}

pub(crate) async fn post_v1_saved_search(
    State(state): State<AppState>,
    Json(request): Json<NewMailSavedSearch>,
) -> Result<Json<MailSavedSearch>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let saved_search = MailCommandService::new(pool)
        .create_saved_search(request)
        .await?;
    Ok(Json(saved_search))
}

pub(crate) async fn put_v1_saved_search(
    State(state): State<AppState>,
    Path(saved_search_id): Path<String>,
    Json(request): Json<UpdateMailSavedSearch>,
) -> Result<Json<MailSavedSearch>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let Some(saved_search) = MailCommandService::new(pool)
        .update_saved_search(&saved_search_id, request)
        .await?
    else {
        return Err(ApiError::NotFound);
    };
    Ok(Json(saved_search))
}

pub(crate) async fn delete_v1_saved_search(
    State(state): State<AppState>,
    Path(saved_search_id): Path<String>,
) -> Result<Json<SavedSearchDeleteResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let deleted = MailCommandService::new(pool)
        .delete_saved_search(&saved_search_id)
        .await?;
    Ok(Json(SavedSearchDeleteResponse { deleted }))
}

fn saved_search_store(state: &AppState) -> Result<MailSavedSearchStore, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    Ok(MailSavedSearchStore::new(pool))
}
