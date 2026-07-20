//! Bounded publication of one already-signed Account JWT to the NATS full resolver.

use std::time::Duration;

use super::{NatsAccountJwtUpdateV1, NatsResolverSystemCredentialsV1, ResolverUpdateErrorV1};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_RESPONSE_BYTES: usize = 4 * 1024;

/// Publishes only an Account JWT that was signed by a separate authority.
pub struct NatsResolverAccountJwtPublisherV1;

impl NatsResolverAccountJwtPublisherV1 {
    pub async fn publish(
        endpoint: &str,
        credentials: &NatsResolverSystemCredentialsV1,
        update: &NatsAccountJwtUpdateV1,
    ) -> Result<(), ResolverUpdateErrorV1> {
        validate_endpoint(endpoint)?;
        let options = async_nats::ConnectOptions::with_credentials(credentials.document())
            .map_err(|_| ResolverUpdateErrorV1::InvalidCredentials)?;
        let client = tokio::time::timeout(
            CONNECT_TIMEOUT,
            options
                .connection_timeout(CONNECT_TIMEOUT)
                .connect(endpoint),
        )
        .await
        .map_err(|_| ResolverUpdateErrorV1::Unavailable)?
        .map_err(|_| ResolverUpdateErrorV1::Unavailable)?;
        let subject = format!(
            "$SYS.REQ.ACCOUNT.{}.CLAIMS.UPDATE",
            update.account_public_key()
        );
        let response = tokio::time::timeout(
            REQUEST_TIMEOUT,
            client.request(subject, update.jwt().as_bytes().to_vec().into()),
        )
        .await
        .map_err(|_| ResolverUpdateErrorV1::Unavailable)?
        .map_err(|_| ResolverUpdateErrorV1::Unavailable)?;
        (response.status.is_none() && response.payload.len() <= MAX_RESPONSE_BYTES)
            .then_some(())
            .ok_or(ResolverUpdateErrorV1::Rejected)
    }
}

fn validate_endpoint(endpoint: &str) -> Result<(), ResolverUpdateErrorV1> {
    (endpoint.starts_with("nats://") && !endpoint.contains(['@', '?', '#', ' ']))
        .then_some(())
        .ok_or(ResolverUpdateErrorV1::InvalidEndpoint)
}
