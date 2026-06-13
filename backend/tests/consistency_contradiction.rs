use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Duration, Utc};
use hermes_hub_backend::domains::calendar::events::{CalendarEventStore, NewCalendarEvent};
use hermes_hub_backend::domains::calendar::meetings::MeetingNoteStore;
use hermes_hub_backend::domains::documents::core::{DocumentImportStore, NewDocumentImport};
use hermes_hub_backend::domains::mail::core::{
    CommunicationIngestionStore, CommunicationProviderKind, EmailProviderKind, NewProviderAccount,
    NewRawCommunicationRecord,
};
use hermes_hub_backend::domains::mail::messages::{
    MessageProjectionStore, project_raw_email_message,
};
use hermes_hub_backend::domains::persons::api::PersonProjectionStore;
use hermes_hub_backend::engines::consistency::{
    AcceptedClaim, ConsistencyEngine, ContradictionObservationStore, ContradictionReviewState,
    ContradictionSeverity, ContradictionSourceKind, EvidenceClaimExtractionInput,
    NewContradictionObservation, NewEvidenceClaim,
};
use hermes_hub_backend::integrations::telegram::client::project_raw_telegram_message;
use hermes_hub_backend::integrations::whatsapp::client::project_raw_whatsapp_web_message;
use hermes_hub_backend::platform::calls::{
    CallDirection, CallIntelligenceStore, CallState, NewCallTranscript, NewTelegramCall,
    TranscriptStatus,
};
use hermes_hub_backend::platform::storage::Database;
use serde_json::json;
use sqlx::postgres::PgPool;

#[test]
fn consistency_engine_detects_direct_claim_contradiction_from_structured_claims() {
    let accepted = AcceptedClaim {
        subject_id: "person:v1:email:alex@example.com".to_owned(),
        claim_type: "location".to_owned(),
        value: "Berlin".to_owned(),
        source_kind: ContradictionSourceKind::Memory,
        source_id: "person_fact:location:alex".to_owned(),
        confidence: 0.95,
    };
    let new_claim = NewEvidenceClaim {
        subject_id: "person:v1:email:alex@example.com".to_owned(),
        claim_type: "location".to_owned(),
        value: "Madrid".to_owned(),
        source_kind: ContradictionSourceKind::Communication,
        source_id: "message:location-update".to_owned(),
        confidence: 0.87,
    };

    let observations = ConsistencyEngine::detect_claim_contradictions(&[accepted], &[new_claim])
        .expect("detect contradictions");

    assert_eq!(observations.len(), 1);
    let observation = &observations[0];
    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(observation.old_source_id, "person_fact:location:alex");
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Communication
    );
    assert_eq!(observation.new_source_id, "message:location-update");
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.87);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.review_state,
        ContradictionReviewState::Suggested
    );
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": "person:v1:email:alex@example.com"}])
    );
}

#[test]
fn consistency_engine_ignores_matching_claims_after_normalization() {
    let accepted = AcceptedClaim {
        subject_id: "project:v1:hermes".to_owned(),
        claim_type: "status".to_owned(),
        value: " Active ".to_owned(),
        source_kind: ContradictionSourceKind::Knowledge,
        source_id: "knowledge:project-status".to_owned(),
        confidence: 0.9,
    };
    let new_claim = NewEvidenceClaim {
        subject_id: "project:v1:hermes".to_owned(),
        claim_type: "status".to_owned(),
        value: "active".to_owned(),
        source_kind: ContradictionSourceKind::Communication,
        source_id: "message:project-status".to_owned(),
        confidence: 0.8,
    };

    let observations = ConsistencyEngine::detect_claim_contradictions(&[accepted], &[new_claim])
        .expect("detect contradictions");

    assert_eq!(observations, Vec::new());
}

#[test]
fn consistency_engine_extracts_structured_claims_from_communication_evidence() {
    let input = EvidenceClaimExtractionInput {
        subject_id: "person:v1:email:alex@example.com".to_owned(),
        source_kind: ContradictionSourceKind::Communication,
        source_id: "message:claim-extraction".to_owned(),
        text: "Location: Madrid\nStatus = active\nNotes without claim\nEmpty:".to_owned(),
        confidence: 0.81,
    };

    let claims =
        ConsistencyEngine::extract_evidence_claims(&input).expect("extract evidence claims");

    assert_eq!(
        claims,
        vec![
            NewEvidenceClaim {
                subject_id: "person:v1:email:alex@example.com".to_owned(),
                claim_type: "location".to_owned(),
                value: "Madrid".to_owned(),
                source_kind: ContradictionSourceKind::Communication,
                source_id: "message:claim-extraction".to_owned(),
                confidence: 0.81,
            },
            NewEvidenceClaim {
                subject_id: "person:v1:email:alex@example.com".to_owned(),
                claim_type: "status".to_owned(),
                value: "active".to_owned(),
                source_kind: ContradictionSourceKind::Communication,
                source_id: "message:claim-extraction".to_owned(),
                confidence: 0.81,
            },
        ]
    );
}

#[test]
fn consistency_engine_extracts_deterministic_natural_language_claims_from_evidence() {
    let input = EvidenceClaimExtractionInput {
        subject_id: "person:v1:email:alex@example.com".to_owned(),
        source_kind: ContradictionSourceKind::Communication,
        source_id: "message:natural-language-claim-extraction".to_owned(),
        text: "Quick update: I am now in Madrid.\nThe project status is blocked.".to_owned(),
        confidence: 0.79,
    };

    let claims =
        ConsistencyEngine::extract_evidence_claims(&input).expect("extract evidence claims");

    assert_eq!(
        claims,
        vec![
            NewEvidenceClaim {
                subject_id: "person:v1:email:alex@example.com".to_owned(),
                claim_type: "location".to_owned(),
                value: "Madrid".to_owned(),
                source_kind: ContradictionSourceKind::Communication,
                source_id: "message:natural-language-claim-extraction".to_owned(),
                confidence: 0.79,
            },
            NewEvidenceClaim {
                subject_id: "person:v1:email:alex@example.com".to_owned(),
                claim_type: "status".to_owned(),
                value: "blocked".to_owned(),
                source_kind: ContradictionSourceKind::Communication,
                source_id: "message:natural-language-claim-extraction".to_owned(),
                confidence: 0.79,
            },
        ]
    );
}

#[test]
fn consistency_engine_detects_document_evidence_contradiction_after_claim_extraction() {
    let accepted = AcceptedClaim {
        subject_id: "project:v1:hermes".to_owned(),
        claim_type: "status".to_owned(),
        value: "green".to_owned(),
        source_kind: ContradictionSourceKind::Memory,
        source_id: "memory:project-status".to_owned(),
        confidence: 0.92,
    };
    let document = EvidenceClaimExtractionInput {
        subject_id: "project:v1:hermes".to_owned(),
        source_kind: ContradictionSourceKind::Document,
        source_id: "document:weekly-report".to_owned(),
        text: "Status: blocked".to_owned(),
        confidence: 0.84,
    };

    let observations = ConsistencyEngine::detect_evidence_contradictions(&[accepted], &[document])
        .expect("detect evidence contradictions");

    assert_eq!(observations.len(), 1);
    let observation = &observations[0];
    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(observation.old_source_id, "memory:project-status");
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Document
    );
    assert_eq!(observation.new_source_id, "document:weekly-report");
    assert_eq!(observation.old_claim, "status=green");
    assert_eq!(observation.new_claim, "status=blocked");
    assert_eq!(observation.confidence, 0.84);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "status",
            "source_kind": "document"
        })
    );
}

#[tokio::test]
async fn contradiction_observation_store_upserts_reviewable_observation_against_postgres() {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live contradiction observation test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store =
        ContradictionObservationStore::new(database.pool().expect("configured pool").clone());
    let suffix = unique_suffix();
    let observation = NewContradictionObservation {
        old_source_kind: ContradictionSourceKind::Memory,
        old_source_id: format!("memory:budget:{suffix}"),
        new_source_kind: ContradictionSourceKind::Communication,
        new_source_id: format!("message:budget:{suffix}"),
        affected_entities: json!([
            {"entity_kind": "project", "entity_id": format!("project:v1:{suffix}")}
        ]),
        conflict_type: "direct_contradiction".to_owned(),
        old_claim: "budget=approved".to_owned(),
        new_claim: "budget=rejected".to_owned(),
        confidence: 0.88,
        severity: ContradictionSeverity::High,
        review_state: ContradictionReviewState::Suggested,
        metadata: json!({"detector": "structured_claim_test"}),
    };

    let first = store
        .upsert(&observation)
        .await
        .expect("first contradiction upsert");
    let second = store
        .upsert(&observation)
        .await
        .expect("idempotent contradiction upsert");

    assert_eq!(first.observation_id, second.observation_id);
    assert_eq!(first.review_state, ContradictionReviewState::Suggested);
    assert_eq!(first.severity, ContradictionSeverity::High);
    assert_eq!(first.confidence, 0.88);

    let open = store.list_open(20).await.expect("open contradictions");
    assert!(
        open.iter()
            .any(|item| item.observation_id == first.observation_id)
    );

    let reviewed = store
        .set_review_state(
            &first.observation_id,
            ContradictionReviewState::UserConfirmed,
            "test-reviewer",
            Some("confirmed contradiction"),
        )
        .await
        .expect("review contradiction");

    assert_eq!(
        reviewed.review_state,
        ContradictionReviewState::UserConfirmed
    );
    assert_eq!(reviewed.reviewed_by.as_deref(), Some("test-reviewer"));
    assert_eq!(
        reviewed.resolution.as_deref(),
        Some("confirmed contradiction")
    );
}

#[tokio::test]
async fn contradiction_refresh_detects_message_claim_against_active_person_fact_without_overwriting_memory()
 {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!("skipping live contradiction refresh test: HERMES_TEST_DATABASE_URL is not set");
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let sender = format!("polygraph-{suffix}@example.com");
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&sender)
        .await
        .expect("person");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO person_facts (person_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let message_id = seed_message(
        &pool,
        suffix,
        &sender,
        &[format!("owner-{suffix}@example.com")],
        &format!("provider-polygraph-{suffix}"),
        &format!("Location update {suffix}"),
        "Location: Madrid",
    )
    .await;

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == message_id)
        .expect("message claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Communication
    );
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": person.person_id}])
    );
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "location",
            "source_kind": "communication"
        })
    );

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM person_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}

#[tokio::test]
async fn contradiction_refresh_detects_natural_language_message_claim_against_active_person_fact_without_overwriting_memory()
 {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live contradiction natural-language refresh test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let sender = format!("polygraph-natural-language-{suffix}@example.com");
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&sender)
        .await
        .expect("person");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO person_facts (person_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let message_id = seed_message(
        &pool,
        suffix,
        &sender,
        &[format!("owner-natural-language-{suffix}@example.com")],
        &format!("provider-polygraph-natural-language-{suffix}"),
        &format!("Natural language location update {suffix}"),
        "Quick update: I am now in Madrid.",
    )
    .await;

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == message_id)
        .expect("natural-language message claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Communication
    );
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM person_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}

#[tokio::test]
async fn contradiction_refresh_detects_document_claim_against_active_person_fact_without_overwriting_memory()
 {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live contradiction document refresh test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let email_address = format!("polygraph-document-{suffix}@example.com");
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&email_address)
        .await
        .expect("person");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO person_facts (person_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let document_id = format!("document_polygraph_{suffix}");
    DocumentImportStore::new(pool.clone())
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            format!("Persona dossier {suffix}"),
            format!("# Persona dossier\nEmail: {email_address}\nLocation: Madrid"),
        ))
        .await
        .expect("document import");

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == document_id)
        .expect("document claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Document
    );
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": person.person_id}])
    );
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "location",
            "source_kind": "document"
        })
    );

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM person_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}

#[tokio::test]
async fn contradiction_refresh_detects_meeting_note_claim_against_active_person_fact_without_overwriting_memory()
 {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live contradiction meeting note refresh test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let email_address = format!("polygraph-meeting-{suffix}@example.com");
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&email_address)
        .await
        .expect("person");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO person_facts (person_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let start_at = Utc::now();
    let event = CalendarEventStore::new(pool.clone())
        .create(&NewCalendarEvent {
            title: format!("Polygraph meeting {suffix}"),
            description: Some("Meeting evidence for consistency refresh".to_owned()),
            start_at,
            end_at: start_at + Duration::minutes(30),
            event_type: Some("meeting".to_owned()),
            ..NewCalendarEvent::default()
        })
        .await
        .expect("calendar event");
    sqlx::query(
        r#"
        INSERT INTO event_participants (event_id, email, display_name, role, person_id)
        VALUES ($1, $2, 'Polygraph Participant', 'attendee', $3)
        "#,
    )
    .bind(&event.event_id)
    .bind(&email_address)
    .bind(&person.person_id)
    .execute(&pool)
    .await
    .expect("event participant");
    let note = MeetingNoteStore::new(pool.clone())
        .create(
            &event.event_id,
            "Location: Madrid",
            Some("markdown"),
            Some("manual"),
        )
        .await
        .expect("meeting note");

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == note.id)
        .expect("meeting note claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(observation.new_source_kind, ContradictionSourceKind::Event);
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": person.person_id}])
    );
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "location",
            "source_kind": "event"
        })
    );

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM person_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}

#[tokio::test]
async fn contradiction_refresh_detects_call_transcript_claim_against_active_person_fact_without_overwriting_memory()
 {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live contradiction call transcript refresh test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let email_address = format!("polygraph-call-{suffix}@example.com");
    let provider_chat_id = format!("telegram-chat-{suffix}");
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&email_address)
        .await
        .expect("person");
    sqlx::query(
        r#"
        INSERT INTO person_identities (person_id, identity_type, identity_value, source, confidence, status)
        VALUES ($1, 'telegram', $2, 'test', 1.0, 'active')
        "#,
    )
    .bind(&person.person_id)
    .bind(&provider_chat_id)
    .execute(&pool)
    .await
    .expect("telegram identity");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO person_facts (person_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let account_id = format!("acct_polygraph_call_{suffix}");
    CommunicationIngestionStore::new(pool.clone())
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Polygraph Call Compatibility Account",
            format!("polygraph-call-{suffix}@example.com"),
        ))
        .await
        .expect("provider account");
    let call_store = CallIntelligenceStore::new(pool.clone());
    let call_id = format!("call:polygraph:{suffix}");
    call_store
        .upsert_call(&NewTelegramCall {
            call_id: call_id.clone(),
            account_id: account_id.clone(),
            provider_call_id: format!("provider-call-{suffix}"),
            provider_chat_id: provider_chat_id.clone(),
            direction: CallDirection::Incoming,
            call_state: CallState::Ended,
            started_at: Some(Utc::now()),
            ended_at: Some(Utc::now()),
            transcription_policy_id: None,
            metadata: json!({"source": "polygraph_test"}),
        })
        .await
        .expect("telegram call");
    let transcript_id = format!("call-transcript-polygraph-{suffix}");
    call_store
        .upsert_transcript(&NewCallTranscript {
            transcript_id: transcript_id.clone(),
            call_id,
            account_id,
            provider_chat_id,
            transcript_status: TranscriptStatus::Succeeded,
            stt_provider: "fixture-stt".to_owned(),
            source_audio_ref: Some(format!("audio-polygraph-{suffix}")),
            language_code: Some("en".to_owned()),
            transcript_text: "Location: Madrid".to_owned(),
            segments: json!([]),
            provenance: json!({"source": "polygraph_test"}),
        })
        .await
        .expect("call transcript");

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == transcript_id)
        .expect("call transcript claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Communication
    );
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": person.person_id}])
    );
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "location",
            "source_kind": "communication"
        })
    );

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM person_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}

#[tokio::test]
async fn contradiction_refresh_detects_telegram_message_claim_against_active_person_fact_without_overwriting_memory()
 {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live contradiction telegram message refresh test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let email_address = format!("polygraph-telegram-{suffix}@example.com");
    let sender_id = format!("telegram-sender-{suffix}");
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&email_address)
        .await
        .expect("person");
    sqlx::query(
        r#"
        INSERT INTO person_identities (person_id, identity_type, identity_value, source, confidence, status)
        VALUES ($1, 'telegram', $2, 'test', 1.0, 'active')
        "#,
    )
    .bind(&person.person_id)
    .bind(&sender_id)
    .execute(&pool)
    .await
    .expect("telegram identity");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO person_facts (person_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let message_id = seed_telegram_message(&pool, suffix, &sender_id, "Location: Madrid").await;

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == message_id)
        .expect("telegram message claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Communication
    );
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": person.person_id}])
    );
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "location",
            "source_kind": "communication"
        })
    );

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM person_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}

#[tokio::test]
async fn contradiction_refresh_detects_whatsapp_message_claim_against_active_person_fact_without_overwriting_memory()
 {
    let Some(database_url) = env::var("HERMES_TEST_DATABASE_URL").ok() else {
        eprintln!(
            "skipping live contradiction WhatsApp message refresh test: HERMES_TEST_DATABASE_URL is not set"
        );
        return;
    };
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let store = ContradictionObservationStore::new(pool.clone());
    let suffix = unique_suffix();
    let email_address = format!("polygraph-whatsapp-{suffix}@example.com");
    let sender_id = format!("whatsapp-sender-{suffix}");
    let person = PersonProjectionStore::new(pool.clone())
        .upsert_email_person(&email_address)
        .await
        .expect("person");
    sqlx::query(
        r#"
        INSERT INTO person_identities (person_id, identity_type, identity_value, source, confidence, status)
        VALUES ($1, 'whatsapp', $2, 'test', 1.0, 'active')
        "#,
    )
    .bind(&person.person_id)
    .bind(&sender_id)
    .execute(&pool)
    .await
    .expect("whatsapp identity");
    let fact_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO person_facts (person_id, fact_type, value, source, confidence)
        VALUES ($1, 'location', 'Berlin', 'manual', 0.93)
        RETURNING id::text
        "#,
    )
    .bind(&person.person_id)
    .fetch_one(&pool)
    .await
    .expect("person fact");
    let message_id = seed_whatsapp_message(&pool, suffix, &sender_id, "Location: Madrid").await;

    let refreshed = store
        .refresh_deterministic_observations(100)
        .await
        .expect("refresh contradictions");
    assert!(refreshed >= 1);

    let open = store.list_open(100).await.expect("open contradictions");
    let observation = open
        .iter()
        .find(|item| item.old_source_id == fact_id && item.new_source_id == message_id)
        .expect("WhatsApp message claim should contradict active remembered person fact");

    assert_eq!(observation.old_source_kind, ContradictionSourceKind::Memory);
    assert_eq!(
        observation.new_source_kind,
        ContradictionSourceKind::Communication
    );
    assert_eq!(observation.conflict_type, "direct_contradiction");
    assert_eq!(observation.old_claim, "location=Berlin");
    assert_eq!(observation.new_claim, "location=Madrid");
    assert_eq!(observation.confidence, 0.8);
    assert_eq!(observation.severity, ContradictionSeverity::Medium);
    assert_eq!(
        observation.affected_entities,
        json!([{"entity_kind": "subject", "entity_id": person.person_id}])
    );
    assert_eq!(
        observation.metadata,
        json!({
            "detector": "structured_evidence_claim",
            "claim_type": "location",
            "source_kind": "communication"
        })
    );

    let remembered_value: String =
        sqlx::query_scalar("SELECT value FROM person_facts WHERE id::text = $1")
            .bind(&fact_id)
            .fetch_one(&pool)
            .await
            .expect("remembered value");
    assert_eq!(remembered_value, "Berlin");
}

#[test]
fn contradiction_observation_rejects_invalid_confidence_before_database_write() {
    let observation = NewContradictionObservation {
        old_source_kind: ContradictionSourceKind::Memory,
        old_source_id: "memory:invalid".to_owned(),
        new_source_kind: ContradictionSourceKind::Communication,
        new_source_id: "message:invalid".to_owned(),
        affected_entities: json!([]),
        conflict_type: "direct_contradiction".to_owned(),
        old_claim: "status=active".to_owned(),
        new_claim: "status=archived".to_owned(),
        confidence: 1.2,
        severity: ContradictionSeverity::Medium,
        review_state: ContradictionReviewState::Suggested,
        metadata: json!({}),
    };

    let error = observation
        .validate()
        .expect_err("invalid confidence must be rejected");

    assert_eq!(
        error.to_string(),
        "confidence must be between 0.0 and 1.0: 1.2"
    );
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

async fn seed_message(
    pool: &PgPool,
    suffix: u128,
    sender: &str,
    recipients: &[String],
    provider_record_id: &str,
    subject: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_polygraph_{suffix}");
    let ingestion_store = CommunicationIngestionStore::new(pool.clone());
    ingestion_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            EmailProviderKind::Gmail,
            "Polygraph Gmail",
            format!("polygraph-{suffix}@example.com"),
        ))
        .await
        .expect("provider account");

    let raw_record_id = format!("raw_polygraph_{suffix}_{provider_record_id}");
    let raw = ingestion_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                &raw_record_id,
                &account_id,
                "email_message",
                provider_record_id,
                format!("sha256:polygraph:{suffix}:{provider_record_id}"),
                format!("batch-polygraph-{suffix}"),
                json!({
                    "subject": subject,
                    "from": sender,
                    "to": recipients,
                    "body_text": body_text,
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({"source":"polygraph_test"})),
        )
        .await
        .expect("raw message");

    let message_store = MessageProjectionStore::new(pool.clone());
    project_raw_email_message(&message_store, &raw)
        .await
        .expect("project message")
        .message_id
}

async fn seed_telegram_message(
    pool: &PgPool,
    suffix: u128,
    sender_id: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_polygraph_telegram_{suffix}");
    let provider_chat_id = format!("telegram-chat-{suffix}");
    let provider_message_id = format!("telegram-message-{suffix}");
    let ingestion_store = CommunicationIngestionStore::new(pool.clone());
    ingestion_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::TelegramUser,
            "Polygraph Telegram",
            format!("polygraph-telegram-{suffix}"),
        ))
        .await
        .expect("provider account");

    let raw = ingestion_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_polygraph_telegram_{suffix}"),
                &account_id,
                "telegram_message",
                &provider_message_id,
                format!("sha256:polygraph:telegram:{suffix}"),
                format!("batch-polygraph-telegram-{suffix}"),
                json!({
                    "provider_chat_id": provider_chat_id,
                    "chat_title": format!("Polygraph Telegram {suffix}"),
                    "chat_kind": "private",
                    "sender_id": sender_id,
                    "sender_display_name": "Polygraph Telegram Sender",
                    "text": body_text,
                    "delivery_state": "received",
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({
                "source": "polygraph_test",
                "provider": "telegram",
                "provider_kind": "telegram_user",
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
            })),
        )
        .await
        .expect("raw telegram message");

    let message_store = MessageProjectionStore::new(pool.clone());
    project_raw_telegram_message(&message_store, &raw)
        .await
        .expect("project telegram message")
        .message_id
}

async fn seed_whatsapp_message(
    pool: &PgPool,
    suffix: u128,
    sender_id: &str,
    body_text: &str,
) -> String {
    let account_id = format!("acct_polygraph_whatsapp_{suffix}");
    let provider_chat_id = format!("whatsapp-chat-{suffix}");
    let provider_message_id = format!("whatsapp-message-{suffix}");
    let ingestion_store = CommunicationIngestionStore::new(pool.clone());
    ingestion_store
        .upsert_provider_account(&NewProviderAccount::new(
            &account_id,
            CommunicationProviderKind::WhatsappWeb,
            "Polygraph WhatsApp",
            format!("polygraph-whatsapp-{suffix}"),
        ))
        .await
        .expect("provider account");

    let raw = ingestion_store
        .record_raw_source(
            &NewRawCommunicationRecord::new(
                format!("raw_polygraph_whatsapp_{suffix}"),
                &account_id,
                "whatsapp_web_message",
                &provider_message_id,
                format!("sha256:polygraph:whatsapp:{suffix}"),
                format!("batch-polygraph-whatsapp-{suffix}"),
                json!({
                    "provider_chat_id": provider_chat_id,
                    "chat_title": format!("Polygraph WhatsApp {suffix}"),
                    "sender_id": sender_id,
                    "sender_display_name": "Polygraph WhatsApp Sender",
                    "text": body_text,
                    "delivery_state": "received",
                }),
            )
            .occurred_at(Utc::now())
            .provenance(json!({
                "source": "polygraph_test",
                "provider": "whatsapp",
                "provider_kind": "whatsapp_web",
                "account_id": account_id,
                "provider_chat_id": provider_chat_id,
            })),
        )
        .await
        .expect("raw WhatsApp message");

    let message_store = MessageProjectionStore::new(pool.clone());
    project_raw_whatsapp_web_message(&message_store, &raw)
        .await
        .expect("project WhatsApp message")
        .message_id
}
