//! Explicit helpers for live persistence conformance against disposable PostgreSQL.

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use crate::{MailDurablePersistence, MailDurablePersistenceError};

pub struct MailPersistenceConformanceV1;

impl MailPersistenceConformanceV1 {
    pub async fn connect(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        database_id: &str,
    ) -> Result<MailDurablePersistence, MailDurablePersistenceError> {
        if host.trim().is_empty()
            || port == 0
            || username.trim().is_empty()
            || password.is_empty()
            || database_id.trim().is_empty()
        {
            return Err(MailDurablePersistenceError::InvalidRow);
        }
        let options = PgConnectOptions::new()
            .host(host)
            .port(port)
            .username(username)
            .password(password)
            .database(database_id);
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect_with(options)
            .await
            .map_err(|_| MailDurablePersistenceError::Database)?;
        Ok(MailDurablePersistence::new(pool))
    }
}
