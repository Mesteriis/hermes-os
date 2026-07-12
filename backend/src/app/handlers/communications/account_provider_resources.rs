use super::*;
use crate::domains::communications::provider_resources::{
    MailProviderResource, MailProviderResourceMappingUpdate, MailProviderResourceStore,
};

#[derive(Serialize)]
pub(crate) struct MailProviderResourceListResponse {
    pub(crate) items: Vec<MailProviderResource>,
}

pub(crate) async fn get_v1_email_account_provider_resources(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<MailProviderResourceListResponse>, ApiError> {
    let account = email_account_or_not_found(&state, &account_id).await?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let items = crate::app::api_support::app_store::<MailProviderResourceStore>(pool)
        .list_for_account(&account.account_id)
        .await?;
    Ok(Json(MailProviderResourceListResponse { items }))
}

pub(crate) async fn put_v1_email_account_provider_resource_mapping(
    State(state): State<AppState>,
    Path((account_id, mapping_id)): Path<(String, String)>,
    Json(update): Json<MailProviderResourceMappingUpdate>,
) -> Result<Json<MailProviderResource>, ApiError> {
    let account = email_account_or_not_found(&state, &account_id).await?;
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    let store = crate::app::api_support::app_store::<MailProviderResourceStore>(pool);
    let Some(current) = store.resource(&mapping_id).await? else {
        return Err(ApiError::NotFound);
    };
    if current.account_id != account.account_id {
        return Err(ApiError::NotFound);
    }
    let updated = store
        .set_manual_mapping(&mapping_id, &update)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(updated))
}
