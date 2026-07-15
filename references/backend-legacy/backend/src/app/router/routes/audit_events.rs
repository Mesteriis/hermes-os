use crate::app::handlers::events::handlers::*;
use crate::app::state::AppState;
use axum::Router;
use axum::routing::get;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/audit/events", get(get_audit_events))
        .route("/api/realtime/v2/events", get(get_events_stream))
        .route("/api/v1/events", get(get_events).post(post_event))
        .route("/api/v1/events/{event_id}", get(get_event))
        .route(
            "/api/v1/events/{event_id}/children",
            get(get_event_children),
        )
        .route("/api/v1/events/{event_id}/trace", get(get_event_trace))
        .route(
            "/api/v1/event-traces/{correlation_id}",
            get(get_event_trace_by_correlation),
        )
}
