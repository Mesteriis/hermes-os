use hermes_hub_backend::domains::persons::identity::{
    PersonIdentityReviewCommand, PersonIdentityReviewState,
};

use super::support::{
    build_review_event, identity_candidate_id_from_persons, live_person_identity_context,
    seed_normalized_persons, unique_suffix,
};

#[tokio::test]
async fn person_identity_reject_suppresses_candidate_against_postgres() {
    let Some(context) = live_person_identity_context().await else {
        return;
    };
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
    let Some(context) = live_person_identity_context().await else {
        return;
    };
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
