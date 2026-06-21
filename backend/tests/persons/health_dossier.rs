use hermes_hub_backend::domains::persons::api::PersonProjectionStore;
use hermes_hub_backend::domains::persons::expertise::PersonExpertiseStore;
use hermes_hub_backend::domains::persons::health::PersonHealthStore;
use hermes_hub_backend::domains::persons::investigator::PersonInvestigator;
use hermes_hub_backend::domains::persons::memory::{PersonFactStore, PersonPreferenceStore};
use hermes_hub_backend::domains::persons::trust::PersonRiskStore;

use super::support::{live_persons_pool, unique_suffix};

#[tokio::test]
async fn person_risk_report_and_resolve_materializes_health_status_cache_against_postgres() {
    let Some(pool) = live_persons_pool("person risk health adapter").await else {
        return;
    };
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
    let Some(pool) = live_persons_pool("person dossier read-model").await else {
        return;
    };
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
