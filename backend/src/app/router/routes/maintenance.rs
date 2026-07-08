use super::support::*;

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
