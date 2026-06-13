use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/audit/events", get(get_audit_events))
        .route("/api/v1/events", post(post_event))
        .route("/api/v1/events/{event_id}", get(get_event))
}
