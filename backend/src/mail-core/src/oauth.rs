use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

const SETUP_ID_ENTROPY_BYTES: usize = 16;
const STATE_ENTROPY_BYTES: usize = 32;
const VERIFIER_ENTROPY_BYTES: usize = 32;

pub const GOOGLE_CONTACTS_WRITE_SCOPE_V1: &str = "https://www.googleapis.com/auth/contacts";

#[derive(Clone, Eq, PartialEq)]
pub struct GmailOAuthAttemptMaterialV1 {
    pub setup_id: String,
    pub state: String,
    pub state_sha256: [u8; 32],
    pub code_verifier: String,
    pub code_challenge: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GmailOAuthPolicyErrorV1 {
    InvalidEntropy,
    InvalidState,
}

pub fn derive_gmail_oauth_attempt(
    setup_id_entropy: &[u8],
    state_entropy: &[u8],
    verifier_entropy: &[u8],
) -> Result<GmailOAuthAttemptMaterialV1, GmailOAuthPolicyErrorV1> {
    if setup_id_entropy.len() != SETUP_ID_ENTROPY_BYTES
        || state_entropy.len() != STATE_ENTROPY_BYTES
        || verifier_entropy.len() != VERIFIER_ENTROPY_BYTES
        || setup_id_entropy.iter().all(|byte| *byte == 0)
        || state_entropy.iter().all(|byte| *byte == 0)
        || verifier_entropy.iter().all(|byte| *byte == 0)
    {
        return Err(GmailOAuthPolicyErrorV1::InvalidEntropy);
    }
    let setup_id = URL_SAFE_NO_PAD.encode(setup_id_entropy);
    let state = URL_SAFE_NO_PAD.encode(state_entropy);
    let code_verifier = URL_SAFE_NO_PAD.encode(verifier_entropy);
    let code_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));
    Ok(GmailOAuthAttemptMaterialV1 {
        setup_id,
        state_sha256: Sha256::digest(state.as_bytes()).into(),
        state,
        code_verifier,
        code_challenge,
    })
}

pub fn validate_gmail_oauth_state(
    expected_sha256: &[u8; 32],
    state: &str,
) -> Result<(), GmailOAuthPolicyErrorV1> {
    let actual = gmail_oauth_state_sha256(state)?;
    let mismatch = actual
        .iter()
        .zip(expected_sha256)
        .fold(0_u8, |difference, (left, right)| {
            difference | (left ^ right)
        });
    (mismatch == 0)
        .then_some(())
        .ok_or(GmailOAuthPolicyErrorV1::InvalidState)
}

pub fn gmail_oauth_state_sha256(state: &str) -> Result<[u8; 32], GmailOAuthPolicyErrorV1> {
    if state.is_empty() || state.len() > 1024 || !state.is_ascii() {
        return Err(GmailOAuthPolicyErrorV1::InvalidState);
    }
    Ok(Sha256::digest(state.as_bytes()).into())
}

#[must_use]
pub fn gmail_oauth_authorization_code_sha256(code: &str) -> [u8; 32] {
    Sha256::digest(code.as_bytes()).into()
}

#[must_use]
pub fn gmail_oauth_scope_sha256(scope: Option<&str>) -> [u8; 32] {
    Sha256::digest(scope.unwrap_or_default().as_bytes()).into()
}

#[must_use]
pub fn gmail_oauth_scope_authorizes_contacts_write(scope: Option<&str>) -> bool {
    scope.is_some_and(|scope| {
        scope
            .split_ascii_whitespace()
            .any(|granted| granted == GOOGLE_CONTACTS_WRITE_SCOPE_V1)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_bounded_pkce_s256_material_and_validates_state() {
        let material = derive_gmail_oauth_attempt(&[1; 16], &[2; 32], &[3; 32])
            .expect("deterministic PKCE material");
        assert_eq!(material.setup_id.len(), 22);
        assert_eq!(material.state.len(), 43);
        assert_eq!(material.code_verifier.len(), 43);
        assert_eq!(material.code_challenge.len(), 43);
        assert_eq!(
            validate_gmail_oauth_state(&material.state_sha256, &material.state),
            Ok(())
        );
        assert_eq!(
            validate_gmail_oauth_state(&material.state_sha256, "different-state"),
            Err(GmailOAuthPolicyErrorV1::InvalidState)
        );
    }

    #[test]
    fn rejects_zero_or_wrong_sized_entropy() {
        assert!(matches!(
            derive_gmail_oauth_attempt(&[0; 16], &[2; 32], &[3; 32]),
            Err(GmailOAuthPolicyErrorV1::InvalidEntropy)
        ));
        assert!(matches!(
            derive_gmail_oauth_attempt(&[1; 15], &[2; 32], &[3; 32]),
            Err(GmailOAuthPolicyErrorV1::InvalidEntropy)
        ));
    }

    #[test]
    fn contacts_write_authority_requires_the_exact_scope_token() {
        assert!(gmail_oauth_scope_authorizes_contacts_write(Some(
            "openid https://www.googleapis.com/auth/contacts email"
        )));
        assert!(!gmail_oauth_scope_authorizes_contacts_write(Some(
            "openid https://www.googleapis.com/auth/contacts.readonly email"
        )));
        assert!(!gmail_oauth_scope_authorizes_contacts_write(None));
    }
}
