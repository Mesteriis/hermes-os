mod communications;
mod communications_ai_runtime;
mod communications_analytics_proto;
mod communications_archive_proto;
mod communications_attachment_policy;
mod communications_attachment_preview;
mod communications_attachment_proto;
mod communications_attachment_search_proto;
mod communications_auth_proto;
mod communications_blocker_proto;
mod communications_draft_proto;
mod communications_errors;
mod communications_extraction_proto;
mod communications_folder_proto;
mod communications_json_policy;
mod communications_message_proto;
mod communications_outbox_proto;
mod communications_persona_proto;
mod communications_request_policy;
mod communications_result_proto;
mod communications_saved_folder_proto;
mod communications_summary_proto;
mod communications_template_proto;
mod communications_thread_proto;
mod communications_timestamp_policy;
mod communications_workflow_request_proto;
mod communications_workflow_response_proto;
mod signal_hub;

use axum::Router;
use axum::middleware;
use connectrpc::Router as ConnectRouter;
use sqlx::postgres::PgPool;

use crate::app::guard;
use crate::app::state::AppState;
use crate::platform::config::app_config::AppConfig;
use crate::vault::HostVault;

pub(crate) fn protected_routes(
    pool: Option<PgPool>,
    config: AppConfig,
    vault: HostVault,
    api_secret: String,
) -> Router<AppState> {
    let connect_router = signal_hub::register(
        communications::register(ConnectRouter::new(), pool.clone(), config.clone(), vault),
        pool,
        config,
    );
    Router::<AppState>::new()
        .fallback_service(connect_router.into_axum_router().into_service())
        .layer(middleware::from_fn_with_state(
            api_secret,
            guard::require_secret,
        ))
}
