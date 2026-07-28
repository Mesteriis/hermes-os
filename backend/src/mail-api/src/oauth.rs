use std::fmt;

use crate::valid_ca_certificate_pem;

pub const GMAIL_OAUTH_AUTHORIZATION_HOST: &str = "accounts.google.com";
pub const GMAIL_OAUTH_AUTHORIZATION_PATH: &str = "/o/oauth2/v2/auth";
pub const GMAIL_OAUTH_TOKEN_HOST: &str = "oauth2.googleapis.com";
pub const GMAIL_OAUTH_TOKEN_PATH: &str = "/token";
pub const GMAIL_OAUTH_HTTPS_PORT: u16 = 443;
pub const GMAIL_OAUTH_ATTEMPT_TTL_SECONDS: i64 = 600;
pub const MAX_GMAIL_OAUTH_VALUE_BYTES: usize = 8 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailOAuthEndpointV1 {
    pub host: String,
    pub port: u16,
    pub path: String,
    pub ca_certificate_pem: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailOAuthConfigurationV1 {
    pub client_id: String,
    pub redirect_uri: String,
    pub authorization_endpoint: GmailOAuthEndpointV1,
    pub token_endpoint: GmailOAuthEndpointV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailOAuthStartRequestV1 {
    pub operation_id: String,
    pub authority: GmailOAuthAuthorityV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GmailOAuthAuthorityV1 {
    Operational,
    PermanentDelete,
}

#[derive(Clone, Eq, PartialEq)]
pub struct GmailOAuthStartedV1 {
    pub operation_id: String,
    pub setup_id: String,
    pub authorization_url: String,
    pub expires_at_unix_seconds: i64,
}

#[derive(Clone, Eq, PartialEq)]
pub struct GmailOAuthCompleteRequestV1 {
    pub operation_id: String,
    pub setup_id: String,
    pub state: String,
    pub authorization_code: String,
}

impl fmt::Debug for GmailOAuthStartedV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GmailOAuthStartedV1")
            .field("operation_id", &self.operation_id)
            .field("setup_id", &self.setup_id)
            .field("authorization_url", &"[redacted]")
            .field("expires_at_unix_seconds", &self.expires_at_unix_seconds)
            .finish()
    }
}

impl fmt::Debug for GmailOAuthCompleteRequestV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GmailOAuthCompleteRequestV1")
            .field("operation_id", &self.operation_id)
            .field("setup_id", &self.setup_id)
            .field("state", &"[redacted]")
            .field("authorization_code", &"[redacted]")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailOAuthRefreshRequestV1 {
    pub operation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailOAuthStatusRequestV1 {
    pub operation_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GmailOAuthOperationKindV1 {
    Complete,
    Refresh,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GmailOAuthOutcomeV1 {
    Pending,
    Completed,
    Rejected,
    OutcomeUnknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailOAuthOperationStatusV1 {
    pub operation_id: String,
    pub kind: GmailOAuthOperationKindV1,
    pub outcome: GmailOAuthOutcomeV1,
    pub requested_at_unix_seconds: i64,
    pub completed_at_unix_seconds: Option<i64>,
}

#[must_use]
pub fn valid_gmail_oauth_configuration(configuration: &GmailOAuthConfigurationV1) -> bool {
    valid_oauth_client_id(&configuration.client_id)
        && valid_redirect_uri(&configuration.redirect_uri)
        && valid_oauth_endpoint(
            &configuration.authorization_endpoint,
            GMAIL_OAUTH_AUTHORIZATION_HOST,
            GMAIL_OAUTH_AUTHORIZATION_PATH,
        )
        && valid_oauth_endpoint(
            &configuration.token_endpoint,
            GMAIL_OAUTH_TOKEN_HOST,
            GMAIL_OAUTH_TOKEN_PATH,
        )
}

fn valid_oauth_endpoint(
    endpoint: &GmailOAuthEndpointV1,
    production_host: &str,
    production_path: &str,
) -> bool {
    if !valid_oauth_host(&endpoint.host)
        || endpoint.port == 0
        || !valid_oauth_path(&endpoint.path)
        || endpoint
            .ca_certificate_pem
            .as_deref()
            .is_some_and(|value| !valid_ca_certificate_pem(value))
    {
        return false;
    }
    let production = endpoint.host == production_host
        && endpoint.port == GMAIL_OAUTH_HTTPS_PORT
        && endpoint.path == production_path
        && endpoint.ca_certificate_pem.is_none();
    if production {
        return true;
    }
    #[cfg(feature = "conformance-test-support")]
    {
        matches!(endpoint.host.as_str(), "127.0.0.1" | "localhost")
            && endpoint.ca_certificate_pem.is_some()
    }
    #[cfg(not(feature = "conformance-test-support"))]
    false
}

fn valid_oauth_client_id(value: &str) -> bool {
    valid_bounded_ascii(value)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

fn valid_redirect_uri(value: &str) -> bool {
    valid_bounded_ascii(value)
        && !value.contains(['\r', '\n', '\0', '#'])
        && (value.starts_with("http://127.0.0.1")
            || value.starts_with("http://localhost")
            || value.starts_with("https://"))
}

fn valid_oauth_host(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
}

fn valid_oauth_path(value: &str) -> bool {
    value.starts_with('/')
        && value.len() <= 4096
        && value.is_ascii()
        && !value.contains(['\r', '\n', '\0', '?', '#'])
}

fn valid_bounded_ascii(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_GMAIL_OAUTH_VALUE_BYTES && value.is_ascii()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn production_configuration() -> GmailOAuthConfigurationV1 {
        GmailOAuthConfigurationV1 {
            client_id: "client-id.apps.googleusercontent.com".to_owned(),
            redirect_uri: "http://127.0.0.1:38123/oauth/callback".to_owned(),
            authorization_endpoint: GmailOAuthEndpointV1 {
                host: GMAIL_OAUTH_AUTHORIZATION_HOST.to_owned(),
                port: GMAIL_OAUTH_HTTPS_PORT,
                path: GMAIL_OAUTH_AUTHORIZATION_PATH.to_owned(),
                ca_certificate_pem: None,
            },
            token_endpoint: GmailOAuthEndpointV1 {
                host: GMAIL_OAUTH_TOKEN_HOST.to_owned(),
                port: GMAIL_OAUTH_HTTPS_PORT,
                path: GMAIL_OAUTH_TOKEN_PATH.to_owned(),
                ca_certificate_pem: None,
            },
        }
    }

    #[test]
    fn production_oauth_configuration_is_exact() {
        let configuration = production_configuration();
        assert!(valid_gmail_oauth_configuration(&configuration));

        let mut drifted = configuration;
        drifted.token_endpoint.host = "oauth.example.test".to_owned();
        assert!(!valid_gmail_oauth_configuration(&drifted));
    }

    #[test]
    fn redirect_uri_rejects_fragment_and_control_bytes() {
        let mut configuration = production_configuration();
        configuration.redirect_uri = "https://desktop.example.test/callback#token".to_owned();
        assert!(!valid_gmail_oauth_configuration(&configuration));
        configuration.redirect_uri = "https://desktop.example.test/callback\r\n".to_owned();
        assert!(!valid_gmail_oauth_configuration(&configuration));
    }

    #[test]
    fn certificate_bound_matches_the_shared_contract_limit() {
        assert_eq!(crate::MAX_CA_CERTIFICATE_PEM_BYTES, 64 * 1024);
    }

    #[test]
    fn sensitive_oauth_debug_is_redacted() {
        let started = GmailOAuthStartedV1 {
            operation_id: "operation".to_owned(),
            setup_id: "setup".to_owned(),
            authorization_url: "https://accounts.google.test?state=private-state".to_owned(),
            expires_at_unix_seconds: 1,
        };
        let complete = GmailOAuthCompleteRequestV1 {
            operation_id: "operation".to_owned(),
            setup_id: "setup".to_owned(),
            state: "private-state".to_owned(),
            authorization_code: "private-code".to_owned(),
        };
        let debug = format!("{started:?} {complete:?}");
        assert!(!debug.contains("private-state"));
        assert!(!debug.contains("private-code"));
    }
}
