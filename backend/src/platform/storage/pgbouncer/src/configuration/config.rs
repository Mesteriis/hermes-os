//! Renders one `[databases]` entry without any credential material.

use super::{PoolAliasV1, PoolConfigErrorV1};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PgBouncerRuntimeConfigV1 {
    alias: PoolAliasV1,
    database_host: String,
    database_port: u16,
    database_name: String,
    max_client_connections: u16,
}

impl PgBouncerRuntimeConfigV1 {
    pub fn new(
        alias: PoolAliasV1,
        database_host: String,
        database_port: u16,
        database_name: String,
        runtime_principal: String,
        max_client_connections: u16,
    ) -> Result<Self, PoolConfigErrorV1> {
        if !valid_host(&database_host) || !valid_identifier(&database_name) {
            return Err(PoolConfigErrorV1::Endpoint);
        }
        if !valid_identifier(&runtime_principal) {
            return Err(PoolConfigErrorV1::Identifier);
        }
        if database_port == 0 || max_client_connections == 0 {
            return Err(PoolConfigErrorV1::ConnectionLimit);
        }
        Ok(Self {
            alias,
            database_host,
            database_port,
            database_name,
            max_client_connections,
        })
    }

    pub fn alias(&self) -> &PoolAliasV1 {
        &self.alias
    }

    pub fn render_database_entry(&self) -> String {
        format!(
            "{} = host={} port={} dbname={} pool_mode=transaction max_db_client_connections={}",
            self.alias.as_str(),
            self.database_host,
            self.database_port,
            self.database_name,
            self.max_client_connections,
        )
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn valid_host(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b':')
        })
}
