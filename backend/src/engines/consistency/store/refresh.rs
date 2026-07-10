use sqlx::postgres::PgPool;

use super::super::constants::STRUCTURED_EVIDENCE_CLAIM_CONFIDENCE;
use super::super::engine::ConsistencyEngine;
use super::super::errors::ConsistencyError;
use super::super::evidence::ActivePersonaFactClaim;
use super::super::models::{AcceptedClaim, ContradictionSourceKind, EvidenceClaimExtractionInput};
use super::super::validation::validate_refresh_limit;
use super::observations;
use super::sources;

pub(super) async fn refresh_deterministic_observations(
    pool: &PgPool,
    limit: i64,
) -> Result<usize, ConsistencyError> {
    let limit = validate_refresh_limit(limit);
    let facts = sources::active_persona_fact_claims(pool, limit).await?;
    let messages = sources::recent_message_evidence(pool, limit).await?;
    let channel_messages = sources::recent_channel_message_evidence(pool, limit).await?;
    let documents = sources::recent_document_evidence(pool, limit).await?;
    let meeting_notes = sources::recent_meeting_note_evidence(pool, limit).await?;
    let call_transcripts = sources::recent_call_transcript_evidence(pool, limit).await?;
    let mut count = 0usize;

    for fact in &facts {
        let accepted = accepted_claim(fact);

        for message in &messages {
            if fact.email_address != message.sender_email_address {
                continue;
            }

            count += detect_and_upsert(
                pool,
                &accepted,
                evidence_claim(
                    &fact.persona_id,
                    ContradictionSourceKind::Communication,
                    &message.message_id,
                    &message.text,
                ),
            )
            .await?;
        }

        for message in &channel_messages {
            if message.persona_id != fact.persona_id {
                continue;
            }

            count += detect_and_upsert(
                pool,
                &accepted,
                evidence_claim(
                    &fact.persona_id,
                    ContradictionSourceKind::Communication,
                    &message.message_id,
                    &message.text,
                ),
            )
            .await?;
        }

        for document in &documents {
            if !document.references_email_address(&fact.email_address) {
                continue;
            }

            count += detect_and_upsert(
                pool,
                &accepted,
                evidence_claim(
                    &fact.persona_id,
                    ContradictionSourceKind::Document,
                    &document.document_id,
                    &document.text,
                ),
            )
            .await?;
        }

        for note in &meeting_notes {
            if note.persona_id != fact.persona_id {
                continue;
            }

            count += detect_and_upsert(
                pool,
                &accepted,
                evidence_claim(
                    &fact.persona_id,
                    ContradictionSourceKind::Event,
                    &note.note_id,
                    &note.text,
                ),
            )
            .await?;
        }

        for transcript in &call_transcripts {
            if transcript.persona_id != fact.persona_id {
                continue;
            }

            count += detect_and_upsert(
                pool,
                &accepted,
                evidence_claim(
                    &fact.persona_id,
                    ContradictionSourceKind::Communication,
                    &transcript.transcript_id,
                    &transcript.text,
                ),
            )
            .await?;
        }
    }

    Ok(count)
}

fn accepted_claim(fact: &ActivePersonaFactClaim) -> AcceptedClaim {
    AcceptedClaim {
        subject_id: fact.persona_id.clone(),
        claim_type: fact.claim_type.clone(),
        value: fact.value.clone(),
        source_kind: ContradictionSourceKind::Memory,
        source_id: fact.fact_id.clone(),
        confidence: fact.confidence,
    }
}

fn evidence_claim(
    subject_id: &str,
    source_kind: ContradictionSourceKind,
    source_id: &str,
    text: &str,
) -> EvidenceClaimExtractionInput {
    EvidenceClaimExtractionInput {
        subject_id: subject_id.to_owned(),
        source_kind,
        source_id: source_id.to_owned(),
        text: text.to_owned(),
        confidence: STRUCTURED_EVIDENCE_CLAIM_CONFIDENCE,
    }
}

async fn detect_and_upsert(
    pool: &PgPool,
    accepted: &AcceptedClaim,
    evidence: EvidenceClaimExtractionInput,
) -> Result<usize, ConsistencyError> {
    let observations = ConsistencyEngine::detect_evidence_contradictions(
        std::slice::from_ref(accepted),
        std::slice::from_ref(&evidence),
    )?;
    let count = observations.len();

    for observation in observations {
        observations::upsert(pool, &observation).await?;
    }

    Ok(count)
}
