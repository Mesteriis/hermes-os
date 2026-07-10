use super::support::{
    age_identity_candidate, assert_identity_candidate_exists, confirm_identity_candidate,
    exclude_personas_from_name_merge_refresh, identity_candidate_id_from_personas,
    identity_candidate_updated_at, live_persona_identity_context, promote_identity_candidate,
    seed_normalized_personas, split_identity_candidate_id_from_personas, unique_suffix,
};

#[tokio::test]
async fn persona_identity_refresh_skips_existing_split_when_generating_next_split_against_postgres()
{
    let Some(context) = live_persona_identity_context().await else {
        return;
    };
    let suffix = unique_suffix();

    let first_left = context
        .persona_store
        .upsert_email_persona(&format!("first.left-{suffix}@example.com"))
        .await
        .expect("upsert first left person");
    let first_right = context
        .persona_store
        .upsert_email_persona(&format!("first.right-{suffix}@example.com"))
        .await
        .expect("upsert first right person");
    let second_left = context
        .persona_store
        .upsert_email_persona(&format!("second.left-{suffix}@example.com"))
        .await
        .expect("upsert second left person");
    let second_right = context
        .persona_store
        .upsert_email_persona(&format!("second.right-{suffix}@example.com"))
        .await
        .expect("upsert second right person");

    seed_normalized_personas(
        &context,
        &first_left.persona_id,
        &first_right.persona_id,
        &format!("First Split Existing {suffix}"),
    )
    .await
    .expect("seed first pair display names");
    seed_normalized_personas(
        &context,
        &second_left.persona_id,
        &second_right.persona_id,
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
        identity_candidate_id_from_personas(&first_left.persona_id, &first_right.persona_id);
    let second_merge_candidate_id =
        identity_candidate_id_from_personas(&second_left.persona_id, &second_right.persona_id);
    let first_split_candidate_id =
        split_identity_candidate_id_from_personas(&first_left.persona_id, &first_right.persona_id);
    let second_split_candidate_id = split_identity_candidate_id_from_personas(
        &second_left.persona_id,
        &second_right.persona_id,
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

    exclude_personas_from_name_merge_refresh(
        &context,
        &[
            &first_left.persona_id,
            &first_right.persona_id,
            &second_left.persona_id,
            &second_right.persona_id,
        ],
        suffix,
    )
    .await
    .expect("exclude personas from merge refresh");

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
