use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::json;
use sqlx::postgres::PgPool;

use hermes_hub_backend::contact_identity::{
    ContactIdentityError, ContactIdentityReviewCommand, ContactIdentityReviewState,
    ContactIdentityStore,
};
use hermes_hub_backend::contacts::ContactProjectionStore;
use hermes_hub_backend::event_log::{EventStore, NewEventEnvelope};
use hermes_hub_backend::storage::Database;

const CONTACT_IDENTITY_REVIEW_EVENT_TYPE: &str = "contact_identity.review_state_changed";

#[tokio::test]
async fn contact_identity_refresh_creates_conservative_merge_candidate_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live contact identity test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = contact_identity_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();
    let shared_name = format!("Alex Meridian {suffix}");

    let left = context
        .contact_store
        .upsert_email_contact(&format!("alex.left-{suffix}@example.com"))
        .await
        .expect("upsert left contact");
    let right = context
        .contact_store
        .upsert_email_contact(&format!("alex.right-{suffix}@example.com"))
        .await
        .expect("upsert right contact");

    seed_normalized_contacts(&context, &left.contact_id, &right.contact_id, &shared_name)
        .await
        .expect("seed display names");

    let created = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh candidates");
    assert!(created >= 1);

    let (left_id, right_id) = ordered_contact_ids(&left.contact_id, &right.contact_id);
    let candidate_id = format!("identity_candidate:v1:merge_contacts:{left_id}:{right_id}");
    let row: (String, String, String) = sqlx::query_as(
        r#"
        SELECT identity_candidate_id, candidate_kind, review_state
        FROM contact_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(&candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("candidate row");
    assert_eq!(row.0, candidate_id);
    assert_eq!(row.1, "merge_contacts");
    assert_eq!(row.2, "suggested");
}

#[tokio::test]
async fn contact_identity_confirm_records_review_without_mutating_contacts_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live contact identity test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = contact_identity_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();
    let shared_name = format!("Jordan Candidate {suffix}");

    let left = context
        .contact_store
        .upsert_email_contact(&format!("jordan.left-{suffix}@example.com"))
        .await
        .expect("upsert left contact");
    let right = context
        .contact_store
        .upsert_email_contact(&format!("jordan.right-{suffix}@example.com"))
        .await
        .expect("upsert right contact");

    seed_normalized_contacts(&context, &left.contact_id, &right.contact_id, &shared_name)
        .await
        .expect("seed display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh");

    let identity_candidate_id =
        identity_candidate_id_from_contacts(&left.contact_id, &right.contact_id);
    let command = ContactIdentityReviewCommand {
        command_id: format!("identity-confirm-{suffix}"),
        identity_candidate_id: identity_candidate_id.clone(),
        review_state: ContactIdentityReviewState::UserConfirmed,
        actor_id: "tests-reviewer".to_owned(),
    };

    let result = context
        .store
        .set_review_state(&command)
        .await
        .expect("confirm identity candidate");
    assert_eq!(
        result.review_state,
        ContactIdentityReviewState::UserConfirmed
    );

    let state: String = sqlx::query_scalar(
        "SELECT review_state FROM contact_identity_candidates WHERE identity_candidate_id = $1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("load state");
    assert_eq!(state, "user_confirmed");

    let contacts =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM contacts WHERE contact_id IN ($1, $2)")
            .bind(&left.contact_id)
            .bind(&right.contact_id)
            .fetch_one(&context.pool)
            .await
            .expect("contacts remain");
    assert_eq!(contacts, 2);
}

#[tokio::test]
async fn contact_identity_refresh_creates_split_candidate_for_confirmed_merge_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live contact identity test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = contact_identity_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();
    let shared_name = format!("Morgan Split Candidate {suffix}");

    let left = context
        .contact_store
        .upsert_email_contact(&format!("morgan.left-{suffix}@example.com"))
        .await
        .expect("upsert left contact");
    let right = context
        .contact_store
        .upsert_email_contact(&format!("morgan.right-{suffix}@example.com"))
        .await
        .expect("upsert right contact");

    seed_normalized_contacts(&context, &left.contact_id, &right.contact_id, &shared_name)
        .await
        .expect("seed display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh merge candidates");
    let merge_candidate_id =
        identity_candidate_id_from_contacts(&left.contact_id, &right.contact_id);

    let _ = context
        .store
        .set_review_state(&ContactIdentityReviewCommand {
            command_id: format!("identity-confirm-for-split-{suffix}"),
            identity_candidate_id: merge_candidate_id,
            review_state: ContactIdentityReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm merge candidate");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh split candidates");

    let split_candidate_id =
        split_identity_candidate_id_from_contacts(&left.contact_id, &right.contact_id);
    let row: (String, String, String, f64) = sqlx::query_as(
        r#"
        SELECT candidate_kind, review_state, evidence_summary, confidence
        FROM contact_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(&split_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("split candidate row");

    assert_eq!(row.0, "split_contact");
    assert_eq!(row.1, "suggested");
    assert!(
        row.2
            .starts_with("Previously confirmed merge can be split:")
    );
    assert_eq!(row.3, 1.0);
}

#[tokio::test]
async fn contact_identity_confirmed_split_removes_merge_from_detail_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live contact identity test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = contact_identity_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();
    let shared_name = format!("Taylor Split Detail {suffix}");

    let left = context
        .contact_store
        .upsert_email_contact(&format!("taylor.left-{suffix}@example.com"))
        .await
        .expect("upsert left contact");
    let right = context
        .contact_store
        .upsert_email_contact(&format!("taylor.right-{suffix}@example.com"))
        .await
        .expect("upsert right contact");

    seed_normalized_contacts(&context, &left.contact_id, &right.contact_id, &shared_name)
        .await
        .expect("seed display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh merge candidates");
    let merge_candidate_id =
        identity_candidate_id_from_contacts(&left.contact_id, &right.contact_id);

    let _ = context
        .store
        .set_review_state(&ContactIdentityReviewCommand {
            command_id: format!("identity-confirm-detail-{suffix}"),
            identity_candidate_id: merge_candidate_id.clone(),
            review_state: ContactIdentityReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm merge candidate");

    let detail = context
        .store
        .contact_identity(&left.contact_id)
        .await
        .expect("contact identity detail");
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
        split_identity_candidate_id_from_contacts(&left.contact_id, &right.contact_id);

    let _ = context
        .store
        .set_review_state(&ContactIdentityReviewCommand {
            command_id: format!("identity-split-detail-{suffix}"),
            identity_candidate_id: split_candidate_id,
            review_state: ContactIdentityReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await
        .expect("confirm split candidate");

    let detail = context
        .store
        .contact_identity(&left.contact_id)
        .await
        .expect("contact identity detail after split");
    assert!(!detail.items.iter().any(|item| {
        item.candidate_kind == "merge_contacts" && item.identity_candidate_id == merge_candidate_id
    }));
}

#[tokio::test]
async fn contact_identity_refresh_skips_existing_split_when_generating_next_split_against_postgres()
{
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live contact identity test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = contact_identity_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();

    let first_left = context
        .contact_store
        .upsert_email_contact(&format!("first.left-{suffix}@example.com"))
        .await
        .expect("upsert first left contact");
    let first_right = context
        .contact_store
        .upsert_email_contact(&format!("first.right-{suffix}@example.com"))
        .await
        .expect("upsert first right contact");
    let second_left = context
        .contact_store
        .upsert_email_contact(&format!("second.left-{suffix}@example.com"))
        .await
        .expect("upsert second left contact");
    let second_right = context
        .contact_store
        .upsert_email_contact(&format!("second.right-{suffix}@example.com"))
        .await
        .expect("upsert second right contact");

    seed_normalized_contacts(
        &context,
        &first_left.contact_id,
        &first_right.contact_id,
        &format!("First Split Existing {suffix}"),
    )
    .await
    .expect("seed first pair display names");
    seed_normalized_contacts(
        &context,
        &second_left.contact_id,
        &second_right.contact_id,
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
        identity_candidate_id_from_contacts(&first_left.contact_id, &first_right.contact_id);
    let second_merge_candidate_id =
        identity_candidate_id_from_contacts(&second_left.contact_id, &second_right.contact_id);
    let first_split_candidate_id =
        split_identity_candidate_id_from_contacts(&first_left.contact_id, &first_right.contact_id);
    let second_split_candidate_id = split_identity_candidate_id_from_contacts(
        &second_left.contact_id,
        &second_right.contact_id,
    );

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

    exclude_contacts_from_name_merge_refresh(
        &context,
        &[
            &first_left.contact_id,
            &first_right.contact_id,
            &second_left.contact_id,
            &second_right.contact_id,
        ],
        suffix,
    )
    .await
    .expect("exclude contacts from merge refresh");

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
async fn contact_identity_reject_suppresses_candidate_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live contact identity test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = contact_identity_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();
    let shared_name = format!("Sam Candidate {suffix}");

    let left = context
        .contact_store
        .upsert_email_contact(&format!("sam.left-{suffix}@example.com"))
        .await
        .expect("upsert left contact");
    let right = context
        .contact_store
        .upsert_email_contact(&format!("sam.right-{suffix}@example.com"))
        .await
        .expect("upsert right contact");

    seed_normalized_contacts(&context, &left.contact_id, &right.contact_id, &shared_name)
        .await
        .expect("seed display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh");
    let identity_candidate_id =
        identity_candidate_id_from_contacts(&left.contact_id, &right.contact_id);

    let _ = context
        .store
        .set_review_state(&ContactIdentityReviewCommand {
            command_id: format!("identity-reject-{suffix}"),
            identity_candidate_id: identity_candidate_id.clone(),
            review_state: ContactIdentityReviewState::UserRejected,
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
        "SELECT review_state FROM contact_identity_candidates WHERE identity_candidate_id = $1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("load state");
    assert_eq!(state, "user_rejected");
}

#[tokio::test]
async fn contact_identity_review_event_rebuilds_state_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live contact identity test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let context = contact_identity_context(&database_url)
        .await
        .expect("context");
    let suffix = unique_suffix();
    let shared_name = format!("Pat Candidate {suffix}");

    let left = context
        .contact_store
        .upsert_email_contact(&format!("pat.left-{suffix}@example.com"))
        .await
        .expect("upsert left contact");
    let right = context
        .contact_store
        .upsert_email_contact(&format!("pat.right-{suffix}@example.com"))
        .await
        .expect("upsert right contact");

    seed_normalized_contacts(&context, &left.contact_id, &right.contact_id, &shared_name)
        .await
        .expect("seed display names");

    let _ = context
        .store
        .refresh_candidates(100)
        .await
        .expect("refresh");
    let identity_candidate_id =
        identity_candidate_id_from_contacts(&left.contact_id, &right.contact_id);

    let confirm_event = build_review_event(
        &identity_candidate_id,
        ContactIdentityReviewState::UserConfirmed,
        "event-reviewer",
        &format!("identity-event-confirm-{suffix}"),
    );
    let reject_event = build_review_event(
        &identity_candidate_id,
        ContactIdentityReviewState::UserRejected,
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
        "SELECT review_state FROM contact_identity_candidates WHERE identity_candidate_id = $1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("load state");
    assert_eq!(state, "user_rejected");

    let event_id: String = sqlx::query_scalar(
        "SELECT event_id FROM contact_identity_candidates WHERE identity_candidate_id = $1",
    )
    .bind(&identity_candidate_id)
    .fetch_one(&context.pool)
    .await
    .expect("load event id");
    assert_eq!(
        event_id,
        format!("contact_identity_review:identity-event-reject-{suffix}")
    );
}

struct ContactIdentityTestContext {
    pool: PgPool,
    store: ContactIdentityStore,
    event_store: EventStore,
    contact_store: ContactProjectionStore,
}

async fn contact_identity_context(database_url: &str) -> Option<ContactIdentityTestContext> {
    let database = Database::connect(Some(database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    Some(ContactIdentityTestContext {
        pool: pool.clone(),
        store: ContactIdentityStore::new(pool.clone()),
        event_store: EventStore::new(pool.clone()),
        contact_store: ContactProjectionStore::new(pool.clone()),
    })
}

async fn seed_normalized_contacts(
    context: &ContactIdentityTestContext,
    left_contact_id: &str,
    right_contact_id: &str,
    display_name: &str,
) -> Result<(), ContactIdentityError> {
    sqlx::query(
        r#"
        UPDATE contacts
        SET display_name = $1
        WHERE contact_id = $2 OR contact_id = $3
        "#,
    )
    .bind(display_name)
    .bind(left_contact_id)
    .bind(right_contact_id)
    .execute(&context.pool)
    .await?;

    Ok(())
}

async fn confirm_identity_candidate(
    context: &ContactIdentityTestContext,
    identity_candidate_id: &str,
    command_id: &str,
) -> Result<(), ContactIdentityError> {
    context
        .store
        .set_review_state(&ContactIdentityReviewCommand {
            command_id: command_id.to_owned(),
            identity_candidate_id: identity_candidate_id.to_owned(),
            review_state: ContactIdentityReviewState::UserConfirmed,
            actor_id: "tests-reviewer".to_owned(),
        })
        .await?;

    Ok(())
}

async fn exclude_contacts_from_name_merge_refresh(
    context: &ContactIdentityTestContext,
    contact_ids: &[&str],
    suffix: u128,
) -> Result<(), ContactIdentityError> {
    for (index, contact_id) in contact_ids.iter().enumerate() {
        sqlx::query(
            r#"
            UPDATE contacts
            SET display_name = $1
            WHERE contact_id = $2
            "#,
        )
        .bind(format!(
            "identity-refresh-skip-{suffix}-{index}@example.com"
        ))
        .bind(contact_id)
        .execute(&context.pool)
        .await?;
    }

    Ok(())
}

async fn promote_identity_candidate(
    context: &ContactIdentityTestContext,
    identity_candidate_id: &str,
) -> Result<(), ContactIdentityError> {
    sqlx::query(
        r#"
        UPDATE contact_identity_candidates
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
    context: &ContactIdentityTestContext,
    identity_candidate_id: &str,
) -> Result<(), ContactIdentityError> {
    sqlx::query(
        r#"
        UPDATE contact_identity_candidates
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
    context: &ContactIdentityTestContext,
    identity_candidate_id: &str,
) -> Result<(), ContactIdentityError> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM contact_identity_candidates
            WHERE identity_candidate_id = $1
        )
        "#,
    )
    .bind(identity_candidate_id)
    .fetch_one(&context.pool)
    .await?
    .then_some(())
    .ok_or(ContactIdentityError::IdentityCandidateNotFound)
}

async fn identity_candidate_updated_at(
    context: &ContactIdentityTestContext,
    identity_candidate_id: &str,
) -> Result<chrono::DateTime<Utc>, ContactIdentityError> {
    let updated_at = sqlx::query_scalar(
        r#"
        SELECT updated_at
        FROM contact_identity_candidates
        WHERE identity_candidate_id = $1
        "#,
    )
    .bind(identity_candidate_id)
    .fetch_one(&context.pool)
    .await?;

    Ok(updated_at)
}

fn identity_candidate_id_from_contacts(left_id: &str, right_id: &str) -> String {
    let (left_contact_id, right_contact_id) = ordered_contact_ids(left_id, right_id);
    format!("identity_candidate:v1:merge_contacts:{left_contact_id}:{right_contact_id}")
}

fn split_identity_candidate_id_from_contacts(left_id: &str, right_id: &str) -> String {
    let (left_contact_id, right_contact_id) = ordered_contact_ids(left_id, right_id);
    format!("identity_candidate:v1:split_contact:{left_contact_id}:{right_contact_id}")
}

fn ordered_contact_ids(left_id: &str, right_id: &str) -> (String, String) {
    if left_id <= right_id {
        (left_id.to_owned(), right_id.to_owned())
    } else {
        (right_id.to_owned(), left_id.to_owned())
    }
}

fn build_review_event(
    identity_candidate_id: &str,
    review_state: ContactIdentityReviewState,
    actor_id: &str,
    command_id: &str,
) -> NewEventEnvelope {
    NewEventEnvelope::builder(
        format!("contact_identity_review:{command_id}"),
        CONTACT_IDENTITY_REVIEW_EVENT_TYPE,
        Utc::now(),
        json!({
            "kind": "contact_identity_review",
            "provider": "local_api",
            "source_id": command_id,
        }),
        json!({"kind": "contact_identity_review"}),
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
