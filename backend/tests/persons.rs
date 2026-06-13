use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Duration, Utc};
use hermes_hub_backend::domains::obligations::{
    ObligationEntityKind, ObligationReviewState, ObligationStatus, ObligationStore,
};
use hermes_hub_backend::domains::persons::api::{
    PersonProjectionError, PersonProjectionStore, PersonaType,
    upsert_persons_from_message_participants,
};
use hermes_hub_backend::domains::persons::core::{
    NewPersonPersona, PersonPersonaStore, PersonRoleStore, PersonsIdentityStore,
};
use hermes_hub_backend::domains::persons::enrichment::PersonEnrichmentStore;
use hermes_hub_backend::domains::persons::enrichment_engine::EnrichmentResultStore;
use hermes_hub_backend::domains::persons::expertise::PersonExpertiseStore;
use hermes_hub_backend::domains::persons::health::PersonHealthStore;
use hermes_hub_backend::domains::persons::intelligence::CommunicationFingerprint;
use hermes_hub_backend::domains::persons::investigator::PersonInvestigator;
use hermes_hub_backend::domains::persons::memory::{PersonFactStore, PersonPreferenceStore};
use hermes_hub_backend::domains::persons::trust::{PersonPromiseStore, PersonRiskStore};
use hermes_hub_backend::platform::storage::Database;
use serde_json::json;

#[tokio::test]
async fn persons_projection_upserts_email_identities_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live persons projection test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = PersonProjectionStore::new(database.pool().expect("configured pool").clone());
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
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live persons invalid batch atomicity test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
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
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live invalid persona type test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
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

#[tokio::test]
async fn person_identities_accept_document_and_message_traces_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live persona identity trace type test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let projection_store = PersonProjectionStore::new(pool.clone());
    let identity_store = PersonsIdentityStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = projection_store
        .upsert_email_person(&format!("identity-trace-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let document_trace = identity_store
        .upsert(
            &person.person_id,
            "document_mention",
            &format!("document:v1:{suffix}:identity-trace"),
            "document_processing",
        )
        .await
        .expect("upsert document mention identity trace");
    let message_trace = identity_store
        .upsert(
            &person.person_id,
            "message_participant",
            &format!("message:v1:{suffix}:identity-trace"),
            "communication_projection",
        )
        .await
        .expect("upsert message participant identity trace");

    assert_eq!(document_trace.identity_type, "document_mention");
    assert_eq!(document_trace.source, "document_processing");
    assert_eq!(message_trace.identity_type, "message_participant");
    assert_eq!(message_trace.source, "communication_projection");

    let identities = identity_store
        .list_by_person(&person.person_id)
        .await
        .expect("list persona identities");
    assert!(
        identities
            .iter()
            .any(|identity| identity.identity_type == "document_mention")
    );
    assert!(
        identities
            .iter()
            .any(|identity| identity.identity_type == "message_participant")
    );
}

#[tokio::test]
async fn person_identities_accept_disputed_status_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live persona identity disputed status test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let projection_store = PersonProjectionStore::new(pool.clone());
    let identity_store = PersonsIdentityStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = projection_store
        .upsert_email_person(&format!("identity-disputed-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let identity = identity_store
        .upsert(
            &person.person_id,
            "email",
            &format!("identity-disputed-trace-{suffix}@example.com"),
            "manual",
        )
        .await
        .expect("upsert identity");

    identity_store
        .update_status(&identity.id, "disputed")
        .await
        .expect("mark identity as disputed");

    let identities = identity_store
        .list_by_person(&person.person_id)
        .await
        .expect("list persona identities");
    let updated = identities
        .iter()
        .find(|candidate| candidate.id == identity.id)
        .expect("updated identity");
    assert_eq!(updated.status, "disputed");
}

#[tokio::test]
async fn person_identities_support_unattached_trace_assignment_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live unattached persona identity trace test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let projection_store = PersonProjectionStore::new(pool.clone());
    let identity_store = PersonsIdentityStore::new(pool.clone());
    let suffix = unique_suffix();

    let trace = identity_store
        .create_unattached(
            "message_participant",
            &format!("message:v1:{suffix}:unattached-participant"),
            "communication_projection",
        )
        .await
        .expect("create unattached identity trace");
    assert_eq!(trace.person_id.as_deref(), None);
    assert_eq!(trace.identity_type, "message_participant");
    assert_eq!(trace.source, "communication_projection");

    let person = projection_store
        .upsert_email_person(&format!("attach-trace-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let attached = identity_store
        .attach_to_persona(&trace.id, &person.person_id)
        .await
        .expect("attach identity trace to persona");

    assert_eq!(attached.id, trace.id);
    assert_eq!(
        attached.person_id.as_deref(),
        Some(person.person_id.as_str())
    );
    assert_eq!(attached.status, "active");

    let identities = identity_store
        .list_by_person(&person.person_id)
        .await
        .expect("list persona identities");
    assert!(identities.iter().any(|identity| identity.id == trace.id
        && identity.person_id.as_deref() == Some(person.person_id.as_str())));
}

#[tokio::test]
async fn person_role_assign_and_remove_materializes_relationship_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live person role relationship adapter test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let person_store = PersonProjectionStore::new(pool.clone());
    let role_store = PersonRoleStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("role-adapter-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let role = role_store
        .assign(
            &person.person_id,
            "Technical Advisor",
            Some("persona:owner"),
        )
        .await
        .expect("assign person role");

    let relationship: (
        String,
        String,
        String,
        String,
        String,
        String,
        f64,
        f64,
        f64,
        serde_json::Value,
    ) = sqlx::query_as(
        r#"
        SELECT
            relationship_id,
            source_entity_kind,
            source_entity_id,
            target_entity_kind,
            target_entity_id,
            review_state,
            trust_score::float8 AS trust_score,
            strength_score::float8 AS strength_score,
            confidence::float8 AS confidence,
            metadata
        FROM relationships
        WHERE source_entity_kind = 'persona'
          AND source_entity_id = $1
          AND target_entity_kind = 'knowledge'
          AND target_entity_id = 'person_role:technical_advisor'
          AND relationship_type = 'has_role'
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("role relationship");

    assert_eq!(relationship.1, "persona");
    assert_eq!(relationship.2, person.person_id);
    assert_eq!(relationship.3, "knowledge");
    assert_eq!(relationship.4, "person_role:technical_advisor");
    assert_eq!(relationship.5, "user_confirmed");
    assert_eq!(relationship.6, 1.0);
    assert_eq!(relationship.7, 0.7);
    assert_eq!(relationship.8, 1.0);
    assert_eq!(relationship.9["compatibility_source"], "person_roles");
    assert_eq!(relationship.9["role"], "Technical Advisor");
    assert_eq!(relationship.9["assigned_by"], "persona:owner");

    let evidence: (String, String, Option<String>, serde_json::Value) = sqlx::query_as(
        r#"
        SELECT source_kind, source_id, excerpt, metadata
        FROM relationship_evidence
        WHERE relationship_id = $1
        "#,
    )
    .bind(&relationship.0)
    .fetch_one(&pool)
    .await
    .expect("role relationship evidence");
    assert_eq!(evidence.0, "raw_record");
    assert_eq!(evidence.1, role.id);
    assert_eq!(evidence.2.as_deref(), Some("Technical Advisor"));
    assert_eq!(evidence.3["compatibility_source"], "person_roles");

    let removed = role_store
        .remove(&person.person_id, "Technical Advisor")
        .await
        .expect("remove person role");
    assert!(removed);

    let review_state: String =
        sqlx::query_scalar("SELECT review_state FROM relationships WHERE relationship_id = $1")
            .bind(&relationship.0)
            .fetch_one(&pool)
            .await
            .expect("demoted role relationship");
    assert_eq!(review_state, "user_rejected");
}

#[tokio::test]
async fn person_persona_upsert_and_delete_materializes_interaction_preferences_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live person persona interaction preference adapter test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let person_store = PersonProjectionStore::new(pool.clone());
    let persona_store = PersonPersonaStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("persona-context-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let persona_id = format!("interaction-context-{suffix}");

    let context = persona_store
        .upsert(&NewPersonPersona {
            persona_id: persona_id.clone(),
            person_id: person.person_id.clone(),
            name: "Work Context".to_owned(),
            context: Some("Professional replies for project updates".to_owned()),
            default_tone: Some("concise".to_owned()),
            default_language: Some("en".to_owned()),
            preferred_channel: Some("email".to_owned()),
        })
        .await
        .expect("upsert interaction context");

    let source = format!("person_personas:{persona_id}");
    let preferences: Vec<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT preference_type, value, source
        FROM person_preferences
        WHERE person_id = $1 AND source = $2
        ORDER BY preference_type
        "#,
    )
    .bind(&person.person_id)
    .bind(&source)
    .fetch_all(&pool)
    .await
    .expect("interaction context preferences");

    assert_eq!(
        preferences,
        vec![
            (
                format!("interaction_context:{persona_id}:context"),
                "Professional replies for project updates".to_owned(),
                source.clone(),
            ),
            (
                format!("interaction_context:{persona_id}:default_language"),
                "en".to_owned(),
                source.clone(),
            ),
            (
                format!("interaction_context:{persona_id}:default_tone"),
                "concise".to_owned(),
                source.clone(),
            ),
            (
                format!("interaction_context:{persona_id}:name"),
                "Work Context".to_owned(),
                source.clone(),
            ),
            (
                format!("interaction_context:{persona_id}:preferred_channel"),
                "email".to_owned(),
                source.clone(),
            ),
        ]
    );

    let deleted = persona_store
        .delete(&context.persona_id)
        .await
        .expect("delete interaction context");
    assert!(deleted);

    let remaining_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM person_preferences WHERE person_id = $1 AND source = $2",
    )
    .bind(&person.person_id)
    .bind(&source)
    .fetch_one(&pool)
    .await
    .expect("remaining interaction context preference count");
    assert_eq!(remaining_count, 0);
}

#[tokio::test]
async fn person_enrichment_trust_score_materializes_owner_relationship_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live person enrichment trust relationship adapter test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let person_store = PersonProjectionStore::new(pool.clone());
    let enrichment_store = PersonEnrichmentStore::new(pool.clone());
    let suffix = unique_suffix();
    let owner = person_store
        .upsert_email_person(&format!("trust-owner-{suffix}@example.com"))
        .await
        .expect("upsert owner persona");
    let target = person_store
        .upsert_email_person(&format!("trust-target-{suffix}@example.com"))
        .await
        .expect("upsert target persona");
    person_store
        .set_owner_persona(&owner.person_id)
        .await
        .expect("set owner persona");

    let fingerprint = CommunicationFingerprint {
        avg_message_length: Some(120),
        avg_response_hours: Some(3.5),
        frequent_topics: vec!["project".to_owned()],
        typical_tone: Some("friendly".to_owned()),
        detected_language: Some("en".to_owned()),
        writing_style: Some("balanced".to_owned()),
        preferred_time_of_day: None,
        trust_score: Some(82),
    };

    enrichment_store
        .enrich_person(&target.person_id, &fingerprint)
        .await
        .expect("enrich target persona");

    let relationship: (
        String,
        String,
        String,
        String,
        f64,
        f64,
        f64,
        serde_json::Value,
    ) = sqlx::query_as(
        r#"
        SELECT
            relationship_id,
            source_entity_id,
            target_entity_id,
            review_state,
            trust_score::float8 AS trust_score,
            strength_score::float8 AS strength_score,
            confidence::float8 AS confidence,
            metadata
        FROM relationships
        WHERE source_entity_kind = 'persona'
          AND source_entity_id = $1
          AND target_entity_kind = 'persona'
          AND target_entity_id = $2
          AND relationship_type = 'trusts'
        "#,
    )
    .bind(&owner.person_id)
    .bind(&target.person_id)
    .fetch_one(&pool)
    .await
    .expect("owner trust relationship");

    assert_eq!(relationship.1, owner.person_id);
    assert_eq!(relationship.2, target.person_id);
    assert_eq!(relationship.3, "suggested");
    assert_eq!(relationship.4, 0.82);
    assert_eq!(relationship.5, 0.5);
    assert_eq!(relationship.6, 1.0);
    assert_eq!(
        relationship.7["compatibility_source"],
        "persons.trust_score"
    );
    assert_eq!(relationship.7["trust_score"], 82);

    let evidence: (String, String, Option<String>, serde_json::Value) = sqlx::query_as(
        r#"
        SELECT source_kind, source_id, excerpt, metadata
        FROM relationship_evidence
        WHERE relationship_id = $1
        "#,
    )
    .bind(&relationship.0)
    .fetch_one(&pool)
    .await
    .expect("trust relationship evidence");
    assert_eq!(evidence.0, "raw_record");
    assert_eq!(
        evidence.1,
        format!("person_enrichment:{}:trust_score", target.person_id)
    );
    assert_eq!(evidence.2.as_deref(), Some("trust_score=82"));
    assert_eq!(evidence.3["compatibility_source"], "persons.trust_score");
    assert_eq!(
        evidence.3["trust_source_reliability"]["signal_type"],
        "source_reliability"
    );
    assert_eq!(
        evidence.3["trust_source_reliability"]["affected_source"],
        format!("person_enrichment:{}:trust_score", target.person_id)
    );
    assert_eq!(
        evidence.3["trust_source_reliability"]["direction"],
        "positive"
    );
    assert_eq!(evidence.3["trust_source_reliability"]["confidence"], 0.82);
}

#[tokio::test]
async fn person_notes_materialize_persona_memory_card_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live person notes memory adapter test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let person_store = PersonProjectionStore::new(pool.clone());
    let enrichment_store = PersonEnrichmentStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("notes-memory-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let source = format!("persons.notes:{}", person.person_id);

    enrichment_store
        .set_notes(
            &person.person_id,
            "Remember that this Persona prefers concise written summaries.",
        )
        .await
        .expect("set notes");

    let root_notes: String = sqlx::query_scalar("SELECT notes FROM persons WHERE person_id = $1")
        .bind(&person.person_id)
        .fetch_one(&pool)
        .await
        .expect("root compatibility notes");
    assert_eq!(
        root_notes,
        "Remember that this Persona prefers concise written summaries."
    );

    let memory_card: (String, String, String, i16) = sqlx::query_as(
        r#"
        SELECT title, description, source, importance
        FROM person_memory_cards
        WHERE person_id = $1 AND source = $2
        "#,
    )
    .bind(&person.person_id)
    .bind(&source)
    .fetch_one(&pool)
    .await
    .expect("notes memory card");
    assert_eq!(memory_card.0, "Compatibility notes");
    assert_eq!(
        memory_card.1,
        "Remember that this Persona prefers concise written summaries."
    );
    assert_eq!(memory_card.2, source);
    assert_eq!(memory_card.3, 5);
}

#[tokio::test]
async fn person_fact_upsert_uses_memory_engine_source_backed_draft_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live person fact memory engine adapter test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let person_store = PersonProjectionStore::new(pool.clone());
    let fact_store = PersonFactStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("fact-memory-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let fact = fact_store
        .upsert(
            &person.person_id,
            " interest ",
            " local-first systems ",
            " communication_messages:message-1 ",
            0.84,
        )
        .await
        .expect("upsert fact");

    assert_eq!(fact.person_id, person.person_id);
    assert_eq!(fact.fact_type, "interest");
    assert_eq!(fact.value, "local-first systems");
    assert_eq!(fact.source, "communication_messages:message-1");
    assert!((fact.confidence - 0.84).abs() < 0.0001);
    assert!(fact.is_active);

    let stored_fact: (String, String, String, f64) = sqlx::query_as(
        r#"
        SELECT fact_type, value, source, confidence::float8 AS confidence
        FROM person_facts
        WHERE id::text = $1
        "#,
    )
    .bind(&fact.id)
    .fetch_one(&pool)
    .await
    .expect("stored fact");
    assert_eq!(stored_fact.0, "interest");
    assert_eq!(stored_fact.1, "local-first systems");
    assert_eq!(stored_fact.2, "communication_messages:message-1");
    assert!((stored_fact.3 - 0.84).abs() < 0.0001);
}

#[tokio::test]
async fn person_favorite_toggle_materializes_ui_preference_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live person favorite preference adapter test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let person_store = PersonProjectionStore::new(pool.clone());
    let enrichment_store = PersonEnrichmentStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("favorite-preference-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let source = format!("persons.is_favorite:{}", person.person_id);

    let is_favorite = enrichment_store
        .toggle_favorite(&person.person_id)
        .await
        .expect("toggle favorite on");
    assert!(is_favorite);

    let preference: (String, String, String, f32) = sqlx::query_as(
        r#"
        SELECT preference_type, value, source, confidence
        FROM person_preferences
        WHERE person_id = $1 AND preference_type = 'ui:favorite'
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("favorite UI preference");
    assert_eq!(preference.0, "ui:favorite");
    assert_eq!(preference.1, "true");
    assert_eq!(preference.2, source);
    assert_eq!(preference.3, 1.0);

    let is_favorite = enrichment_store
        .toggle_favorite(&person.person_id)
        .await
        .expect("toggle favorite off");
    assert!(!is_favorite);

    let preference_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM person_preferences WHERE person_id = $1 AND preference_type = 'ui:favorite'",
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("remaining favorite preference count");
    assert_eq!(preference_count, 0);
}

#[tokio::test]
async fn person_enrichment_result_upsert_materializes_pending_source_backed_candidate_against_postgres()
 {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live person enrichment result candidate test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let person_store = PersonProjectionStore::new(pool.clone());
    let enrichment_result_store = EnrichmentResultStore::new(pool);
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("enrichment-candidate-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let result = enrichment_result_store
        .upsert(
            &person.person_id,
            "communication_messages:message-1",
            json!({
                "field": "communication_style",
                "value": "concise asynchronous summaries",
                "extracted_claim": "prefers concise asynchronous summaries"
            }),
            0.82,
        )
        .await
        .expect("upsert enrichment result");

    assert_eq!(result.person_id, person.person_id);
    assert_eq!(result.source, "communication_messages:message-1");
    assert!((result.confidence - 0.82).abs() < 0.0001);
    assert_eq!(result.status, "pending");
    assert_eq!(result.data["field"], "communication_style");
    assert_eq!(
        result.data["_enrichment"]["affected_entity_kind"],
        "persona"
    );
    assert_eq!(
        result.data["_enrichment"]["affected_entity_id"],
        person.person_id
    );
    assert_eq!(
        result.data["_enrichment"]["extracted_claim"],
        "prefers concise asynchronous summaries"
    );
    assert_eq!(result.data["_enrichment"]["review_state"], "pending");
    assert_eq!(result.data["_enrichment"]["freshness"], "current");
    assert_eq!(result.data["_enrichment"]["conflict_marker"], false);
}

#[tokio::test]
async fn person_watchlist_toggle_materializes_ui_preference_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live person watchlist preference adapter test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let person_store = PersonProjectionStore::new(pool.clone());
    let health_store = PersonHealthStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("watchlist-preference-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let source = format!("persons.watchlist:{}", person.person_id);

    let watchlist = health_store
        .toggle_watchlist(&person.person_id)
        .await
        .expect("toggle watchlist on");
    assert!(watchlist);

    let preference: (String, String, String, f32) = sqlx::query_as(
        r#"
        SELECT preference_type, value, source, confidence
        FROM person_preferences
        WHERE person_id = $1 AND preference_type = 'ui:watchlist'
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("watchlist UI preference");
    assert_eq!(preference.0, "ui:watchlist");
    assert_eq!(preference.1, "true");
    assert_eq!(preference.2, source);
    assert_eq!(preference.3, 1.0);

    let watchlist = health_store
        .toggle_watchlist(&person.person_id)
        .await
        .expect("toggle watchlist off");
    assert!(!watchlist);

    let preference_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM person_preferences WHERE person_id = $1 AND preference_type = 'ui:watchlist'",
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("remaining watchlist preference count");
    assert_eq!(preference_count, 0);
}

#[tokio::test]
async fn person_risk_report_and_resolve_materializes_health_status_cache_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live person risk health adapter test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let person_store = PersonProjectionStore::new(pool.clone());
    let risk_store = PersonRiskStore::new(pool.clone());
    let health_store = PersonHealthStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("health-risk-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    let risk = risk_store
        .report(
            &person.person_id,
            "relationship_attention",
            "Open evidence-backed relationship risk requires owner review.",
            "high",
            "risk-engine:test",
        )
        .await
        .expect("report risk");

    let stored_risk: (String, String, String, String, f64) = sqlx::query_as(
        r#"
        SELECT risk_type, description, severity, source, confidence::float8 AS confidence
        FROM person_risks
        WHERE id::text = $1
        "#,
    )
    .bind(&risk.id)
    .fetch_one(&pool)
    .await
    .expect("stored risk observation");
    assert_eq!(stored_risk.0, "relationship_attention");
    assert_eq!(
        stored_risk.1,
        "Open evidence-backed relationship risk requires owner review."
    );
    assert_eq!(stored_risk.2, "high");
    assert_eq!(stored_risk.3, "risk-engine:test");
    assert_eq!(stored_risk.4, 0.5);

    let health = health_store
        .get(&person.person_id)
        .await
        .expect("load health")
        .expect("health row");
    assert_eq!(health.health_status, "at_risk");
    assert_eq!(health.open_risks, 1);

    risk_store
        .resolve(&risk.id, "owner reviewed and closed the risk")
        .await
        .expect("resolve risk");

    let health = health_store
        .get(&person.person_id)
        .await
        .expect("load health after resolve")
        .expect("health row after resolve");
    assert_eq!(health.health_status, "healthy");
    assert_eq!(health.open_risks, 0);
}

#[tokio::test]
async fn person_dossier_includes_target_sections_and_source_refs_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live person dossier read-model test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let person_store = PersonProjectionStore::new(pool.clone());
    let fact_store = PersonFactStore::new(pool.clone());
    let preference_store = PersonPreferenceStore::new(pool.clone());
    let expertise_store = PersonExpertiseStore::new(pool.clone());
    let investigator = PersonInvestigator::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("dossier-read-model-{suffix}@example.com"))
        .await
        .expect("upsert persona");

    fact_store
        .upsert(
            &person.person_id,
            "interest",
            "local-first systems",
            "message:dossier-interest",
            0.9,
        )
        .await
        .expect("insert interest fact");
    fact_store
        .upsert(
            &person.person_id,
            "project",
            "Hermes Memory Graph",
            "document:dossier-project",
            0.8,
        )
        .await
        .expect("insert project fact");
    fact_store
        .upsert(
            &person.person_id,
            "organization",
            "Hermes Lab",
            "relationship:dossier-organization",
            0.85,
        )
        .await
        .expect("insert organization fact");
    preference_store
        .upsert(
            &person.person_id,
            "communication:preferred_channel",
            "telegram",
            "message:dossier-preference",
        )
        .await
        .expect("insert communication preference");
    expertise_store
        .upsert(
            &person.person_id,
            "Rust backend architecture",
            Some("software"),
            "document:dossier-skill",
            0.95,
        )
        .await
        .expect("insert expertise");

    let dossier = investigator
        .assemble_dossier(&person.person_id)
        .await
        .expect("assemble dossier");
    let dossier_json = serde_json::to_value(&dossier).expect("dossier json");

    assert_eq!(dossier_json["interests"][0]["value"], "local-first systems");
    assert_eq!(
        dossier_json["interests"][0]["source_refs"][0],
        "message:dossier-interest"
    );
    assert_eq!(dossier_json["projects"][0]["value"], "Hermes Memory Graph");
    assert_eq!(dossier_json["organizations"][0]["value"], "Hermes Lab");
    assert_eq!(
        dossier_json["skills"][0]["value"],
        "Rust backend architecture"
    );
    assert_eq!(
        dossier_json["communication_patterns"][0]["value"],
        "telegram"
    );
    assert!(
        dossier_json["source_refs"]
            .as_array()
            .expect("source refs array")
            .iter()
            .any(|source| source == "document:dossier-skill")
    );
    assert!(
        dossier_json["generated_at"].is_string(),
        "dossier must include generated_at"
    );
    assert!(
        dossier_json["ai_observations"].is_array(),
        "dossier must expose ai_observations as a labeled derived section"
    );
}

#[tokio::test]
async fn person_promise_create_materializes_user_confirmed_obligation_without_task_against_postgres()
 {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live person promise obligation adapter test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let person_store = PersonProjectionStore::new(pool.clone());
    let promise_store = PersonPromiseStore::new(pool.clone());
    let obligation_store = ObligationStore::new(pool.clone());
    let suffix = unique_suffix();
    let person = person_store
        .upsert_email_person(&format!("promise-adapter-{suffix}@example.com"))
        .await
        .expect("upsert persona");
    let due_at = Utc::now() + Duration::days(5);
    let description = format!("Send the persona promise evidence package {suffix}");

    let promise = promise_store
        .create(&person.person_id, &description, Some(due_at))
        .await
        .expect("create person promise");

    let obligations = obligation_store
        .list_for_entity(ObligationEntityKind::Persona, &person.person_id, 10)
        .await
        .expect("persona obligations");
    let obligation = obligations
        .iter()
        .find(|item| item.statement == description)
        .expect("person promise should create a durable Obligation");

    assert_eq!(
        obligation.review_state,
        ObligationReviewState::UserConfirmed
    );
    assert_eq!(obligation.status, ObligationStatus::Open);
    assert_eq!(
        obligation.due_at.map(|value| value.timestamp_micros()),
        Some(due_at.timestamp_micros())
    );
    assert_eq!(
        obligation.metadata["person_promise_id"],
        serde_json::json!(promise.id)
    );

    let evidence: (String, String, Option<String>) = sqlx::query_as(
        "SELECT source_kind, source_id, quote FROM obligation_evidence WHERE obligation_id = $1",
    )
    .bind(&obligation.obligation_id)
    .fetch_one(&pool)
    .await
    .expect("obligation evidence");
    assert_eq!(evidence.0, "raw_record");
    assert_eq!(evidence.1, promise.id);
    assert_eq!(evidence.2.as_deref(), Some(description.as_str()));

    let task_link_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM obligation_task_links WHERE obligation_id = $1")
            .bind(&obligation.obligation_id)
            .fetch_one(&pool)
            .await
            .expect("task link count");
    assert_eq!(task_link_count, 0);
}

async fn live_persons_store(test_name: &str) -> Option<PersonProjectionStore> {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live {test_name} test: HERMES_TEST_DATABASE_URL is not set");
        return None;
    };

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    Some(PersonProjectionStore::new(
        database.pool().expect("configured pool").clone(),
    ))
}

fn disconnected_persons_store() -> PersonProjectionStore {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://hermes:unused@127.0.0.1:1/hermes_hub")
        .expect("create lazy test pool");
    PersonProjectionStore::new(pool)
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}
