//! Validates the separate PgBouncer administrative endpoint.

use super::PgBouncerAdminConnectionErrorV1;

pub struct PgBouncerAdminEndpointV1 {
    host: String,
    port: u16,
}

impl PgBouncerAdminEndpointV1 {
    pub fn new(host: String, port: u16) -> Result<Self, PgBouncerAdminConnectionErrorV1> {
        (valid_host(&host) && port != 0)
            .then_some(Self { host, port })
            .ok_or(PgBouncerAdminConnectionErrorV1::InvalidEndpoint)
    }

    pub(super) fn host(&self) -> &str {
        &self.host
    }

    pub(super) const fn port(&self) -> u16 {
        self.port
    }
}

fn valid_host(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
}
