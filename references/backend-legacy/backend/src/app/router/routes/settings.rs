use crate::app::handlers::settings::*;
use crate::app::state::AppState;
use axum::Router;
use axum::routing::{get, patch, put};

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
