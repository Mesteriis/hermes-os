use super::super::*;
use crate::domains::mail::folders::{
    FolderMessageActionResponse, FolderMessageListQuery, FolderMessagePage, MailFolder,
    MailFolderListPage, MailFolderListQuery, MailFolderStore, NewMailFolder, UpdateMailFolder,
};
use crate::domains::mail::service::MailCommandService;

#[derive(Deserialize)]
pub(crate) struct FoldersQuery {
    pub(crate) account_id: Option<String>,
    pub(crate) cursor: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct FolderMessagesQuery {
    pub(crate) cursor: Option<String>,
    pub(crate) limit: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct FolderDeleteResponse {
    pub(crate) deleted: bool,
}

pub(crate) async fn get_v1_mail_folders(
    State(state): State<AppState>,
    Query(query): Query<FoldersQuery>,
) -> Result<Json<MailFolderListPage>, ApiError> {
    let page = folder_store(&state)?
        .list(MailFolderListQuery {
            account_id: query.account_id.as_deref(),
            cursor: query.cursor.as_deref(),
            limit: query.limit.unwrap_or(500),
        })
        .await?;
    Ok(Json(page))
}

pub(crate) async fn post_v1_mail_folder(
    State(state): State<AppState>,
    Json(request): Json<NewMailFolder>,
) -> Result<Json<MailFolder>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let folder = MailCommandService::new(pool).create_folder(request).await?;
    Ok(Json(folder))
}

pub(crate) async fn put_v1_mail_folder(
    State(state): State<AppState>,
    Path(folder_id): Path<String>,
    Json(request): Json<UpdateMailFolder>,
) -> Result<Json<MailFolder>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let Some(folder) = MailCommandService::new(pool)
        .update_folder(&folder_id, request)
        .await?
    else {
        return Err(ApiError::NotFound);
    };
    Ok(Json(folder))
}

pub(crate) async fn delete_v1_mail_folder(
    State(state): State<AppState>,
    Path(folder_id): Path<String>,
) -> Result<Json<FolderDeleteResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let deleted = MailCommandService::new(pool)
        .delete_folder(&folder_id)
        .await?;
    Ok(Json(FolderDeleteResponse { deleted }))
}

pub(crate) async fn get_v1_mail_folder_messages(
    State(state): State<AppState>,
    Path(folder_id): Path<String>,
    Query(query): Query<FolderMessagesQuery>,
) -> Result<Json<FolderMessagePage>, ApiError> {
    let page = folder_store(&state)?
        .list_messages(FolderMessageListQuery {
            folder_id: &folder_id,
            cursor: query.cursor.as_deref(),
            limit: query.limit.unwrap_or(250),
        })
        .await?;
    Ok(Json(page))
}

pub(crate) async fn post_v1_copy_message_to_folder(
    State(state): State<AppState>,
    Path((folder_id, message_id)): Path<(String, String)>,
) -> Result<Json<FolderMessageActionResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let Some(response) = MailCommandService::new(pool)
        .copy_message_to_folder(&folder_id, &message_id)
        .await?
    else {
        return Err(ApiError::NotFound);
    };
    Ok(Json(response))
}

pub(crate) async fn post_v1_move_message_to_folder(
    State(state): State<AppState>,
    Path((folder_id, message_id)): Path<(String, String)>,
) -> Result<Json<FolderMessageActionResponse>, ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let Some(response) = MailCommandService::new(pool)
        .move_message_to_folder(&folder_id, &message_id)
        .await?
    else {
        return Err(ApiError::NotFound);
    };
    Ok(Json(response))
}

fn folder_store(state: &AppState) -> Result<MailFolderStore, ApiError> {
    let Some(pool) = state.database.pool().cloned() else {
        return Err(ApiError::DatabaseNotConfigured);
    };
    Ok(MailFolderStore::new(pool))
}
