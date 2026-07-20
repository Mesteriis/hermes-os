//! Keeps the short-lived PgBouncer administrative credential non-debuggable.

use zeroize::Zeroizing;

use super::PgBouncerAdminConnectionErrorV1;

pub struct PgBouncerAdminCredentialV1 {
    username: String,
    password: Zeroizing<String>,
}

impl PgBouncerAdminCredentialV1 {
    pub fn new(
        username: String,
        password: Zeroizing<String>,
    ) -> Result<Self, PgBouncerAdminConnectionErrorV1> {
        valid_identifier(&username)
            .then_some(Self { username, password })
            .ok_or(PgBouncerAdminConnectionErrorV1::InvalidCredential)
    }

    pub(super) fn username(&self) -> &str {
        &self.username
    }

    pub(super) fn password(&self) -> &str {
        &self.password
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
