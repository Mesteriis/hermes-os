//! Narrow runtime-session probe used by storage conformance checks.

use sqlx::{Connection, PgConnection, query_scalar};
use zeroize::Zeroizing;

use super::{PostgresAdapterErrorV1, admin::connection_url};

pub struct PostgresRuntimeSessionProbeV1 {
    connection: PgConnection,
}

impl PostgresRuntimeSessionProbeV1 {
    pub async fn connect(database_url: &str) -> Result<Self, PostgresAdapterErrorV1> {
        let connection = PgConnection::connect(database_url)
            .await
            .map_err(|_| PostgresAdapterErrorV1::Connection)?;
        Ok(Self { connection })
    }

    pub async fn connect_with_password(
        host: &str,
        port: u16,
        database: &str,
        username: &str,
        password: &Zeroizing<Vec<u8>>,
    ) -> Result<Self, PostgresAdapterErrorV1> {
        let url = connection_url(host, port, database, username, password)?;
        Self::connect(&url).await
    }

    pub async fn current_principal(&mut self) -> Result<String, PostgresAdapterErrorV1> {
        query_scalar("SELECT current_user")
            .fetch_one(&mut self.connection)
            .await
            .map_err(|_| PostgresAdapterErrorV1::Query)
    }

    pub async fn remains_connected(&mut self) -> bool {
        query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&mut self.connection)
            .await
            .is_ok()
    }
}
