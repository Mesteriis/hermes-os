use serde::Deserialize;

use crate::platform::secrets::models::ResolvedSecret;

use super::errors::ConfigError;
use super::parsing::required_trimmed;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoogleOAuthClientType {
    Installed,
    Web,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoogleOAuthClientConfig {
    client_type: GoogleOAuthClientType,
    client_id: String,
    client_secret: Option<ResolvedSecret>,
    authorization_endpoint: String,
    token_endpoint: String,
    redirect_uris: Vec<String>,
}

impl GoogleOAuthClientConfig {
    pub(crate) fn from_client_secret_json(raw_json: &str) -> Result<Self, ConfigError> {
        let file: GoogleOAuthClientSecretsFile =
            serde_json::from_str(raw_json).map_err(ConfigError::GoogleOAuthClientConfigJson)?;
        if let Some(installed) = file.installed {
            return Self::from_payload(GoogleOAuthClientType::Installed, installed);
        }
        if let Some(web) = file.web {
            return Self::from_payload(GoogleOAuthClientType::Web, web);
        }

        Err(ConfigError::InvalidGoogleOAuthClientConfig {
            field: "client_type",
            message: "must contain installed or web client credentials",
        })
    }

    fn from_payload(
        client_type: GoogleOAuthClientType,
        payload: GoogleOAuthClientSecretsPayload,
    ) -> Result<Self, ConfigError> {
        let client_id = required_trimmed("client_id", payload.client_id)?;
        let authorization_endpoint = required_trimmed("auth_uri", payload.auth_uri)?;
        let token_endpoint = required_trimmed("token_uri", payload.token_uri)?;
        let client_secret = payload
            .client_secret
            .map(|secret| required_trimmed("client_secret", Some(secret)))
            .transpose()?
            .map(ResolvedSecret::new)
            .transpose()?;
        let redirect_uris = payload
            .redirect_uris
            .unwrap_or_default()
            .into_iter()
            .map(|uri| required_trimmed("redirect_uris", Some(uri)))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            client_type,
            client_id,
            client_secret,
            authorization_endpoint,
            token_endpoint,
            redirect_uris,
        })
    }

    pub fn client_type(&self) -> GoogleOAuthClientType {
        self.client_type
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn client_secret(&self) -> Option<&ResolvedSecret> {
        self.client_secret.as_ref()
    }

    pub fn authorization_endpoint(&self) -> &str {
        &self.authorization_endpoint
    }

    pub fn token_endpoint(&self) -> &str {
        &self.token_endpoint
    }

    pub fn redirect_uris(&self) -> &[String] {
        &self.redirect_uris
    }
}

#[derive(Debug, Deserialize)]
struct GoogleOAuthClientSecretsFile {
    installed: Option<GoogleOAuthClientSecretsPayload>,
    web: Option<GoogleOAuthClientSecretsPayload>,
}

#[derive(Debug, Deserialize)]
struct GoogleOAuthClientSecretsPayload {
    client_id: Option<String>,
    client_secret: Option<String>,
    auth_uri: Option<String>,
    token_uri: Option<String>,
    redirect_uris: Option<Vec<String>>,
}
