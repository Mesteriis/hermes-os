//! Sanitized PgBouncer database-catalog lookup for configuration conformance.

use tokio_postgres::SimpleQueryMessage;

use crate::PoolAliasV1;

use super::{
    PgBouncerAdminConnectionErrorV1, PgBouncerAdminCredentialV1, PgBouncerAdminEndpointV1,
    TokioPostgresPgBouncerAdminPortV1,
};

pub async fn database_is_configured(
    endpoint: &PgBouncerAdminEndpointV1,
    credential: &PgBouncerAdminCredentialV1,
    alias: &PoolAliasV1,
) -> Result<bool, PgBouncerAdminConnectionErrorV1> {
    let port = TokioPostgresPgBouncerAdminPortV1::connect(endpoint, credential).await?;
    let rows = port
        .client
        .simple_query("SHOW DATABASES")
        .await
        .map_err(|_| PgBouncerAdminConnectionErrorV1::Unavailable)?;
    Ok(rows
        .into_iter()
        .any(|message| matches_alias(message, alias)))
}

fn matches_alias(message: SimpleQueryMessage, alias: &PoolAliasV1) -> bool {
    match message {
        SimpleQueryMessage::Row(row) => row.get(0) == Some(alias.as_str()),
        _ => false,
    }
}
