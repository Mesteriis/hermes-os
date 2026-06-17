#![allow(dead_code)]

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use hermes_hub_backend::domains::persons::api::PersonProjectionStore;
use hermes_hub_backend::platform::storage::Database;
use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn live_persons_pool(test_name: &str) -> Option<PgPool> {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live {test_name} test: HERMES_TEST_DATABASE_URL is not set");
        return None;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    Some(database.pool().expect("configured pool").clone())
}

pub async fn live_persons_store(test_name: &str) -> Option<PersonProjectionStore> {
    let pool = live_persons_pool(test_name).await?;
    Some(PersonProjectionStore::new(pool))
}

pub fn disconnected_persons_store() -> PersonProjectionStore {
    let pool = PgPoolOptions::new()
        .connect_lazy("postgres://hermes:unused@127.0.0.1:1/hermes_hub")
        .expect("create lazy test pool");
    PersonProjectionStore::new(pool)
}

pub fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
