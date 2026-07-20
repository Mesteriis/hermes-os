//! The one SQLx pool owned by the PostgreSQL admin adapter.

use sqlx::{PgPool, postgres::PgPoolOptions};
use zeroize::Zeroizing;

use super::PostgresAdapterErrorV1;

pub struct PostgresAdminConnectorV1 {
    pool: PgPool,
}

impl PostgresAdminConnectorV1 {
    pub async fn connect(database_url: &str) -> Result<Self, PostgresAdapterErrorV1> {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(database_url)
            .await
            .map_err(|_| PostgresAdapterErrorV1::Connection)?;
        Ok(Self { pool })
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

    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }
}

pub(crate) fn connection_url(
    host: &str,
    port: u16,
    database: &str,
    username: &str,
    password: &[u8],
) -> Result<Zeroizing<String>, PostgresAdapterErrorV1> {
    let password = std::str::from_utf8(password).map_err(|_| PostgresAdapterErrorV1::Connection)?;
    if !valid_host(host)
        || !valid_token(database)
        || !valid_token(username)
        || !valid_password(password)
    {
        return Err(PostgresAdapterErrorV1::Connection);
    }
    let host = url_host(host);
    Ok(Zeroizing::new(format!(
        "postgres://{username}:{password}@{host}:{port}/{database}"
    )))
}

fn valid_host(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b':'))
}

fn url_host(value: &str) -> String {
    if value.contains(':') {
        format!("[{value}]")
    } else {
        value.to_owned()
    }
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

fn valid_password(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 4 * 1024
        && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
