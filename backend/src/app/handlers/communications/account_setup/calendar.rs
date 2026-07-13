use super::super::*;

pub(super) async fn upsert_google_workspace_calendar_account(
    state: &AppState,
    mail_account_id: &str,
    display_name: &str,
    external_account_id: &str,
    secret_ref: &str,
) -> Result<(), ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::app::api_support::stores::domain_stores::app_store::<
        crate::domains::calendar::events::CalendarAccountStore,
    >(pool)
    .upsert_google_workspace_account(
        mail_account_id,
        display_name,
        Some(external_account_id),
        secret_ref,
    )
    .await?;
    Ok(())
}

pub(super) async fn upsert_apple_icloud_calendar_account(
    state: &AppState,
    mail_account_id: &str,
    display_name: &str,
    external_account_id: &str,
    secret_ref: &str,
) -> Result<(), ApiError> {
    let pool = state
        .database
        .pool()
        .ok_or(ApiError::DatabaseNotConfigured)?
        .clone();
    crate::app::api_support::stores::domain_stores::app_store::<
        crate::domains::calendar::events::CalendarAccountStore,
    >(pool)
    .upsert_apple_icloud_account(
        mail_account_id,
        display_name,
        Some(external_account_id),
        secret_ref,
    )
    .await?;
    Ok(())
}
