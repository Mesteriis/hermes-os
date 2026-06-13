use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

const MAX_REFRESH_LIMIT: i64 = 100;
const MIN_REFRESH_LIMIT: i64 = 1;
const STRUCTURED_EVIDENCE_CLAIM_CONFIDENCE: f64 = 0.8;

#[derive(Clone)]
pub struct ContradictionObservationStore {
    pool: PgPool,
}

impl ContradictionObservationStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn refresh_deterministic_observations(
        &self,
        limit: i64,
    ) -> Result<usize, ConsistencyError> {
        let limit = validate_refresh_limit(limit);
        let facts = self.active_person_fact_claims(limit).await?;
        let messages = self.recent_message_evidence(limit).await?;
        let channel_messages = self.recent_channel_message_evidence(limit).await?;
        let documents = self.recent_document_evidence(limit).await?;
        let meeting_notes = self.recent_meeting_note_evidence(limit).await?;
        let call_transcripts = self.recent_call_transcript_evidence(limit).await?;
        let mut count = 0usize;

        for fact in &facts {
            for message in &messages {
                if fact.email_address != message.sender_email_address {
                    continue;
                }

                let accepted = AcceptedClaim {
                    subject_id: fact.person_id.clone(),
                    claim_type: fact.claim_type.clone(),
                    value: fact.value.clone(),
                    source_kind: ContradictionSourceKind::Memory,
                    source_id: fact.fact_id.clone(),
                    confidence: fact.confidence,
                };
                let evidence = EvidenceClaimExtractionInput {
                    subject_id: fact.person_id.clone(),
                    source_kind: ContradictionSourceKind::Communication,
                    source_id: message.message_id.clone(),
                    text: message.text.clone(),
                    confidence: STRUCTURED_EVIDENCE_CLAIM_CONFIDENCE,
                };
                let observations =
                    ConsistencyEngine::detect_evidence_contradictions(&[accepted], &[evidence])?;

                for observation in observations {
                    self.upsert(&observation).await?;
                    count += 1;
                }
            }

            for message in &channel_messages {
                if message.person_id != fact.person_id {
                    continue;
                }

                let accepted = AcceptedClaim {
                    subject_id: fact.person_id.clone(),
                    claim_type: fact.claim_type.clone(),
                    value: fact.value.clone(),
                    source_kind: ContradictionSourceKind::Memory,
                    source_id: fact.fact_id.clone(),
                    confidence: fact.confidence,
                };
                let evidence = EvidenceClaimExtractionInput {
                    subject_id: fact.person_id.clone(),
                    source_kind: ContradictionSourceKind::Communication,
                    source_id: message.message_id.clone(),
                    text: message.text.clone(),
                    confidence: STRUCTURED_EVIDENCE_CLAIM_CONFIDENCE,
                };
                let observations =
                    ConsistencyEngine::detect_evidence_contradictions(&[accepted], &[evidence])?;

                for observation in observations {
                    self.upsert(&observation).await?;
                    count += 1;
                }
            }

            for document in &documents {
                if !document.references_email_address(&fact.email_address) {
                    continue;
                }

                let accepted = AcceptedClaim {
                    subject_id: fact.person_id.clone(),
                    claim_type: fact.claim_type.clone(),
                    value: fact.value.clone(),
                    source_kind: ContradictionSourceKind::Memory,
                    source_id: fact.fact_id.clone(),
                    confidence: fact.confidence,
                };
                let evidence = EvidenceClaimExtractionInput {
                    subject_id: fact.person_id.clone(),
                    source_kind: ContradictionSourceKind::Document,
                    source_id: document.document_id.clone(),
                    text: document.text.clone(),
                    confidence: STRUCTURED_EVIDENCE_CLAIM_CONFIDENCE,
                };
                let observations =
                    ConsistencyEngine::detect_evidence_contradictions(&[accepted], &[evidence])?;

                for observation in observations {
                    self.upsert(&observation).await?;
                    count += 1;
                }
            }

            for note in &meeting_notes {
                if note.person_id != fact.person_id {
                    continue;
                }

                let accepted = AcceptedClaim {
                    subject_id: fact.person_id.clone(),
                    claim_type: fact.claim_type.clone(),
                    value: fact.value.clone(),
                    source_kind: ContradictionSourceKind::Memory,
                    source_id: fact.fact_id.clone(),
                    confidence: fact.confidence,
                };
                let evidence = EvidenceClaimExtractionInput {
                    subject_id: fact.person_id.clone(),
                    source_kind: ContradictionSourceKind::Event,
                    source_id: note.note_id.clone(),
                    text: note.text.clone(),
                    confidence: STRUCTURED_EVIDENCE_CLAIM_CONFIDENCE,
                };
                let observations =
                    ConsistencyEngine::detect_evidence_contradictions(&[accepted], &[evidence])?;

                for observation in observations {
                    self.upsert(&observation).await?;
                    count += 1;
                }
            }

            for transcript in &call_transcripts {
                if transcript.person_id != fact.person_id {
                    continue;
                }

                let accepted = AcceptedClaim {
                    subject_id: fact.person_id.clone(),
                    claim_type: fact.claim_type.clone(),
                    value: fact.value.clone(),
                    source_kind: ContradictionSourceKind::Memory,
                    source_id: fact.fact_id.clone(),
                    confidence: fact.confidence,
                };
                let evidence = EvidenceClaimExtractionInput {
                    subject_id: fact.person_id.clone(),
                    source_kind: ContradictionSourceKind::Communication,
                    source_id: transcript.transcript_id.clone(),
                    text: transcript.text.clone(),
                    confidence: STRUCTURED_EVIDENCE_CLAIM_CONFIDENCE,
                };
                let observations =
                    ConsistencyEngine::detect_evidence_contradictions(&[accepted], &[evidence])?;

                for observation in observations {
                    self.upsert(&observation).await?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    pub async fn upsert(
        &self,
        observation: &NewContradictionObservation,
    ) -> Result<ContradictionObservation, ConsistencyError> {
        observation.validate()?;
        let observation_id = contradiction_observation_id(observation);
        let row = sqlx::query(
            r#"
            INSERT INTO contradiction_observations (
                observation_id,
                old_source_kind,
                old_source_id,
                new_source_kind,
                new_source_id,
                affected_entities,
                conflict_type,
                old_claim,
                new_claim,
                confidence,
                severity,
                review_state,
                metadata
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                $8,
                $9,
                CAST($10 AS NUMERIC(5,4)),
                $11,
                $12,
                $13
            )
            ON CONFLICT (observation_id)
            DO UPDATE SET
                affected_entities = EXCLUDED.affected_entities,
                old_claim = EXCLUDED.old_claim,
                new_claim = EXCLUDED.new_claim,
                confidence = EXCLUDED.confidence,
                severity = EXCLUDED.severity,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            RETURNING
                observation_id,
                old_source_kind,
                old_source_id,
                new_source_kind,
                new_source_id,
                affected_entities,
                conflict_type,
                old_claim,
                new_claim,
                confidence::float8 AS confidence,
                severity,
                review_state,
                metadata,
                reviewed_by,
                reviewed_at,
                resolution,
                created_at,
                updated_at
            "#,
        )
        .bind(&observation_id)
        .bind(observation.old_source_kind.as_str())
        .bind(&observation.old_source_id)
        .bind(observation.new_source_kind.as_str())
        .bind(&observation.new_source_id)
        .bind(&observation.affected_entities)
        .bind(&observation.conflict_type)
        .bind(&observation.old_claim)
        .bind(&observation.new_claim)
        .bind(observation.confidence)
        .bind(observation.severity.as_str())
        .bind(observation.review_state.as_str())
        .bind(&observation.metadata)
        .fetch_one(&self.pool)
        .await?;

        row_to_observation(row)
    }

    pub async fn list_open(
        &self,
        limit: i64,
    ) -> Result<Vec<ContradictionObservation>, ConsistencyError> {
        let rows = sqlx::query(
            r#"
            SELECT
                observation_id,
                old_source_kind,
                old_source_id,
                new_source_kind,
                new_source_id,
                affected_entities,
                conflict_type,
                old_claim,
                new_claim,
                confidence::float8 AS confidence,
                severity,
                review_state,
                metadata,
                reviewed_by,
                reviewed_at,
                resolution,
                created_at,
                updated_at
            FROM contradiction_observations
            WHERE review_state = 'suggested'
            ORDER BY updated_at DESC, observation_id ASC
            LIMIT $1
            "#,
        )
        .bind(limit.clamp(1, 100))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_observation).collect()
    }

    pub async fn set_review_state(
        &self,
        observation_id: &str,
        review_state: ContradictionReviewState,
        reviewed_by: &str,
        resolution: Option<&str>,
    ) -> Result<ContradictionObservation, ConsistencyError> {
        validate_non_empty("observation_id", observation_id)?;
        validate_non_empty("reviewed_by", reviewed_by)?;
        let row = sqlx::query(
            r#"
            UPDATE contradiction_observations
            SET review_state = $2,
                reviewed_by = $3,
                reviewed_at = now(),
                resolution = $4,
                updated_at = now()
            WHERE observation_id = $1
            RETURNING
                observation_id,
                old_source_kind,
                old_source_id,
                new_source_kind,
                new_source_id,
                affected_entities,
                conflict_type,
                old_claim,
                new_claim,
                confidence::float8 AS confidence,
                severity,
                review_state,
                metadata,
                reviewed_by,
                reviewed_at,
                resolution,
                created_at,
                updated_at
            "#,
        )
        .bind(observation_id)
        .bind(review_state.as_str())
        .bind(reviewed_by)
        .bind(resolution)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Err(ConsistencyError::ObservationNotFound(
                observation_id.to_owned(),
            ));
        };

        row_to_observation(row)
    }

    async fn active_person_fact_claims(
        &self,
        limit: i64,
    ) -> Result<Vec<ActivePersonFactClaim>, ConsistencyError> {
        let rows = sqlx::query(
            r#"
            SELECT
                fact.id::text AS fact_id,
                fact.person_id,
                fact.fact_type,
                fact.value,
                fact.confidence::float8 AS confidence,
                person.email_address
            FROM person_facts fact
            JOIN persons person ON person.person_id = fact.person_id
            WHERE fact.is_active = true
              AND length(trim(person.email_address)) > 0
            ORDER BY fact.updated_at DESC, fact.id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_active_person_fact_claim)
            .collect()
    }

    async fn recent_message_evidence(
        &self,
        limit: i64,
    ) -> Result<Vec<MessageEvidence>, ConsistencyError> {
        let rows = sqlx::query(
            r#"
            SELECT
                message_id,
                sender,
                subject,
                body_text
            FROM communication_messages
            ORDER BY COALESCE(occurred_at, projected_at) DESC, message_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_message_evidence).collect()
    }

    async fn recent_channel_message_evidence(
        &self,
        limit: i64,
    ) -> Result<Vec<ChannelMessageEvidence>, ConsistencyError> {
        let rows = sqlx::query(
            r#"
            SELECT
                message.message_id,
                identity.person_id,
                message.subject,
                message.body_text
            FROM communication_messages message
            JOIN person_identities identity
              ON identity.status = 'active'
             AND identity.identity_value = message.message_metadata->>'sender_id'
             AND (
                    (
                        message.channel_kind IN ('telegram_user', 'telegram_bot')
                    AND identity.identity_type = 'telegram'
                    )
                 OR (
                        message.channel_kind = 'whatsapp_web'
                    AND identity.identity_type = 'whatsapp'
                    )
                 )
            WHERE message.channel_kind IN ('telegram_user', 'telegram_bot', 'whatsapp_web')
              AND length(trim(message.body_text)) > 0
              AND length(trim(message.message_metadata->>'sender_id')) > 0
            ORDER BY COALESCE(message.occurred_at, message.projected_at) DESC, message.message_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_channel_message_evidence)
            .collect()
    }

    async fn recent_document_evidence(
        &self,
        limit: i64,
    ) -> Result<Vec<DocumentEvidence>, ConsistencyError> {
        let rows = sqlx::query(
            r#"
            SELECT
                document_id,
                title,
                extracted_text
            FROM documents
            WHERE length(trim(extracted_text)) > 0
            ORDER BY imported_at DESC, document_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_document_evidence).collect()
    }

    async fn recent_meeting_note_evidence(
        &self,
        limit: i64,
    ) -> Result<Vec<MeetingNoteEvidence>, ConsistencyError> {
        let rows = sqlx::query(
            r#"
            SELECT
                note.id::text AS note_id,
                participant.person_id,
                event.title,
                note.content
            FROM meeting_notes note
            JOIN calendar_events event ON event.event_id = note.event_id
            JOIN event_participants participant ON participant.event_id = note.event_id
            WHERE participant.person_id IS NOT NULL
              AND length(trim(note.content)) > 0
            ORDER BY note.updated_at DESC, note.id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_meeting_note_evidence).collect()
    }

    async fn recent_call_transcript_evidence(
        &self,
        limit: i64,
    ) -> Result<Vec<CallTranscriptEvidence>, ConsistencyError> {
        let rows = sqlx::query(
            r#"
            SELECT
                transcript.transcript_id,
                identity.person_id,
                transcript.transcript_text
            FROM call_transcripts transcript
            JOIN person_identities identity
              ON identity.identity_type = 'telegram'
             AND identity.status = 'active'
             AND identity.identity_value = transcript.provider_chat_id
            WHERE transcript.transcript_status = 'succeeded'
              AND length(trim(transcript.transcript_text)) > 0
            ORDER BY transcript.updated_at DESC, transcript.transcript_id
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(row_to_call_transcript_evidence)
            .collect()
    }
}

pub struct ConsistencyEngine;

impl ConsistencyEngine {
    pub fn detect_claim_contradictions(
        accepted_claims: &[AcceptedClaim],
        new_claims: &[NewEvidenceClaim],
    ) -> Result<Vec<NewContradictionObservation>, ConsistencyError> {
        Self::detect_claim_contradictions_with_detector(
            accepted_claims,
            new_claims,
            "structured_claim",
        )
    }

    pub fn extract_evidence_claims(
        input: &EvidenceClaimExtractionInput,
    ) -> Result<Vec<NewEvidenceClaim>, ConsistencyError> {
        input.validate()?;

        let mut claims = Vec::new();
        for line in input.text.lines() {
            let Some((claim_type, value)) = parse_evidence_claim_line(line) else {
                continue;
            };

            claims.push(NewEvidenceClaim {
                subject_id: input.subject_id.clone(),
                claim_type,
                value,
                source_kind: input.source_kind,
                source_id: input.source_id.clone(),
                confidence: input.confidence,
            });
        }

        Ok(claims)
    }

    pub fn detect_evidence_contradictions(
        accepted_claims: &[AcceptedClaim],
        evidence_inputs: &[EvidenceClaimExtractionInput],
    ) -> Result<Vec<NewContradictionObservation>, ConsistencyError> {
        let mut extracted_claims = Vec::new();
        for input in evidence_inputs {
            extracted_claims.extend(Self::extract_evidence_claims(input)?);
        }

        Self::detect_claim_contradictions_with_detector(
            accepted_claims,
            &extracted_claims,
            "structured_evidence_claim",
        )
    }

    fn detect_claim_contradictions_with_detector(
        accepted_claims: &[AcceptedClaim],
        new_claims: &[NewEvidenceClaim],
        detector: &str,
    ) -> Result<Vec<NewContradictionObservation>, ConsistencyError> {
        for claim in accepted_claims {
            claim.validate()?;
        }
        for claim in new_claims {
            claim.validate()?;
        }

        let mut observations = Vec::new();
        for accepted in accepted_claims {
            for new_claim in new_claims {
                if accepted.subject_id != new_claim.subject_id {
                    continue;
                }
                if accepted.claim_type.trim() != new_claim.claim_type.trim() {
                    continue;
                }
                if normalize_claim_value(&accepted.value) == normalize_claim_value(&new_claim.value)
                {
                    continue;
                }

                let confidence = accepted.confidence.min(new_claim.confidence);
                observations.push(NewContradictionObservation {
                    old_source_kind: accepted.source_kind,
                    old_source_id: accepted.source_id.clone(),
                    new_source_kind: new_claim.source_kind,
                    new_source_id: new_claim.source_id.clone(),
                    affected_entities: json!([{
                        "entity_kind": "subject",
                        "entity_id": accepted.subject_id,
                    }]),
                    conflict_type: "direct_contradiction".to_owned(),
                    old_claim: claim_text(&accepted.claim_type, &accepted.value),
                    new_claim: claim_text(&new_claim.claim_type, &new_claim.value),
                    confidence,
                    severity: severity_for_confidence(confidence),
                    review_state: ContradictionReviewState::Suggested,
                    metadata: contradiction_metadata(detector, accepted, new_claim),
                });
            }
        }

        Ok(observations)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionSourceKind {
    Communication,
    Document,
    Event,
    Memory,
    Knowledge,
    Decision,
    Obligation,
    Task,
    Relationship,
    RawRecord,
}

impl ContradictionSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Communication => "communication",
            Self::Document => "document",
            Self::Event => "event",
            Self::Memory => "memory",
            Self::Knowledge => "knowledge",
            Self::Decision => "decision",
            Self::Obligation => "obligation",
            Self::Task => "task",
            Self::Relationship => "relationship",
            Self::RawRecord => "raw_record",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ContradictionSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionReviewState {
    Suggested,
    UserConfirmed,
    UserRejected,
}

impl ContradictionReviewState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Suggested => "suggested",
            Self::UserConfirmed => "user_confirmed",
            Self::UserRejected => "user_rejected",
        }
    }

    pub fn parse(value: impl AsRef<str>) -> Result<Self, ConsistencyError> {
        let value = value.as_ref().trim();
        match value {
            "suggested" => Ok(Self::Suggested),
            "user_confirmed" => Ok(Self::UserConfirmed),
            "user_rejected" => Ok(Self::UserRejected),
            _ => Err(ConsistencyError::UnknownReviewState(value.to_owned())),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AcceptedClaim {
    pub subject_id: String,
    pub claim_type: String,
    pub value: String,
    pub source_kind: ContradictionSourceKind,
    pub source_id: String,
    pub confidence: f64,
}

impl AcceptedClaim {
    fn validate(&self) -> Result<(), ConsistencyError> {
        validate_non_empty("subject_id", &self.subject_id)?;
        validate_non_empty("claim_type", &self.claim_type)?;
        validate_non_empty("value", &self.value)?;
        validate_non_empty("source_id", &self.source_id)?;
        validate_confidence(self.confidence)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewEvidenceClaim {
    pub subject_id: String,
    pub claim_type: String,
    pub value: String,
    pub source_kind: ContradictionSourceKind,
    pub source_id: String,
    pub confidence: f64,
}

impl NewEvidenceClaim {
    fn validate(&self) -> Result<(), ConsistencyError> {
        validate_non_empty("subject_id", &self.subject_id)?;
        validate_non_empty("claim_type", &self.claim_type)?;
        validate_non_empty("value", &self.value)?;
        validate_non_empty("source_id", &self.source_id)?;
        validate_confidence(self.confidence)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EvidenceClaimExtractionInput {
    pub subject_id: String,
    pub source_kind: ContradictionSourceKind,
    pub source_id: String,
    pub text: String,
    pub confidence: f64,
}

impl EvidenceClaimExtractionInput {
    fn validate(&self) -> Result<(), ConsistencyError> {
        validate_non_empty("subject_id", &self.subject_id)?;
        validate_non_empty("source_id", &self.source_id)?;
        validate_non_empty("text", &self.text)?;
        validate_confidence(self.confidence)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewContradictionObservation {
    pub old_source_kind: ContradictionSourceKind,
    pub old_source_id: String,
    pub new_source_kind: ContradictionSourceKind,
    pub new_source_id: String,
    pub affected_entities: Value,
    pub conflict_type: String,
    pub old_claim: String,
    pub new_claim: String,
    pub confidence: f64,
    pub severity: ContradictionSeverity,
    pub review_state: ContradictionReviewState,
    pub metadata: Value,
}

impl NewContradictionObservation {
    pub fn validate(&self) -> Result<(), ConsistencyError> {
        validate_non_empty("old_source_id", &self.old_source_id)?;
        validate_non_empty("new_source_id", &self.new_source_id)?;
        validate_non_empty("conflict_type", &self.conflict_type)?;
        validate_non_empty("old_claim", &self.old_claim)?;
        validate_non_empty("new_claim", &self.new_claim)?;
        validate_confidence(self.confidence)?;
        validate_json_array_or_object("affected_entities", &self.affected_entities)?;
        validate_json_object("metadata", &self.metadata)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ContradictionObservation {
    pub observation_id: String,
    pub old_source_kind: ContradictionSourceKind,
    pub old_source_id: String,
    pub new_source_kind: ContradictionSourceKind,
    pub new_source_id: String,
    pub affected_entities: Value,
    pub conflict_type: String,
    pub old_claim: String,
    pub new_claim: String,
    pub confidence: f64,
    pub severity: ContradictionSeverity,
    pub review_state: ContradictionReviewState,
    pub metadata: Value,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum ConsistencyError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),

    #[error("confidence must be between 0.0 and 1.0: {0}")]
    InvalidConfidence(f64),

    #[error("{0} must be a JSON object")]
    InvalidJsonObject(&'static str),

    #[error("{0} must be a JSON array or object")]
    InvalidJsonArrayOrObject(&'static str),

    #[error("unknown contradiction source kind stored in database: {0}")]
    UnknownSourceKind(String),

    #[error("unknown contradiction severity stored in database: {0}")]
    UnknownSeverity(String),

    #[error("unknown contradiction review state stored in database: {0}")]
    UnknownReviewState(String),

    #[error("contradiction observation not found: {0}")]
    ObservationNotFound(String),
}

pub fn contradiction_observation_id(observation: &NewContradictionObservation) -> String {
    format!(
        "contradiction:v1:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        observation.old_source_kind.as_str().len(),
        observation.old_source_kind.as_str(),
        observation.old_source_id.len(),
        observation.old_source_id,
        observation.new_source_kind.as_str().len(),
        observation.new_source_kind.as_str(),
        observation.new_source_id.len(),
        observation.new_source_id,
        observation.conflict_type.len(),
        observation.conflict_type
    )
}

fn claim_text(claim_type: &str, value: &str) -> String {
    let claim_type = claim_type.trim();
    let value = value.trim();
    format!("{claim_type}={value}")
}

fn normalize_claim_value(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn severity_for_confidence(confidence: f64) -> ContradictionSeverity {
    if confidence >= 0.95 {
        ContradictionSeverity::Critical
    } else if confidence >= 0.9 {
        ContradictionSeverity::High
    } else if confidence >= 0.7 {
        ContradictionSeverity::Medium
    } else {
        ContradictionSeverity::Low
    }
}

fn contradiction_metadata(
    detector: &str,
    accepted: &AcceptedClaim,
    new_claim: &NewEvidenceClaim,
) -> Value {
    if detector == "structured_evidence_claim" {
        json!({
            "detector": detector,
            "claim_type": accepted.claim_type.trim(),
            "source_kind": new_claim.source_kind.as_str(),
        })
    } else {
        json!({
            "detector": detector,
            "claim_type": accepted.claim_type.trim(),
        })
    }
}

fn parse_evidence_claim_line(line: &str) -> Option<(String, String)> {
    parse_structured_claim_line(line).or_else(|| parse_natural_language_claim_line(line))
}

fn parse_structured_claim_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let delimiter_index = match (trimmed.find(':'), trimmed.find('=')) {
        (Some(colon), Some(equals)) => Some(colon.min(equals)),
        (Some(colon), None) => Some(colon),
        (None, Some(equals)) => Some(equals),
        (None, None) => None,
    }?;

    let raw_claim_type = trimmed[..delimiter_index].trim();
    let value = trimmed[delimiter_index + 1..].trim();
    if raw_claim_type.is_empty() || value.is_empty() {
        return None;
    }

    let claim_type = raw_claim_type
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
        .to_lowercase();
    if claim_type.is_empty()
        || !claim_type
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
        || !is_supported_deterministic_claim_type(&claim_type)
    {
        return None;
    }

    Some((claim_type, normalize_extracted_claim_value(value)?))
}

fn parse_natural_language_claim_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();
    for prefix in [
        "i am now in ",
        "i am in ",
        "i'm now in ",
        "i'm in ",
        "location is ",
        "location changed to ",
        "location became ",
    ] {
        if let Some(value) = value_after_case_insensitive_pattern(trimmed, &lower, prefix) {
            return Some(("location".to_owned(), value));
        }
    }

    for prefix in ["status is ", "status changed to ", "status became "] {
        if let Some(value) = value_after_case_insensitive_pattern(trimmed, &lower, prefix) {
            return Some(("status".to_owned(), value));
        }
    }

    None
}

fn value_after_case_insensitive_pattern(
    original: &str,
    lower: &str,
    pattern: &str,
) -> Option<String> {
    let start = lower.find(pattern)? + pattern.len();
    normalize_extracted_claim_value(&original[start..])
}

fn normalize_extracted_claim_value(value: &str) -> Option<String> {
    let value = value
        .trim()
        .trim_matches(|character: char| {
            matches!(
                character,
                '.' | ',' | ';' | ':' | '!' | '?' | '"' | '\'' | ')' | '('
            )
        })
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if value.is_empty() { None } else { Some(value) }
}

fn is_supported_deterministic_claim_type(claim_type: &str) -> bool {
    matches!(claim_type, "location" | "status")
}

fn row_to_observation(row: PgRow) -> Result<ContradictionObservation, ConsistencyError> {
    Ok(ContradictionObservation {
        observation_id: row.try_get("observation_id")?,
        old_source_kind: parse_source_kind(row.try_get("old_source_kind")?)?,
        old_source_id: row.try_get("old_source_id")?,
        new_source_kind: parse_source_kind(row.try_get("new_source_kind")?)?,
        new_source_id: row.try_get("new_source_id")?,
        affected_entities: row.try_get("affected_entities")?,
        conflict_type: row.try_get("conflict_type")?,
        old_claim: row.try_get("old_claim")?,
        new_claim: row.try_get("new_claim")?,
        confidence: row.try_get("confidence")?,
        severity: parse_severity(row.try_get("severity")?)?,
        review_state: parse_review_state(row.try_get("review_state")?)?,
        metadata: row.try_get("metadata")?,
        reviewed_by: row.try_get("reviewed_by")?,
        reviewed_at: row.try_get("reviewed_at")?,
        resolution: row.try_get("resolution")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

struct ActivePersonFactClaim {
    fact_id: String,
    person_id: String,
    claim_type: String,
    value: String,
    confidence: f64,
    email_address: String,
}

fn row_to_active_person_fact_claim(row: PgRow) -> Result<ActivePersonFactClaim, ConsistencyError> {
    Ok(ActivePersonFactClaim {
        fact_id: row.try_get("fact_id")?,
        person_id: row.try_get("person_id")?,
        claim_type: row.try_get("fact_type")?,
        value: row.try_get("value")?,
        confidence: row.try_get("confidence")?,
        email_address: normalize_email_address_for_match(
            row.try_get::<String, _>("email_address")?.as_str(),
        ),
    })
}

struct MessageEvidence {
    message_id: String,
    sender_email_address: String,
    text: String,
}

fn row_to_message_evidence(row: PgRow) -> Result<MessageEvidence, ConsistencyError> {
    let subject = row.try_get::<String, _>("subject")?;
    let body_text = row.try_get::<String, _>("body_text")?;

    Ok(MessageEvidence {
        message_id: row.try_get("message_id")?,
        sender_email_address: normalize_email_address_for_match(
            row.try_get::<String, _>("sender")?.as_str(),
        ),
        text: format!("{subject}\n{body_text}"),
    })
}

struct ChannelMessageEvidence {
    message_id: String,
    person_id: String,
    text: String,
}

fn row_to_channel_message_evidence(row: PgRow) -> Result<ChannelMessageEvidence, ConsistencyError> {
    let subject = row.try_get::<String, _>("subject")?;
    let body_text = row.try_get::<String, _>("body_text")?;

    Ok(ChannelMessageEvidence {
        message_id: row.try_get("message_id")?,
        person_id: row.try_get("person_id")?,
        text: format!("{subject}\n{body_text}"),
    })
}

struct DocumentEvidence {
    document_id: String,
    normalized_text: String,
    text: String,
}

impl DocumentEvidence {
    fn references_email_address(&self, email_address: &str) -> bool {
        self.normalized_text.contains(email_address)
    }
}

fn row_to_document_evidence(row: PgRow) -> Result<DocumentEvidence, ConsistencyError> {
    let title = row.try_get::<String, _>("title")?;
    let extracted_text = row.try_get::<String, _>("extracted_text")?;
    let text = format!("{title}\n{extracted_text}");

    Ok(DocumentEvidence {
        document_id: row.try_get("document_id")?,
        normalized_text: text.to_ascii_lowercase(),
        text,
    })
}

struct MeetingNoteEvidence {
    note_id: String,
    person_id: String,
    text: String,
}

fn row_to_meeting_note_evidence(row: PgRow) -> Result<MeetingNoteEvidence, ConsistencyError> {
    let title = row.try_get::<String, _>("title")?;
    let content = row.try_get::<String, _>("content")?;

    Ok(MeetingNoteEvidence {
        note_id: row.try_get("note_id")?,
        person_id: row.try_get("person_id")?,
        text: format!("{title}\n{content}"),
    })
}

struct CallTranscriptEvidence {
    transcript_id: String,
    person_id: String,
    text: String,
}

fn row_to_call_transcript_evidence(row: PgRow) -> Result<CallTranscriptEvidence, ConsistencyError> {
    Ok(CallTranscriptEvidence {
        transcript_id: row.try_get("transcript_id")?,
        person_id: row.try_get("person_id")?,
        text: row.try_get("transcript_text")?,
    })
}

fn parse_source_kind(value: String) -> Result<ContradictionSourceKind, ConsistencyError> {
    match value.as_str() {
        "communication" => Ok(ContradictionSourceKind::Communication),
        "document" => Ok(ContradictionSourceKind::Document),
        "event" => Ok(ContradictionSourceKind::Event),
        "memory" => Ok(ContradictionSourceKind::Memory),
        "knowledge" => Ok(ContradictionSourceKind::Knowledge),
        "decision" => Ok(ContradictionSourceKind::Decision),
        "obligation" => Ok(ContradictionSourceKind::Obligation),
        "task" => Ok(ContradictionSourceKind::Task),
        "relationship" => Ok(ContradictionSourceKind::Relationship),
        "raw_record" => Ok(ContradictionSourceKind::RawRecord),
        _ => Err(ConsistencyError::UnknownSourceKind(value)),
    }
}

fn parse_severity(value: String) -> Result<ContradictionSeverity, ConsistencyError> {
    match value.as_str() {
        "low" => Ok(ContradictionSeverity::Low),
        "medium" => Ok(ContradictionSeverity::Medium),
        "high" => Ok(ContradictionSeverity::High),
        "critical" => Ok(ContradictionSeverity::Critical),
        _ => Err(ConsistencyError::UnknownSeverity(value)),
    }
}

fn parse_review_state(value: String) -> Result<ContradictionReviewState, ConsistencyError> {
    ContradictionReviewState::parse(value)
}

fn validate_non_empty(field_name: &'static str, value: &str) -> Result<(), ConsistencyError> {
    if value.trim().is_empty() {
        return Err(ConsistencyError::EmptyField(field_name));
    }

    Ok(())
}

fn validate_confidence(confidence: f64) -> Result<(), ConsistencyError> {
    if !(0.0..=1.0).contains(&confidence) {
        return Err(ConsistencyError::InvalidConfidence(confidence));
    }

    Ok(())
}

fn validate_json_object(field_name: &'static str, value: &Value) -> Result<(), ConsistencyError> {
    if !value.is_object() {
        return Err(ConsistencyError::InvalidJsonObject(field_name));
    }

    Ok(())
}

fn validate_json_array_or_object(
    field_name: &'static str,
    value: &Value,
) -> Result<(), ConsistencyError> {
    if !value.is_array() && !value.is_object() {
        return Err(ConsistencyError::InvalidJsonArrayOrObject(field_name));
    }

    Ok(())
}

fn validate_refresh_limit(limit: i64) -> i64 {
    limit.clamp(MIN_REFRESH_LIMIT, MAX_REFRESH_LIMIT)
}

fn normalize_email_address_for_match(email_address: &str) -> String {
    email_addr_spec(email_address).trim().to_ascii_lowercase()
}

fn email_addr_spec(value: &str) -> &str {
    let value = value.trim();
    if let Some((_, tail)) = value.rsplit_once('<') {
        if let Some((addr, _)) = tail.split_once('>') {
            return addr.trim();
        }
    }
    value.trim_matches('"')
}
