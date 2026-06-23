#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use hermes_hub_backend::domains::persons::api::PersonProjectionStore;
use hermes_hub_backend::platform::events::{EventConsumerConfig, EventConsumerRunner};
use hermes_hub_backend::platform::storage::Database;
use hermes_hub_backend::workflows::person_derived_evidence::{
    PERSON_DERIVED_EVIDENCE_CONSUMER, project_person_derived_evidence_event,
};
use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn live_persons_pool(_test_name: &str) -> Option<PgPool> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

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

pub async fn run_person_derived_evidence_consumer(pool: PgPool) {
    let runner = EventConsumerRunner::new(
        pool.clone(),
        EventConsumerConfig::new(PERSON_DERIVED_EVIDENCE_CONSUMER),
    );
    runner
        .process_next_batch(|event| project_person_derived_evidence_event(pool.clone(), event))
        .await
        .expect("person derived evidence consumer");
}
