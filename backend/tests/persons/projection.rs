use hermes_hub_backend::domains::persons::api::{
    PersonProjectionError, PersonProjectionStore, PersonaType,
    upsert_persons_from_message_participants,
};

use super::support::{
    disconnected_persons_store, live_persons_pool, live_persons_store, unique_suffix,
};

#[tokio::test]
async fn persons_projection_upserts_email_identities_against_postgres() {
    let Some(store) = live_persons_store("persons projection").await else {
        return;
    };
    let suffix = unique_suffix();

    let persons = upsert_persons_from_message_participants(
        &store,
        &[
            format!("alice-{suffix}@example.com"),
            format!("bob-{suffix}@example.com"),
        ],
    )
    .await
    .expect("upsert persons");

    assert_eq!(persons.len(), 2);
    assert!(
        persons
            .iter()
            .any(|p| p.email_address == format!("alice-{suffix}@example.com"))
    );
    assert!(
        persons
            .iter()
            .any(|p| p.email_address == format!("bob-{suffix}@example.com"))
    );
}

#[tokio::test]
async fn persons_projection_normalizes_and_deduplicates_participants_against_postgres() {
    let Some(store) = live_persons_store("persons normalization and deduplication").await else {
        return;
    };
    let suffix = unique_suffix();
    let normalized_email = format!("alice-{suffix}@example.com");

    let persons = upsert_persons_from_message_participants(
        &store,
        &[
            format!(" Alice-{suffix}@Example.com "),
            format!("alice-{suffix}@example.com"),
        ],
    )
    .await
    .expect("upsert normalized persons");

    assert_eq!(persons.len(), 1);
    assert_eq!(persons[0].email_address, normalized_email);
    assert_eq!(persons[0].display_name, normalized_email);
}

#[tokio::test]
async fn persons_projection_rejects_blank_email_participant() {
    let store = disconnected_persons_store();

    let error = upsert_persons_from_message_participants(&store, &[String::from("   ")])
        .await
        .expect_err("blank email input must fail");

    assert!(
        matches!(error, PersonProjectionError::EmptyEmailAddress),
        "expected EmptyEmailAddress, got {error:?}"
    );
}

#[tokio::test]
async fn persons_projection_rejects_invalid_batch_before_writing_against_postgres() {
    let Some(pool) = live_persons_pool("persons invalid batch atomicity").await else {
        return;
    };
    let store = PersonProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let valid_email = format!("valid-before-blank-{suffix}@example.com");

    let error = upsert_persons_from_message_participants(
        &store,
        &[valid_email.clone(), String::from("   ")],
    )
    .await
    .expect_err("invalid participant batch must fail");

    assert!(
        matches!(error, PersonProjectionError::EmptyEmailAddress),
        "expected EmptyEmailAddress, got {error:?}"
    );

    let count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM persons WHERE email_address = $1")
            .bind(&valid_email)
            .fetch_one(&pool)
            .await
            .expect("person count after rejected batch");
    assert_eq!(count, 0);
}

#[tokio::test]
async fn persons_projection_distinguishes_delimiter_bearing_email_identities_against_postgres() {
    let Some(store) = live_persons_store("delimiter-bearing person identities").await else {
        return;
    };
    let suffix = unique_suffix();

    let left = store
        .upsert_email_person(&format!("person:{suffix}@example.com"))
        .await
        .expect("upsert delimiter-bearing person");
    let right = store
        .upsert_email_person(&format!("person-{suffix}@example.com"))
        .await
        .expect("upsert non-delimiter person");

    assert_ne!(left.person_id, right.person_id);
    assert!(left.person_id.starts_with("person:v1:email:"));
    assert!(right.person_id.starts_with("person:v1:email:"));
}

#[tokio::test]
async fn persons_projection_defaults_to_human_non_owner_persona_against_postgres() {
    let Some(store) = live_persons_store("persona defaults").await else {
        return;
    };
    let suffix = unique_suffix();

    let person = store
        .upsert_email_person(&format!("persona-default-{suffix}@example.com"))
        .await
        .expect("upsert default persona");

    assert_eq!(person.persona_type, PersonaType::Human);
    assert!(!person.is_self);
}

#[tokio::test]
async fn persons_projection_tracks_single_owner_persona_against_postgres() {
    let Some(store) = live_persons_store("single owner persona").await else {
        return;
    };
    let suffix = unique_suffix();
    let first = store
        .upsert_email_person(&format!("owner-first-{suffix}@example.com"))
        .await
        .expect("upsert first owner candidate");
    let second = store
        .upsert_email_person(&format!("owner-second-{suffix}@example.com"))
        .await
        .expect("upsert second owner candidate");

    let first_owner = store
        .set_owner_persona(&first.person_id)
        .await
        .expect("set first owner persona");
    assert!(first_owner.is_self);

    let second_owner = store
        .set_owner_persona(&second.person_id)
        .await
        .expect("move owner persona");
    assert!(second_owner.is_self);

    let owner = store
        .owner_persona()
        .await
        .expect("load owner persona")
        .expect("owner persona exists");
    assert_eq!(owner.person_id, second.person_id);
}

#[tokio::test]
async fn persons_projection_sets_supported_persona_type_against_postgres() {
    let Some(store) = live_persons_store("persona type update").await else {
        return;
    };
    let suffix = unique_suffix();
    let person = store
        .upsert_email_person(&format!("ai-agent-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let updated = store
        .set_persona_type(&person.person_id, PersonaType::AiAgent)
        .await
        .expect("set persona type");

    assert_eq!(updated.persona_type, PersonaType::AiAgent);
    assert!(!updated.is_self);
}

#[tokio::test]
async fn persons_schema_rejects_invalid_persona_type_against_postgres() {
    let Some(pool) = live_persons_pool("invalid persona type").await else {
        return;
    };
    let store = PersonProjectionStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = store
        .upsert_email_person(&format!("invalid-type-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let error = sqlx::query("UPDATE persons SET person_type = 'contact' WHERE person_id = $1")
        .bind(&person.person_id)
        .execute(&pool)
        .await
        .expect_err("invalid persona type must violate the check constraint");

    assert!(
        error.to_string().contains("persons_person_type_check"),
        "expected persons_person_type_check violation, got {error}"
    );
}
