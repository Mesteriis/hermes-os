use super::super::{healthz, readyz};
use super::support::*;

pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route(
            "/api/v1/communications/mail/accounts/gmail/oauth/callback",
            get(get_gmail_oauth_callback),
        )
        .route(
            "/api/v1/communications/messages/{message_id}/remote-image",
            get(get_v1_communication_message_remote_image),
        )
}
