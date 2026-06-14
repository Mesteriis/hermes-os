use super::super::*;

pub(super) fn gmail_pending_external_account_id(pending: &GmailOAuthPendingGrant) -> String {
    trimmed_optional(Some(pending.request.external_account_id.clone()))
        .unwrap_or_else(|| pending.account_id.clone())
}

pub(super) fn trimmed_optional(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_owned())
        .filter(|trimmed| !trimmed.is_empty())
}
