use chrono::{Duration, Utc};
use hermes_hub_backend::domains::obligations::{
    ObligationEntityKind, ObligationReviewState, ObligationStatus, ObligationStore,
};
use hermes_hub_backend::domains::persons::api::PersonProjectionStore;
use hermes_hub_backend::domains::persons::core::PersonRoleStore;
use hermes_hub_backend::domains::persons::enrichment::PersonEnrichmentStore;
use hermes_hub_backend::domains::persons::intelligence::CommunicationFingerprint;
use hermes_hub_backend::domains::persons::trust::PersonPromiseStore;

use super::support::{live_persons_pool, unique_suffix};

#[tokio::test]
async fn person_role_assign_and_remove_materializes_relationship_against_postgres() {
    let Some(pool) = live_persons_pool("person role relationship adapter").await else {
        return;
    };
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
async fn person_enrichment_trust_score_materializes_owner_relationship_against_postgres() {
    let Some(pool) = live_persons_pool("person enrichment trust relationship adapter").await else {
        return;
    };
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
async fn person_promise_create_materializes_user_confirmed_obligation_without_task_against_postgres()
 {
    let Some(pool) = live_persons_pool("person promise obligation adapter").await else {
        return;
    };
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
