use super::*;

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomOAuthStartRequest {
    pub account_id: String,
    pub display_name: String,
    pub external_account_id: String,
    pub account_email: Option<String>,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub client_secret_ref: Option<String>,
    pub webhook_secret_ref: Option<String>,
    pub redirect_uri: String,
    pub app_return_url: Option<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    pub authorization_endpoint: Option<String>,
    pub token_endpoint: Option<String>,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
}

impl ZoomOAuthStartRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("display_name", &self.display_name)?;
        validate_non_empty("external_account_id", &self.external_account_id)?;
        validate_non_empty("client_id", &self.client_id)?;
        validate_non_empty("redirect_uri", &self.redirect_uri)?;
        validate_optional_ref("client_secret_ref", &self.client_secret_ref)?;
        validate_optional_ref("webhook_secret_ref", &self.webhook_secret_ref)?;
        if self
            .client_secret
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(ZoomError::InvalidRequest(
                "client_secret must not be empty".to_owned(),
            ));
        }
        if self
            .client_secret
            .as_ref()
            .is_none_or(|value| value.trim().is_empty())
            && self
                .client_secret_ref
                .as_ref()
                .is_none_or(|value| value.trim().is_empty())
        {
            return Err(ZoomError::InvalidRequest(
                "client_secret or client_secret_ref is required for Zoom OAuth token exchange"
                    .to_owned(),
            ));
        }
        if let Some(endpoint) = &self.authorization_endpoint {
            validate_non_empty("authorization_endpoint", endpoint)?;
        }
        if let Some(endpoint) = &self.token_endpoint {
            validate_non_empty("token_endpoint", endpoint)?;
        }
        for scope in &self.scopes {
            validate_non_empty("scope", scope)?;
        }
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }

    pub(crate) fn live_account_request(&self) -> ZoomLiveAccountSetupRequest {
        ZoomLiveAccountSetupRequest {
            account_id: self.account_id.trim().to_owned(),
            display_name: self.display_name.trim().to_owned(),
            external_account_id: self.external_account_id.trim().to_owned(),
            account_email: trimmed_optional(&self.account_email).map(str::to_owned),
            auth_shape: ZoomAuthShape::OAuthUser,
            client_id: self.client_id.trim().to_owned(),
            token_secret_ref: None,
            client_secret_ref: trimmed_optional(&self.client_secret_ref).map(str::to_owned),
            webhook_secret_ref: trimmed_optional(&self.webhook_secret_ref).map(str::to_owned),
            metadata: json!({
                "oauth_setup": {
                    "redirect_uri": self.redirect_uri.trim(),
                    "requested_scopes": normalized_scopes(&self.scopes),
                    "authorization_endpoint": self.authorization_endpoint(),
                    "token_endpoint": self.token_endpoint(),
                    "client_secret_source": if trimmed_optional(&self.client_secret_ref).is_some() {
                        "secret_reference"
                    } else {
                        "runtime_request"
                    },
                    "secret_material": "excluded",
                },
                "metadata": &self.metadata,
            }),
        }
    }

    pub(crate) fn authorization_endpoint(&self) -> String {
        self.authorization_endpoint
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_ZOOM_AUTHORIZATION_ENDPOINT)
            .to_owned()
    }

    pub(crate) fn token_endpoint(&self) -> String {
        self.token_endpoint
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_ZOOM_TOKEN_ENDPOINT)
            .to_owned()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ZoomOAuthStartResponse {
    pub setup_id: String,
    pub authorization_url: String,
    pub state: String,
    pub redirect_uri: String,
}

pub struct ZoomOAuthPendingGrant {
    pub setup_id: String,
    pub account_id: String,
    pub authorization_url: String,
    pub state: String,
    pub request: ZoomOAuthStartRequest,
}

impl ZoomOAuthPendingGrant {
    pub fn response(&self) -> ZoomOAuthStartResponse {
        ZoomOAuthStartResponse {
            setup_id: self.setup_id.clone(),
            authorization_url: self.authorization_url.clone(),
            state: self.state.clone(),
            redirect_uri: self.request.redirect_uri.trim().to_owned(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomOAuthCompleteRequest {
    pub setup_id: String,
    pub state: String,
    pub authorization_code: String,
    pub external_account_id: Option<String>,
}

impl ZoomOAuthCompleteRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        validate_non_empty("setup_id", &self.setup_id)?;
        validate_non_empty("state", &self.state)?;
        validate_non_empty("authorization_code", &self.authorization_code)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomServerToServerAuthorizeRequest {
    pub account_id: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub client_secret_ref: Option<String>,
    pub zoom_account_id: Option<String>,
    pub token_endpoint: Option<String>,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
}

impl ZoomServerToServerAuthorizeRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_non_empty("client_id", &self.client_id)?;
        validate_optional_ref("client_secret_ref", &self.client_secret_ref)?;
        if self
            .client_secret
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(ZoomError::InvalidRequest(
                "client_secret must not be empty".to_owned(),
            ));
        }
        if self
            .client_secret
            .as_ref()
            .is_none_or(|value| value.trim().is_empty())
            && self
                .client_secret_ref
                .as_ref()
                .is_none_or(|value| value.trim().is_empty())
        {
            return Err(ZoomError::InvalidRequest(
                "client_secret or client_secret_ref is required for Zoom Server-to-Server token exchange"
                    .to_owned(),
            ));
        }
        if let Some(zoom_account_id) = &self.zoom_account_id {
            validate_non_empty("zoom_account_id", zoom_account_id)?;
        }
        if let Some(endpoint) = &self.token_endpoint {
            validate_non_empty("token_endpoint", endpoint)?;
        }
        validate_object("metadata", &self.metadata)?;
        Ok(())
    }

    pub(crate) fn token_endpoint(&self) -> String {
        self.token_endpoint
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_ZOOM_TOKEN_ENDPOINT)
            .to_owned()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomTokenRefreshRequest {
    pub account_id: String,
    #[serde(default)]
    pub force: bool,
    pub refresh_expiring_within_seconds: Option<i64>,
}

impl ZoomTokenRefreshRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        validate_non_empty("account_id", &self.account_id)?;
        validate_refresh_threshold(self.refresh_expiring_within_seconds)
    }

    pub(crate) fn refresh_expiring_within_seconds(&self) -> i64 {
        self.refresh_expiring_within_seconds
            .unwrap_or(ZOOM_EXPLICIT_TOKEN_REFRESH_THRESHOLD_SECONDS)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomTokenRefreshResult {
    pub account_id: String,
    pub provider_kind: String,
    pub auth_shape: String,
    pub token_secret_ref: String,
    pub refreshed: bool,
    pub refresh_strategy: String,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub checked_at: DateTime<Utc>,
    pub secret_kind: String,
    pub store_kind: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ZoomTokenMaintenanceRequest {
    pub account_id: Option<String>,
    #[serde(default)]
    pub force: bool,
    pub refresh_expiring_within_seconds: Option<i64>,
}

impl ZoomTokenMaintenanceRequest {
    pub fn validate(&self) -> Result<(), ZoomError> {
        if let Some(account_id) = &self.account_id {
            validate_non_empty("account_id", account_id)?;
        }
        validate_refresh_threshold(self.refresh_expiring_within_seconds)
    }

    pub(crate) fn refresh_expiring_within_seconds(&self) -> i64 {
        self.refresh_expiring_within_seconds
            .unwrap_or(ZOOM_TOKEN_MAINTENANCE_REFRESH_THRESHOLD_SECONDS)
    }

    pub(crate) fn refresh_request_for(&self, account_id: &str) -> ZoomTokenRefreshRequest {
        ZoomTokenRefreshRequest {
            account_id: account_id.trim().to_owned(),
            force: self.force,
            refresh_expiring_within_seconds: Some(self.refresh_expiring_within_seconds()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomTokenMaintenanceItem {
    pub account_id: String,
    pub provider_kind: String,
    pub auth_shape: String,
    pub status: String,
    pub refreshed: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ZoomTokenMaintenanceResult {
    pub checked_count: usize,
    pub refreshed_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
    pub refresh_expiring_within_seconds: i64,
    pub checked_at: DateTime<Utc>,
    pub items: Vec<ZoomTokenMaintenanceItem>,
}
