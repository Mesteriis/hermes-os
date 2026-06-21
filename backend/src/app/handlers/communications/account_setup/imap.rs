use super::super::*;
use super::calendar::upsert_apple_icloud_calendar_account;
use super::models::{EmailAccountSetupApiResponse, ImapAccountSetupApiRequest};

pub(crate) async fn post_imap_account_setup(
    State(state): State<AppState>,
    Json(request): Json<ImapAccountSetupApiRequest>,
) -> Result<Json<EmailAccountSetupApiResponse>, ApiError> {
    let setup_request = request.into_setup_request()?;
    let service = account_setup_service(&state)?;
    require_unlocked_host_vault(&state)?;
    let icloud_calendar_account =
        (setup_request.provider_kind == EmailProviderKind::Icloud).then(|| {
            (
                setup_request.account_id.clone(),
                setup_request.display_name.clone(),
                setup_request.external_account_id.clone(),
            )
        });
    let result = service.setup_imap_account(setup_request).await?;
    if let Some((mail_account_id, display_name, external_account_id)) = icloud_calendar_account {
        upsert_apple_icloud_calendar_account(
            &state,
            &mail_account_id,
            &display_name,
            &external_account_id,
            &result.secret_ref,
        )
        .await?;
    }

    Ok(Json(result.into()))
}
