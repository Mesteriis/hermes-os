use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::postgres::PgPool;

use super::errors::CalendarCoreError;
use super::link_calendar_entity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventParticipant {
    pub id: String,
    pub event_id: String,
    pub person_id: Option<String>,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub response_status: String,
    pub source: String,
    pub organization_id: Option<String>,
    pub timezone: Option<String>,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EventParticipantStore {
    pool: PgPool,
}

impl EventParticipantStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, event_id: &str) -> Result<Vec<EventParticipant>, CalendarCoreError> {
        let rows = sqlx::query("SELECT id::text, event_id, person_id, email, display_name, role, response_status, source, organization_id, timezone, confidence, created_at FROM event_participants WHERE event_id=$1 ORDER BY role, email")
            .bind(event_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(EventParticipant {
                    id: r.try_get("id")?,
                    event_id: r.try_get("event_id")?,
                    person_id: r.try_get("person_id")?,
                    email: r.try_get("email")?,
                    display_name: r.try_get("display_name")?,
                    role: r.try_get("role")?,
                    response_status: r.try_get("response_status")?,
                    source: r.try_get("source")?,
                    organization_id: r.try_get("organization_id")?,
                    timezone: r.try_get("timezone")?,
                    confidence: f64::from(r.try_get::<f32, _>("confidence")?),
                    created_at: r.try_get("created_at")?,
                })
            })
            .collect()
    }

    pub async fn add(
        &self,
        event_id: &str,
        email: &str,
        display_name: Option<&str>,
        role: Option<&str>,
        person_id: Option<&str>,
        org_id: Option<&str>,
    ) -> Result<EventParticipant, CalendarCoreError> {
        self.add_with_source(
            event_id,
            email,
            display_name,
            role,
            person_id,
            org_id,
            "manual",
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_with_source(
        &self,
        event_id: &str,
        email: &str,
        display_name: Option<&str>,
        role: Option<&str>,
        person_id: Option<&str>,
        org_id: Option<&str>,
        source: &str,
    ) -> Result<EventParticipant, CalendarCoreError> {
        self.add_with_observation(
            event_id,
            email,
            display_name,
            role,
            person_id,
            org_id,
            source,
            None,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_with_observation(
        &self,
        event_id: &str,
        email: &str,
        display_name: Option<&str>,
        role: Option<&str>,
        person_id: Option<&str>,
        org_id: Option<&str>,
        source: &str,
        observation_id: Option<&str>,
    ) -> Result<EventParticipant, CalendarCoreError> {
        let row = sqlx::query("INSERT INTO event_participants (event_id, email, display_name, role, person_id, organization_id, source) VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING id::text, event_id, person_id, email, display_name, role, response_status, source, organization_id, timezone, confidence, created_at")
            .bind(event_id).bind(email).bind(display_name).bind(role.unwrap_or("attendee")).bind(person_id).bind(org_id).bind(source).fetch_one(&self.pool).await?;
        let participant = EventParticipant {
            id: row.try_get("id")?,
            event_id: row.try_get("event_id")?,
            person_id: row.try_get("person_id")?,
            email: row.try_get("email")?,
            display_name: row.try_get("display_name")?,
            role: row.try_get("role")?,
            response_status: row.try_get("response_status")?,
            source: row.try_get("source")?,
            organization_id: row.try_get("organization_id")?,
            timezone: row.try_get("timezone")?,
            confidence: f64::from(row.try_get::<f32, _>("confidence")?),
            created_at: row.try_get("created_at")?,
        };
        if let Some(observation_id) = observation_id.filter(|value| !value.is_empty()) {
            link_calendar_entity(
                &self.pool,
                observation_id,
                "event_participant",
                participant.id.clone(),
                Some(serde_json::json!({
                    "event_id": event_id,
                    "email": participant.email,
                    "role": participant.role,
                })),
            )
            .await?;
        }
        Ok(participant)
    }
}
