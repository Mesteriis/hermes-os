use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;

use hermes_hub_backend::event_log::{EventStore, NewEventEnvelope};
use hermes_hub_backend::person_identity::{
    PersonIdentityError, PersonIdentityReviewCommand, PersonIdentityReviewState,
    PersonIdentityStore,
};
use hermes_hub_backend::persons::PersonProjectionStore;
use hermes_hub_backend::storage::Database;

const CONTACT_IDENTITY_REVIEW_EVENT_TYPE: &str = "person_identity.review_state_changed";

#[tokio::test]
async fn person_identity_refresh_creates_conservative_merge_candidate_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live person identity test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = person_identity_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();
    let shared_name = format!("Alex Meridian {suffix}");

    let left = context
        .person_store
        .upsert_email_person(&format!("alex.left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = context
        .person_store
        .upsert_email_person(&format!("alex.right-{suffix}@example.com"))
        .await
        .expect("upsert right person");

    seed_normalized_persons(&context, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let created = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh candidates");
    assert!(created >= 1);

    let (left_id, right_id) = ordered_person_ids(&left.person_id, &right.person_id);
    let candidate_id = format!("identity_candidate:v1:merge_persons:{left_id}:{right_id}");
    let row: (String, String, String) = sqlx::query_as(
        r#"
        SELECT identity_candidate_id, candidate_kind, review_state
        FROM person_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(&candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("candidate row");
    assert_eq!(row.0, candidate_id);
    assert_eq!(row.1, "merge_persons");
    assert_eq!(row.2, "suggested");
}

#[tokio::test]
async fn person_identity_confirm_records_review_without_mutating_persons_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live person identity test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = person_identity_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();
    let shared_name = format!("Jordan Candidate {suffix}");

    let left = context
        .person_store
        .upsert_email_person(&format!("jordan.left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = context
        .person_store
        .upsert_email_person(&format!("jordan.right-{suffix}@example.com"))
        .await
        .expect("upsert right person");

    seed_normalized_persons(&context, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh");

    let identity_candidate_id =
        identity_candidate_id_from_persons(&left.person_id, &right.person_id);
    let command = PersonIdentityReviewCommand {
        command_id: format!("identity-confirm-{suffix}"),
        identity_candidate_id: identity_candidate_id.clone(),
        review_state: PersonIdentityReviewState::UserConfirmed,
        actor_id: "tests-reviewer".to_owned(),
    };

    let result = context
        .store
        .set_review_state(&command)
        .await
        .expect("confirm identity candidate");
    assert_eq!(
        result.review_state,
        PersonIdentityReviewState::UserConfirmed
    );

    let state: String = sqlx::query_scalar(
        "SELECT review_state FROM person_identity_candidates WHERE identity_candidate_id = $1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("load state");
    assert_eq!(state, "user_confirmed");

    let persons =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM persons WHERE person_id IN ($1, $2)")
            .bind(&left.person_id)
            .bind(&right.person_id)
            .fetch_one(&context.pool)
            .await
            .expect("persons remain");
    assert_eq!(persons, 2);
}

#[tokio::test]
async fn person_identity_confirm_materializes_split_candidate_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live person identity test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = person_identity_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();
    let shared_name = format!("Morgan Split Candidate {suffix}");

    let left = context
        .person_store
        .upsert_email_person(&format!("morgan.left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = context
        .person_store
        .upsert_email_person(&format!("morgan.right-{suffix}@example.com"))
        .await
        .expect("upsert right person");

    seed_normalized_persons(&context, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh merge candidates");
    let merge_candidate_id = identity_candidate_id_from_persons(&left.person_id, &right.person_id);

    let _ = context
        .store
        .set_review_state(&PersonIdentityReviewCommand {
            command_id: format!("identity-confirm-for-split-{suffix}"),
            identity_candidate_id: merge_candidate_id,
            review_state: PersonIdentityReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm merge candidate");

    let split_candidate_id =
        split_identity_candidate_id_from_persons(&left.person_id, &right.person_id);
    let row: (String, String, String, f64) = sqlx::query_as(
        r#"
        SELECT candidate_kind, review_state, evidence_summary, confidence
        FROM person_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(&split_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("split candidate row");

    assert_eq!(row.0, "split_person");
    assert_eq!(row.1, "suggested");
    assert!(
        row.2
            .starts_with("Previously confirmed merge can be split:")
    );
    assert_eq!(row.3, 1.0);
}

#[tokio::test]
async fn person_identity_confirmed_split_removes_merge_from_detail_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live person identity test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = person_identity_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();
    let shared_name = format!("Taylor Split Detail {suffix}");

    let left = context
        .person_store
        .upsert_email_person(&format!("taylor.left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = context
        .person_store
        .upsert_email_person(&format!("taylor.right-{suffix}@example.com"))
        .await
        .expect("upsert right person");

    seed_normalized_persons(&context, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh merge candidates");
    let merge_candidate_id = identity_candidate_id_from_persons(&left.person_id, &right.person_id);

    let _ = context
        .store
        .set_review_state(&PersonIdentityReviewCommand {
            command_id: format!("identity-confirm-detail-{suffix}"),
            identity_candidate_id: merge_candidate_id.clone(),
            review_state: PersonIdentityReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm merge candidate");

    let detail = context
        .store
        .person_identity(&left.person_id)
        .await
        .expect("person identity detail");
    assert!(
        detail
            .items
            .iter()
            .any(|item| item.identity_candidate_id == merge_candidate_id)
    );

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh split candidates");
    let split_candidate_id =
        split_identity_candidate_id_from_persons(&left.person_id, &right.person_id);

    let _ = context
        .store
        .set_review_state(&PersonIdentityReviewCommand {
            command_id: format!("identity-split-detail-{suffix}"),
            identity_candidate_id: split_candidate_id,
            review_state: PersonIdentityReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm split candidate");

    let detail = context
        .store
        .person_identity(&left.person_id)
        .await
        .expect("person identity detail after split");
    assert!(!detail.items.iter().any(|item| {
        item.candidate_kind == "merge_persons" && item.identity_candidate_id == merge_candidate_id
    }));
}

#[tokio::test]
async fn person_identity_refresh_skips_existing_split_when_generating_next_split_against_postgres()
{
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live person identity test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = person_identity_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();

    let first_left = context
        .person_store
        .upsert_email_person(&format!("first.left-{suffix}@example.com"))
        .await
        .expect("upsert first left person");
    let first_right = context
        .person_store
        .upsert_email_person(&format!("first.right-{suffix}@example.com"))
        .await
        .expect("upsert first right person");
    let second_left = context
        .person_store
        .upsert_email_person(&format!("second.left-{suffix}@example.com"))
        .await
        .expect("upsert second left person");
    let second_right = context
        .person_store
        .upsert_email_person(&format!("second.right-{suffix}@example.com"))
        .await
        .expect("upsert second right person");

    seed_normalized_persons(
        &context,
        &first_left.person_id,
        &first_right.person_id,
        &format!("First Split Existing {suffix}"),
    )
    .await
    .expect("seed first pair display names");
    seed_normalized_persons(
        &context,
        &second_left.person_id,
        &second_right.person_id,
        &format!("Second Split Pending {suffix}"),
    )
    .await
    .expect("seed second pair display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh merge candidates");

    let first_merge_candidate_id =
        identity_candidate_id_from_persons(&first_left.person_id, &first_right.person_id);
    let second_merge_candidate_id =
        identity_candidate_id_from_persons(&second_left.person_id, &second_right.person_id);
    let first_split_candidate_id =
        split_identity_candidate_id_from_persons(&first_left.person_id, &first_right.person_id);
    let second_split_candidate_id =
        split_identity_candidate_id_from_persons(&second_left.person_id, &second_right.person_id);

    confirm_identity_candidate(
        &context,
        &first_merge_candidate_id,
        &format!("identity-confirm-first-split-skip-{suffix}"),
    )
    .await
    .expect("confirm first merge candidate");
    confirm_identity_candidate(
        &context,
        &second_merge_candidate_id,
        &format!("identity-confirm-second-split-skip-{suffix}"),
    )
    .await
    .expect("confirm second merge candidate");

    exclude_persons_from_name_merge_refresh(
        &context,
        &[
            &first_left.person_id,
            &first_right.person_id,
            &second_left.person_id,
            &second_right.person_id,
        ],
        suffix,
    )
    .await
    .expect("exclude persons from merge refresh");

    promote_identity_candidate(&context, &first_merge_candidate_id)
        .await
        .expect("promote first merge candidate");
    let _ = context
        .store
        .refresh_candidates(1)
        .await
        .expect("create first split candidate");
    assert_identity_candidate_exists(&context, &first_split_candidate_id)
        .await
        .expect("first split candidate exists");

    age_identity_candidate(&context, &first_split_candidate_id)
        .await
        .expect("age first split candidate");
    let first_split_updated_at_before =
        identity_candidate_updated_at(&context, &first_split_candidate_id)
            .await
            .expect("first split updated_at before second refresh");

    promote_identity_candidate(&context, &second_merge_candidate_id)
        .await
        .expect("promote second merge candidate");
    promote_identity_candidate(&context, &first_merge_candidate_id)
        .await
        .expect("promote first merge candidate above second");

    let _ = context
        .store
        .refresh_candidates(1)
        .await
        .expect("create second split candidate");

    assert_identity_candidate_exists(&context, &second_split_candidate_id)
        .await
        .expect("second split candidate exists");
    let first_split_updated_at_after =
        identity_candidate_updated_at(&context, &first_split_candidate_id)
            .await
            .expect("first split updated_at after second refresh");
    assert_eq!(first_split_updated_at_after, first_split_updated_at_before);
}

#[tokio::test]
async fn person_identity_reject_suppresses_candidate_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live person identity test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = person_identity_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();
    let shared_name = format!("Sam Candidate {suffix}");

    let left = context
        .person_store
        .upsert_email_person(&format!("sam.left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = context
        .person_store
        .upsert_email_person(&format!("sam.right-{suffix}@example.com"))
        .await
        .expect("upsert right person");

    seed_normalized_persons(&context, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh");
    let identity_candidate_id =
        identity_candidate_id_from_persons(&left.person_id, &right.person_id);

    let _ = context
        .store
        .set_review_state(&PersonIdentityReviewCommand {
            command_id: format!("identity-reject-{suffix}"),
            identity_candidate_id: identity_candidate_id.clone(),
            review_state: PersonIdentityReviewState::UserRejected,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("reject candidate");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh again");

    let state: String = sqlx::query_scalar(
        "SELECT review_state FROM person_identity_candidates WHERE identity_candidate_id = $1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("load state");
    assert_eq!(state, "user_rejected");
}

#[tokio::test]
async fn person_identity_review_event_rebuilds_state_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live person identity test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = person_identity_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();
    let shared_name = format!("Pat Candidate {suffix}");

    let left = context
        .person_store
        .upsert_email_person(&format!("pat.left-{suffix}@example.com"))
        .await
        .expect("upsert left person");
    let right = context
        .person_store
        .upsert_email_person(&format!("pat.right-{suffix}@example.com"))
        .await
        .expect("upsert right person");

    seed_normalized_persons(&context, &left.person_id, &right.person_id, &shared_name)
        .await
        .expect("seed display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh");
    let identity_candidate_id =
        identity_candidate_id_from_persons(&left.person_id, &right.person_id);

    let confirm_event = build_review_event(
        &identity_candidate_id,
        PersonIdentityReviewState::UserConfirmed,
        "event-reviewer",
        &format!("identity-event-confirm-{suffix}"),
    );
    let reject_event = build_review_event(
        &identity_candidate_id,
        PersonIdentityReviewState::UserRejected,
        "event-reviewer",
        &format!("identity-event-reject-{suffix}"),
    );

    context
        .event_store
        .append(&confirm_event)
        .await
        .expect("append confirm event");
    context
        .event_store
        .append(&reject_event)
        .await
        .expect("append reject event");

    let confirm_event = context
        .event_store
        .get_by_id(&confirm_event.event_id)
        .await
        .expect("load confirm event")
        .expect("confirm event exists");
    context
        .store
        .apply_review_event(&confirm_event)
        .await
        .expect("apply confirm event");

    let reject_event = context
        .event_store
        .get_by_id(&reject_event.event_id)
        .await
        .expect("load reject event")
        .expect("reject event exists");
    context
        .store
        .apply_review_event(&reject_event)
        .await
        .expect("apply reject event");

    let state: String = sqlx::query_scalar(
        "SELECT review_state FROM person_identity_candidates WHERE identity_candidate_id = $1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("load state");
    assert_eq!(state, "user_rejected");

    let event_id: String = sqlx::query_scalar(
        "SELECT event_id FROM person_identity_candidates WHERE identity_candidate_id = $1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("load event id");
    assert_eq!(
        event_id,
        format!("person_identity_review:identity-event-reject-{suffix}")
    );
}

struct PersonIdentityTestContext {
    pool: PgPool,
    store: PersonIdentityStore,
    event_store: EventStore,
    person_store: PersonProjectionStore,
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

async fn seed_normalized_persons(
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

async fn confirm_identity_candidate(
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

async fn exclude_persons_from_name_merge_refresh(
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

async fn promote_identity_candidate(
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

async fn age_identity_candidate(
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

async fn assert_identity_candidate_exists(
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

async fn identity_candidate_updated_at(
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

fn identity_candidate_id_from_persons(left_id: &str, right_id: &str) -> String {
    let (left_person_id, right_person_id) = ordered_person_ids(left_id, right_id);
    format!("identity_candidate:v1:merge_persons:{left_person_id}:{right_person_id}")
}

fn split_identity_candidate_id_from_persons(left_id: &str, right_id: &str) -> String {
    let (left_person_id, right_person_id) = ordered_person_ids(left_id, right_id);
    format!("identity_candidate:v1:split_person:{left_person_id}:{right_person_id}")
}

fn ordered_person_ids(left_id: &str, right_id: &str) -> (String, String) {
    if left_id <= right_id {
        (left_id.to_owned(), right_id.to_owned())
    } else {
        (right_id.to_owned(), left_id.to_owned())
    }
}

fn build_review_event(
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

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
