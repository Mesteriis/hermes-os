use super::*;
use crate::integrations::zoom::client::models::ZoomAuthorizationResult;

pub(super) fn canonical_zoom_webhook_event_types(event_types: &[String]) -> Vec<String> {
    let mut normalized = event_types
        .iter()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

pub(super) fn find_managed_zoom_webhook_subscription<'a>(
    subscriptions: &'a [ZoomWebhookSubscription],
    managed_subscription_id: Option<&str>,
    subscription_name: &str,
) -> Option<&'a ZoomWebhookSubscription> {
    managed_subscription_id
        .and_then(|subscription_id| {
            subscriptions
                .iter()
                .find(|subscription| subscription.subscription_id == subscription_id.trim())
        })
        .or_else(|| {
            subscriptions
                .iter()
                .find(|subscription| subscription.subscription_name == subscription_name.trim())
        })
}

pub(super) fn authorization_result(
    account: ZoomAccount,
    token_secret_ref: String,
    client_secret_ref: Option<String>,
    authorized_at: DateTime<Utc>,
) -> ZoomAuthorizationResult {
    ZoomAuthorizationResult {
        account_id: account.account_id,
        provider_kind: account.provider_kind,
        auth_shape: account.auth_shape,
        lifecycle_state: account.lifecycle_state,
        runtime_kind: account.runtime_kind,
        token_secret_ref,
        client_secret_ref,
        secret_kind: SecretKind::OauthToken.as_str().to_owned(),
        store_kind: SecretStoreKind::HostVault.as_str().to_owned(),
        authorized_at,
    }
}

pub(super) fn refresh_flow_label(auth_shape: &str) -> &'static str {
    if auth_shape == ZoomAuthShape::OAuthUser.as_str() {
        "oauth_user"
    } else if auth_shape == ZoomAuthShape::ServerToServer.as_str() {
        "server_to_server"
    } else {
        "unknown"
    }
}
