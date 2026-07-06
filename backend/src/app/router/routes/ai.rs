use super::support::*;

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/api/v1/ai/status", get(get_ai_status))
        .route(
            "/api/v1/ai/settings/overview",
            get(get_ai_settings_overview),
        )
        .route(
            "/api/v1/ai/providers",
            get(get_ai_providers).post(post_ai_provider),
        )
        .route(
            "/api/v1/ai/providers/{provider_id}",
            patch(patch_ai_provider),
        )
        .route(
            "/api/v1/ai/providers/{provider_id}/test",
            post(post_ai_provider_test),
        )
        .route(
            "/api/v1/ai/providers/{provider_id}/sync-models",
            post(post_ai_provider_sync_models),
        )
        .route(
            "/api/v1/ai/providers/{provider_id}/consent",
            post(post_ai_provider_consent),
        )
        .route(
            "/api/v1/ai/provider-auth/start",
            post(post_ai_provider_auth_start),
        )
        .route(
            "/api/v1/ai/provider-auth/{setup_id}",
            get(get_ai_provider_auth_status),
        )
        .route("/api/v1/ai/models", get(get_ai_models))
        .route(
            "/api/v1/ai/models/availability",
            patch(patch_ai_model_availability),
        )
        .route("/api/v1/ai/model-routes/{slot}", put(put_ai_model_route))
        .route(
            "/api/v1/ai/prompts",
            get(get_ai_prompts).post(post_ai_prompt),
        )
        .route(
            "/api/v1/ai/prompts/{prompt_id}/versions",
            post(post_ai_prompt_version),
        )
        .route(
            "/api/v1/ai/prompts/{prompt_id}/activate",
            post(post_ai_prompt_activate),
        )
        .route(
            "/api/v1/ai/prompts/{prompt_id}/test",
            post(post_ai_prompt_test),
        )
        .route("/api/v1/ai/agents", get(get_ai_agents))
        .route("/api/v1/ai/runs", get(get_ai_runs))
        .route("/api/v1/ai/runs/{run_id}", get(get_ai_run))
        .route("/api/v1/ai/answers", post(post_ai_answer))
        .route(
            "/api/v1/ai/task-candidates/refresh",
            post(post_ai_task_candidates_refresh),
        )
        .route("/api/v1/ai/meeting-prep", post(post_ai_meeting_prep))
}
