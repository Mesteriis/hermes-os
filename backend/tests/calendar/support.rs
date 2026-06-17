use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use hermes_hub_backend::platform::storage::Database;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

pub async fn live_pool() -> Option<PgPool> {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live calendar test: HERMES_TEST_DATABASE_URL is not set");
        return None;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    Some(database.pool().expect("configured pool").clone())
}

pub fn disconnected_pool() -> PgPool {
    PgPoolOptions::new()
        .connect_lazy("postgres://hermes:unused@127.0.0.1:1/hermes_hub")
        .expect("create lazy test pool")
}
