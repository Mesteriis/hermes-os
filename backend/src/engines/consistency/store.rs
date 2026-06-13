use sqlx::postgres::PgPool;

use super::constants::STRUCTURED_EVIDENCE_CLAIM_CONFIDENCE;
use super::engine::ConsistencyEngine;
use super::errors::ConsistencyError;
use super::evidence::{
    ActivePersonFactClaim, CallTranscriptEvidence, ChannelMessageEvidence, DocumentEvidence,
    MeetingNoteEvidence, MessageEvidence, row_to_active_person_fact_claim,
    row_to_call_transcript_evidence, row_to_channel_message_evidence, row_to_document_evidence,
    row_to_meeting_note_evidence, row_to_message_evidence,
};
use super::helpers::contradiction_observation_id;
use super::models::{
    AcceptedClaim, ContradictionObservation, ContradictionReviewState, ContradictionSourceKind,
    EvidenceClaimExtractionInput, NewContradictionObservation,
};
use super::rows::row_to_observation;
use super::validation::{validate_non_empty, validate_refresh_limit};

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
