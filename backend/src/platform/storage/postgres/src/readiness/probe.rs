//! Readiness remains a bounded query and never returns connection details.

use sqlx::query_scalar;

use crate::{PostgresAdapterErrorV1, PostgresAdminConnectorV1};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostgresReadinessV1 {
    database_id: String,
}

impl PostgresReadinessV1 {
    pub fn database_id(&self) -> &str {
        &self.database_id
    }
}

pub async fn read_readiness(
    connector: &PostgresAdminConnectorV1,
) -> Result<PostgresReadinessV1, PostgresAdapterErrorV1> {
    let database_id = query_scalar::<_, String>("SELECT current_database()")
        .fetch_one(connector.pool())
        .await
        .map_err(|_| PostgresAdapterErrorV1::Query)?;
    Ok(PostgresReadinessV1 { database_id })
}
