use sqlx::postgres::PgPool;

use super::super::errors::ConsistencyError;
use super::super::evidence::{
    ActivePersonFactClaim, CallTranscriptEvidence, ChannelMessageEvidence, DocumentEvidence,
    MeetingNoteEvidence, MessageEvidence, row_to_active_person_fact_claim,
    row_to_call_transcript_evidence, row_to_channel_message_evidence, row_to_document_evidence,
    row_to_meeting_note_evidence, row_to_message_evidence,
};

pub(super) async fn active_person_fact_claims(
    pool: &PgPool,
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
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_active_person_fact_claim)
        .collect()
}

pub(super) async fn recent_message_evidence(
    pool: &PgPool,
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
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_message_evidence).collect()
}

pub(super) async fn recent_channel_message_evidence(
    pool: &PgPool,
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
         AND (
                (
                    message.channel_kind IN ('telegram_user', 'telegram_bot')
                AND identity.identity_type = 'telegram'
                AND identity.identity_value = message.message_metadata->>'sender_id'
                )
             OR (
                    message.channel_kind IN ('whatsapp_web', 'whatsapp_business_cloud')
                AND identity.identity_type = 'whatsapp'
                AND identity.identity_value = message.message_metadata->>'sender_id'
                )
             OR (
                    message.channel_kind = 'zulip'
                AND identity.identity_type = 'zulip'
                AND identity.identity_value = COALESCE(
                    NULLIF(trim(message.message_metadata->>'sender_id'), ''),
                    NULLIF(trim(message.message_metadata->>'sender_email'), '')
                )
                )
             )
        WHERE message.channel_kind IN (
            'telegram_user',
            'telegram_bot',
            'whatsapp_web',
            'whatsapp_business_cloud',
            'zulip'
        )
          AND length(trim(message.body_text)) > 0
          AND (
                length(trim(message.message_metadata->>'sender_id')) > 0
             OR (
                    message.channel_kind = 'zulip'
                AND length(trim(message.message_metadata->>'sender_email')) > 0
                )
          )
        ORDER BY COALESCE(message.occurred_at, message.projected_at) DESC, message.message_id
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_channel_message_evidence)
        .collect()
}

pub(super) async fn recent_document_evidence(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<DocumentEvidence>, ConsistencyError> {
    let rows = sqlx::query(
        r#"
        SELECT
            document_id,
            observation_id,
            title,
            extracted_text
        FROM documents
        WHERE length(trim(extracted_text)) > 0
        ORDER BY imported_at DESC, document_id
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_document_evidence).collect()
}

pub(super) async fn recent_meeting_note_evidence(
    pool: &PgPool,
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
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(row_to_meeting_note_evidence).collect()
}

pub(super) async fn recent_call_transcript_evidence(
    pool: &PgPool,
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
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(row_to_call_transcript_evidence)
        .collect()
}
