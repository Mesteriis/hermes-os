use crate::app::handlers::maintenance::*;
use crate::app::state::AppState;
use axum::Router;
use axum::routing::{get, post};

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/v1/maintenance/overview",
            get(get_maintenance_overview),
        )
        .route(
            "/api/v1/maintenance/actions/{action_id}",
            post(post_maintenance_action),
        )
}
