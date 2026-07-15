use super::super::{healthz, readyz};
use crate::ai::api::control_center::get_ai_provider_auth_callback;
use crate::app::handlers::communications::account_setup::gmail_callback::get_gmail_oauth_callback;
use crate::app::handlers::communications::remote_images::handler::get_v1_communication_message_remote_image;
use crate::app::state::AppState;
use axum::Router;
use axum::routing::get;

pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route(
            "/api/v1/integrations/mail/accounts/gmail/oauth/callback",
            get(get_gmail_oauth_callback),
        )
        .route(
            "/api/v1/ai/provider-auth/callback",
            get(get_ai_provider_auth_callback),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/remote-image",
            get(get_v1_communication_message_remote_image),
        )
}
