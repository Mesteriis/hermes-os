use hermes_hub_backend::domains::persons::identity::{
    PersonIdentityReviewCommand, PersonIdentityReviewState,
};

use super::support::{
    identity_candidate_id_from_persons, live_person_identity_context, ordered_person_ids,
    seed_normalized_persons, split_identity_candidate_id_from_persons, unique_suffix,
};

#[tokio::test]
async fn person_identity_refresh_creates_conservative_merge_candidate_against_postgres() {
    let Some(context) = live_person_identity_context().await else {
        return;
    };
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
    let Some(context) = live_person_identity_context().await else {
        return;
    };
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
    let Some(context) = live_person_identity_context().await else {
        return;
    };
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
    let Some(context) = live_person_identity_context().await else {
        return;
    };
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
