use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/settings", get(get_application_settings))
        .route(
            "/api/v1/settings/accounts",
            get(get_application_settings_accounts),
        )
        .route(
            "/api/v1/settings/accounts/{account_id}",
            patch(patch_application_settings_account),
        )
        .route(
            "/api/v1/settings/{setting_key}",
            put(put_application_setting),
        )
}
