use sqlx::{Postgres, Transaction};

use super::meetings::errors::MeetingsError;
use super::meetings::models::{EventRecording, EventTranscript, MeetingOutcome};
use super::meetings::outcomes::MeetingOutcomeStore;
use super::meetings::recordings::EventRecordingStore;
use super::meetings::transcripts::EventTranscriptStore;

pub struct EventRecordingPort;

impl EventRecordingPort {
    pub(crate) async fn find_by_file_path_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        event_id: &str,
        file_path: &str,
    ) -> Result<Option<EventRecording>, MeetingsError> {
        EventRecordingStore::find_by_file_path_in_transaction(transaction, event_id, file_path)
            .await
    }

    pub(crate) async fn add_with_observation_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        event_id: &str,
        file_path: Option<&str>,
        source: Option<&str>,
        duration_seconds: Option<i32>,
        observation_id: Option<&str>,
    ) -> Result<EventRecording, MeetingsError> {
        EventRecordingStore::add_with_observation_in_transaction(
            transaction,
            event_id,
            file_path,
            source,
            duration_seconds,
            observation_id,
        )
        .await
    }

    pub(crate) async fn attach_transcript_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        recording_id: &str,
        transcript_id: &str,
        observation_id: Option<&str>,
    ) -> Result<EventRecording, MeetingsError> {
        EventRecordingStore::attach_transcript_in_transaction(
            transaction,
            recording_id,
            transcript_id,
            observation_id,
        )
        .await
    }
}

pub struct EventTranscriptPort;

impl EventTranscriptPort {
    pub(crate) async fn add_with_observation_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        event_id: &str,
        text: &str,
        language: Option<&str>,
        summary: Option<&str>,
        model: Option<&str>,
        observation_id: Option<&str>,
    ) -> Result<EventTranscript, MeetingsError> {
        EventTranscriptStore::add_with_observation_in_transaction(
            transaction,
            event_id,
            text,
            language,
            summary,
            model,
            observation_id,
        )
        .await
    }
}

pub struct MeetingOutcomePort;

impl MeetingOutcomePort {
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn add_with_observation_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        event_id: &str,
        outcome_type: &str,
        title: &str,
        description: Option<&str>,
        owner_person_id: Option<&str>,
        due_date: Option<chrono::DateTime<chrono::Utc>>,
        source: Option<&str>,
        observation_id: Option<&str>,
    ) -> Result<MeetingOutcome, MeetingsError> {
        MeetingOutcomeStore::add_with_observation_in_transaction(
            transaction,
            event_id,
            outcome_type,
            title,
            description,
            owner_person_id,
            due_date,
            source,
            observation_id,
        )
        .await
    }

    pub(crate) async fn set_linked_entity_id_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        outcome_id: &str,
        linked_entity_id: &str,
    ) -> Result<MeetingOutcome, MeetingsError> {
        MeetingOutcomeStore::set_linked_entity_id_in_transaction(
            transaction,
            outcome_id,
            linked_entity_id,
        )
        .await
    }
}

#[derive(Clone)]
pub struct EventParticipantPort(super::core::participants::EventParticipantStore);

impl EventParticipantPort {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self(super::core::participants::EventParticipantStore::new(pool))
    }

    pub async fn list(
        &self,
        event_id: &str,
    ) -> Result<
        Vec<super::core::participants::EventParticipant>,
        super::core::errors::CalendarCoreError,
    > {
        self.0.list(event_id).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_with_observation(
        &self,
        event_id: &str,
        email: &str,
        display_name: Option<&str>,
        role: Option<&str>,
        persona_id: Option<&str>,
        org_id: Option<&str>,
        source: &str,
        observation_id: Option<&str>,
    ) -> Result<super::core::participants::EventParticipant, super::core::errors::CalendarCoreError>
    {
        self.0
            .add_with_observation(
                event_id,
                email,
                display_name,
                role,
                persona_id,
                org_id,
                source,
                observation_id,
            )
            .await
    }
}

#[derive(Clone)]
pub struct EventRelationPort(super::core::relations::EventRelationStore);

impl EventRelationPort {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self(super::core::relations::EventRelationStore::new(pool))
    }

    pub async fn link_with_observation(
        &self,
        event_id: &str,
        entity_type: &str,
        entity_id: &str,
        relation_type: &str,
        source: &str,
        observation_id: Option<&str>,
    ) -> Result<super::core::relations::EventRelation, super::core::errors::CalendarCoreError> {
        self.0
            .link_with_observation(
                event_id,
                entity_type,
                entity_id,
                relation_type,
                source,
                observation_id,
            )
            .await
    }
}

#[derive(Clone)]
pub struct CalendarEventQueryPort(super::events::event_store::CalendarEventStore);

impl CalendarEventQueryPort {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self(super::events::event_store::CalendarEventStore::new(pool))
    }

    pub async fn find_zoom_conference_match(
        &self,
        join_url: Option<&str>,
        meeting_id: &str,
        started_at: Option<chrono::DateTime<chrono::Utc>>,
        ended_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Option<super::events::models::CalendarEvent>, super::events::errors::CalendarError>
    {
        self.0
            .find_zoom_conference_match(join_url, meeting_id, started_at, ended_at)
            .await
    }

    pub async fn find_yandex_telemost_conference_match(
        &self,
        join_url: Option<&str>,
        conference_id: &str,
    ) -> Result<Option<super::events::models::CalendarEvent>, super::events::errors::CalendarError>
    {
        self.0
            .find_yandex_telemost_conference_match(join_url, conference_id)
            .await
    }

    pub(crate) async fn observation_id_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        event_id: &str,
    ) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar("SELECT observation_id FROM calendar_events WHERE event_id = $1")
            .bind(event_id)
            .fetch_optional(&mut **transaction)
            .await
    }
}
