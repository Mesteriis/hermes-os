use crate::app::handlers::communications::account_management::*;
use crate::app::handlers::communications::account_provider_resources::{
    get_v1_email_account_provider_resources, put_v1_email_account_provider_resource_mapping,
};
use crate::app::handlers::communications::account_setup::{
    gmail_oauth::{post_gmail_oauth_complete, post_gmail_oauth_start},
    imap::post_imap_account_setup,
};
use crate::app::state::AppState;
use axum::Router;
use axum::routing::{delete, get, post, put};

pub(super) fn add_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route(
            "/api/v1/integrations/mail/accounts/gmail/oauth/start",
            post(post_gmail_oauth_start),
        )
        .route(
            "/api/v1/integrations/mail/accounts/gmail/oauth/complete",
            post(post_gmail_oauth_complete),
        )
        .route(
            "/api/v1/integrations/mail/accounts",
            get(get_v1_email_accounts),
        )
        .route(
            "/api/v1/communications/email/accounts",
            get(get_v1_email_accounts),
        )
        .route(
            "/api/v1/integrations/mail/accounts/import",
            post(post_v1_email_account_import),
        )
        .route(
            "/api/v1/integrations/mail/accounts/imap",
            post(post_imap_account_setup),
        )
        .route(
            "/api/v1/integrations/mail/accounts/{account_id}",
            get(get_v1_email_account).delete(delete_v1_email_account),
        )
        .route(
            "/api/v1/integrations/mail/accounts/{account_id}/export",
            get(get_v1_email_account_export),
        )
        .route(
            "/api/v1/integrations/mail/accounts/{account_id}/logout",
            post(post_v1_email_account_logout),
        )
        .route(
            "/api/v1/integrations/mail/accounts/sync-status",
            get(get_v1_email_account_sync_status),
        )
        .route(
            "/api/v1/integrations/mail/accounts/{account_id}/sync-settings",
            get(get_v1_email_account_sync_settings).put(put_v1_email_account_sync_settings),
        )
        .route(
            "/api/v1/integrations/mail/accounts/{account_id}/content-egress-settings",
            get(get_v1_email_account_content_egress_settings)
                .put(put_v1_email_account_content_egress_settings),
        )
        .route(
            "/api/v1/integrations/mail/accounts/{account_id}/sensitive-forwarding-policies",
            get(get_v1_mail_sensitive_forwarding_policies)
                .post(post_v1_mail_sensitive_forwarding_policy),
        )
        .route(
            "/api/v1/integrations/mail/accounts/{account_id}/sensitive-forwarding-policies/{policy_id}",
            delete(delete_v1_mail_sensitive_forwarding_policy),
        )
        .route(
            "/api/v1/integrations/mail/accounts/{account_id}/provider-resources",
            get(get_v1_email_account_provider_resources),
        )
        .route(
            "/api/v1/integrations/mail/accounts/{account_id}/provider-resources/{mapping_id}",
            put(put_v1_email_account_provider_resource_mapping),
        )
        .route(
            "/api/v1/integrations/mail/accounts/{account_id}/sync-now",
            post(post_v1_email_account_sync_now),
        )
        .route(
            "/api/v1/integrations/mail/accounts/{account_id}/address-book-sync-now",
            post(post_v1_email_account_address_book_sync_now),
        )
        .route(
            "/api/v1/integrations/mail/accounts/{account_id}/sync-full-resync",
            post(post_v1_email_account_sync_full_resync),
        )
}
