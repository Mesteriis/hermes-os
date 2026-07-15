use crate::app::handlers::tasks::{
    candidates::*, core_records::*, health::*, intelligence::*, items::*, providers::*, rules::*,
};
use crate::app::state::AppState;
use axum::Router;
use axum::routing::{delete, get, post, put};

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/tasks", get(get_tasks).post(post_task))
        .route("/api/v1/tasks/{task_id}", get(get_task).put(put_task))
        .route("/api/v1/tasks/{task_id}/archive", post(post_task_archive))
        .route("/api/v1/tasks/{task_id}/status", post(post_task_status))
        .route(
            "/api/v1/tasks/{task_id}/context-pack",
            get(get_task_context_pack).post(post_task_context_pack),
        )
        .route(
            "/api/v1/tasks/{task_id}/evidence",
            get(get_task_evidence).post(post_task_evidence),
        )
        .route(
            "/api/v1/tasks/{task_id}/relations",
            get(get_task_relations).post(post_task_relation),
        )
        .route(
            "/api/v1/tasks/{task_id}/checklist",
            get(get_task_checklist).post(post_task_checklist),
        )
        .route(
            "/api/v1/tasks/{task_id}/subtasks",
            get(get_task_subtasks).post(post_task_subtask),
        )
        .route("/api/v1/tasks/{task_id}/analyze", post(post_task_analyze))
        .route("/api/v1/tasks/{task_id}/export", get(get_task_export))
        .route("/api/v1/tasks/{task_id}/external", get(get_task_external))
        .route(
            "/api/v1/tasks/providers",
            get(get_task_providers).post(post_task_provider),
        )
        .route("/api/v1/tasks/brain", post(post_task_brain))
        .route("/api/v1/tasks/search", get(get_task_search))
        .route("/api/v1/tasks/daily-brief", get(get_task_daily_brief))
        .route(
            "/api/v1/tasks/rules",
            get(get_task_rules).post(post_task_rule),
        )
        .route("/api/v1/tasks/rules/{rule_id}", delete(delete_task_rule))
        .route("/api/v1/tasks/templates", get(get_task_templates))
        .route("/api/v1/tasks/watchtower", get(get_task_watchtower))
        .route("/api/v1/tasks/health", get(get_task_health))
        .route("/api/v1/tasks/analytics", get(get_task_analytics))
        .route("/api/v1/task-candidates", get(get_task_candidates))
        .route(
            "/api/v1/task-candidates/{task_candidate_id}/review",
            put(put_task_candidate_review),
        )
}
