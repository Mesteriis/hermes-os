use super::*;

pub(super) fn zoom_oauth_secret_metadata(
    account: &ZoomAccount,
    auth_shape: &str,
    expires_at: Option<DateTime<Utc>>,
    metadata: &Value,
) -> Value {
    json!({
        "provider": "zoom",
        "provider_kind": account.provider_kind,
        "account_id": account.account_id,
        "external_account_id": account.external_account_id,
        "auth_shape": auth_shape,
        "secret_purpose": ProviderAccountSecretPurpose::ZoomOauthToken.as_str(),
        "secret_material": "excluded",
        "expires_at": expires_at,
        "metadata": metadata,
    })
}

pub(super) fn zoom_client_secret_metadata(account: &ZoomAccount, auth_shape: &str) -> Value {
    json!({
        "provider": "zoom",
        "provider_kind": account.provider_kind,
        "account_id": account.account_id,
        "external_account_id": account.external_account_id,
        "auth_shape": auth_shape,
        "secret_purpose": ProviderAccountSecretPurpose::ZoomClientSecret.as_str(),
        "secret_material": "excluded",
    })
}

pub(super) fn resolve_client_secret(
    vault: &HostVault,
    client_secret: Option<&str>,
    client_secret_ref: Option<&str>,
) -> Result<String, ZoomError> {
    if let Some(client_secret) = client_secret
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(client_secret.to_owned());
    }
    let secret_ref = client_secret_ref
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ZoomError::InvalidRequest(
                "Zoom client secret is required for token exchange".to_owned(),
            )
        })?;
    vault.read_secret(secret_ref).map_err(Into::into)
}

pub(super) fn validate_required(field: &'static str, value: &str) -> Result<String, ZoomError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ZoomError::InvalidRequest(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

pub(super) fn zoom_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest client configuration must be valid")
}

pub(super) fn validate_account_id(account_id: &str) -> Result<String, ZoomError> {
    let trimmed = account_id.trim();
    if trimmed.is_empty() {
        return Err(ZoomError::InvalidRequest(
            "account_id must not be empty".to_owned(),
        ));
    }
    Ok(trimmed.to_owned())
}
