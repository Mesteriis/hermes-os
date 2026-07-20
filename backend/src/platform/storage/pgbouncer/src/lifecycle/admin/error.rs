//! Sanitized failures for the administrative connection setup.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PgBouncerAdminConnectionErrorV1 {
    InvalidEndpoint,
    InvalidCredential,
    Unavailable,
}
