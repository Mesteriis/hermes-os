//! Typed, bounded resolver-update inputs that keep credentials out of process globals.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::Deserialize;
use zeroize::Zeroizing;

const MAX_ACCOUNT_JWT_BYTES: usize = 16 * 1024;
const MAX_SYSTEM_CREDENTIAL_BYTES: usize = 16 * 1024;

/// A broker-verified Account JWT bound to the exact Account NKey in its subject.
pub struct NatsAccountJwtUpdateV1 {
    account_public_key: String,
    jwt: Zeroizing<String>,
}

/// A System Account `.creds` document received from a scoped secret provider.
pub struct NatsResolverSystemCredentialsV1 {
    document: Zeroizing<String>,
}

impl NatsAccountJwtUpdateV1 {
    pub fn new(
        account_public_key: impl Into<String>,
        jwt: impl Into<String>,
    ) -> Result<Self, ResolverUpdateErrorV1> {
        let account_public_key = account_public_key.into();
        let jwt = Zeroizing::new(jwt.into());
        if !valid_account_key(&account_public_key) || jwt.len() > MAX_ACCOUNT_JWT_BYTES {
            return Err(ResolverUpdateErrorV1::InvalidUpdate);
        }
        let claims = decode_account_claims(&jwt)?;
        if claims.sub != account_public_key {
            return Err(ResolverUpdateErrorV1::AccountMismatch);
        }
        if !valid_operator_key(&claims.iss) {
            return Err(ResolverUpdateErrorV1::InvalidIssuer);
        }
        if claims.nats.claim_type != "account" {
            return Err(ResolverUpdateErrorV1::InvalidUpdate);
        }
        if claims
            .nats
            .signing_keys
            .iter()
            .any(|key| !valid_account_key(key))
        {
            return Err(ResolverUpdateErrorV1::InvalidSigningKey);
        }
        Ok(Self {
            account_public_key,
            jwt,
        })
    }

    #[must_use]
    pub fn account_public_key(&self) -> &str {
        &self.account_public_key
    }

    pub(crate) fn jwt(&self) -> &str {
        self.jwt.as_str()
    }
}

impl NatsResolverSystemCredentialsV1 {
    pub fn new(document: impl Into<String>) -> Result<Self, ResolverUpdateErrorV1> {
        let document = Zeroizing::new(document.into());
        (document.len() <= MAX_SYSTEM_CREDENTIAL_BYTES
            && !document.contains('\0')
            && document.contains("-----BEGIN NATS USER JWT-----")
            && document.contains("-----BEGIN USER NKEY SEED-----"))
        .then_some(Self { document })
        .ok_or(ResolverUpdateErrorV1::InvalidCredentials)
    }

    pub(crate) fn document(&self) -> &str {
        self.document.as_str()
    }
}

fn decode_account_claims(jwt: &str) -> Result<AccountJwtClaims, ResolverUpdateErrorV1> {
    let mut parts = jwt.split('.');
    let (Some(header), Some(payload), Some(signature), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(ResolverUpdateErrorV1::MalformedJwt);
    };
    if header.is_empty() || payload.is_empty() || signature.is_empty() {
        return Err(ResolverUpdateErrorV1::MalformedJwt);
    }
    let payload = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| ResolverUpdateErrorV1::MalformedJwt)?;
    serde_json::from_slice(&payload).map_err(|_| ResolverUpdateErrorV1::MalformedJwt)
}

#[derive(Deserialize)]
struct AccountJwtClaims {
    iss: String,
    sub: String,
    nats: AccountNatsClaims,
}

#[derive(Deserialize)]
struct AccountNatsClaims {
    #[serde(rename = "type")]
    claim_type: String,
    #[serde(default)]
    signing_keys: Vec<String>,
}

fn valid_account_key(value: &str) -> bool {
    value.len() == 56
        && value.starts_with('A')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

fn valid_operator_key(value: &str) -> bool {
    value.len() == 56
        && value.starts_with('O')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolverUpdateErrorV1 {
    AccountMismatch,
    InvalidCredentials,
    InvalidEndpoint,
    InvalidIssuer,
    InvalidSigningKey,
    InvalidUpdate,
    MalformedJwt,
    Unavailable,
    Rejected,
}
