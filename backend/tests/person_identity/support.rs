use std::time::{SystemTime, UNIX_EPOCH};
use testkit::context::TestContext;

use chrono::Utc;
use hermes_hub_backend::domains::persons::api::PersonProjectionStore;
use hermes_hub_backend::domains::persons::identity::{
    PersonIdentityError, PersonIdentityReviewCommand, PersonIdentityReviewState,
    PersonIdentityStore,
};
use hermes_hub_backend::platform::events::{EventStore, NewEventEnvelope};
use hermes_hub_backend::platform::storage::Database;
use serde_json::json;
use sqlx::postgres::PgPool;

const CONTACT_IDENTITY_REVIEW_EVENT_TYPE: &str = "person_identity.review_state_changed";

pub(crate) struct PersonIdentityTestContext {
    pub(crate) pool: PgPool,
    pub(crate) store: PersonIdentityStore,
    pub(crate) event_store: EventStore,
    pub(crate) person_store: PersonProjectionStore,
}

pub(crate) async fn live_person_identity_context() -> Option<PersonIdentityTestContext> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    person_identity_context(&database_url).await
}

async fn person_identity_context(database_url: &str) -> Option<PersonIdentityTestContext> {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    Some(PersonIdentityTestContext {
        pool: pool.clone(),
        store: PersonIdentityStore::new(pool.clone()),
        event_store: EventStore::new(pool.clone()),
        person_store: PersonProjectionStore::new(pool.clone()),
    })
}

pub(crate) async fn seed_normalized_persons(
    context: &PersonIdentityTestContext,
    left_person_id: &str,
    right_person_id: &str,
    display_name: &str,
) -> Result<(), PersonIdentityError> {
    sqlx::query(
        r#"
        UPDATE persons
        SET display_name = $1
        WHERE person_id = $2 OR person_id = $3
        "#,
    )
    .bind(display_name)
    .bind(left_person_id)
    .bind(right_person_id)
    .execute(&context.pool)
    .await?;

    Ok(())
}

pub(crate) async fn confirm_identity_candidate(
    context: &PersonIdentityTestContext,
    identity_candidate_id: &str,
    command_id: &str,
) -> Result<(), PersonIdentityError> {
    context
        .store
        .set_review_state(&PersonIdentityReviewCommand {
            command_id: command_id.to_owned(),
            identity_candidate_id: identity_candidate_id.to_owned(),
            review_state: PersonIdentityReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await?;

    Ok(())
}

pub(crate) async fn exclude_persons_from_name_merge_refresh(
    context: &PersonIdentityTestContext,
    person_ids: &[&str],
    suffix: u128,
) -> Result<(), PersonIdentityError> {
    for (index, person_id) in person_ids.iter().enumerate() {
        sqlx::query(
            r#"
            UPDATE persons
            SET display_name = $1
            WHERE person_id = $2
            "#,
        )
        .bind(format!(
            "identity-refresh-skip-{suffix}-{index}@example.com"
        ))
        .bind(person_id)
        .execute(&context.pool)
        .await?;
    }

    Ok(())
}

pub(crate) async fn promote_identity_candidate(
    context: &PersonIdentityTestContext,
    identity_candidate_id: &str,
) -> Result<(), PersonIdentityError> {
    sqlx::query(
        r#"
        UPDATE person_identity_candidates
        SET updated_at = clock_timestamp()
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(identity_candidate_id)
    .execute(&context.pool)
    .await?;

    Ok(())
}

pub(crate) async fn age_identity_candidate(
    context: &PersonIdentityTestContext,
    identity_candidate_id: &str,
) -> Result<(), PersonIdentityError> {
    sqlx::query(
        r#"
        UPDATE person_identity_candidates
        SET updated_at = clock_timestamp() - INTERVAL '1 hour'
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(identity_candidate_id)
    .execute(&context.pool)
    .await?;

    Ok(())
}

pub(crate) async fn assert_identity_candidate_exists(
    context: &PersonIdentityTestContext,
    identity_candidate_id: &str,
) -> Result<(), PersonIdentityError> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM person_identity_candidates
            WHERE identity_candidate_id = $1
        )
        "#,
    )
    .bind(identity_candidate_id)
    .fetch_one(&context.pool)
    .await?
    .then_some(())
    .ok_or(PersonIdentityError::IdentityCandidateNotFound)
}

pub(crate) async fn identity_candidate_updated_at(
    context: &PersonIdentityTestContext,
    identity_candidate_id: &str,
) -> Result<chrono::DateTime<Utc>, PersonIdentityError> {
    let updated_at = sqlx::query_scalar(
        r#"
        SELECT updated_at
        FROM person_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(identity_candidate_id)
    .fetch_one(&context.pool)
    .await?;

    Ok(updated_at)
}

pub(crate) fn identity_candidate_id_from_persons(left_id: &str, right_id: &str) -> String {
    let (left_person_id, right_person_id) = ordered_person_ids(left_id, right_id);
    format!("identity_candidate:v1:merge_persons:{left_person_id}:{right_person_id}")
}

pub(crate) fn split_identity_candidate_id_from_persons(left_id: &str, right_id: &str) -> String {
    let (left_person_id, right_person_id) = ordered_person_ids(left_id, right_id);
    format!("identity_candidate:v1:split_person:{left_person_id}:{right_person_id}")
}

pub(crate) fn ordered_person_ids(left_id: &str, right_id: &str) -> (String, String) {
    if left_id <= right_id {
        (left_id.to_owned(), right_id.to_owned())
    } else {
        (right_id.to_owned(), left_id.to_owned())
    }
}

pub(crate) fn build_review_event(
    identity_candidate_id: &str,
    review_state: PersonIdentityReviewState,
    actor_id: &str,
    command_id: &str,
) -> NewEventEnvelope {
    NewEventEnvelope::builder(
        format!("person_identity_review:{command_id}"),
        CONTACT_IDENTITY_REVIEW_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": "person_identity_review",
            "provider": "local_api",
            "source_id": command_id,
        }),
        json!({"kind": "person_identity_review"}),
    )
    .actor(json!({"actor_id": actor_id}))
    .payload(json!({
        "identity_candidate_id": identity_candidate_id,
        "review_state": review_state.as_str(),
    }))
    .build()
    .expect("review event")
}

pub(crate) fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
