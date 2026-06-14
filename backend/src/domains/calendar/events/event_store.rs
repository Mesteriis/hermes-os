use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use super::errors::CalendarError;
use super::models::{CalendarEvent, CalendarEventUpdate, NewCalendarEvent};
use super::queries::CalendarEventListQuery;
use super::rows::row_to_event;

#[derive(Clone)]
pub struct CalendarEventStore {
    pool: PgPool,
}

impl CalendarEventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: &NewCalendarEvent) -> Result<CalendarEvent, CalendarError> {
        let mut transaction = self.pool.begin().await?;
        let event = Self::create_in_transaction(&mut transaction, req).await?;
        transaction.commit().await?;
        Ok(event)
    }

    pub(crate) async fn create_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        req: &NewCalendarEvent,
    ) -> Result<CalendarEvent, CalendarError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let event_id = format!("evt:v1:{ts:x}");
        let row = sqlx::query(
            "INSERT INTO calendar_events (event_id, source_event_id, account_id, source_id, title, description, location, start_at, end_at, timezone, all_day, recurrence_rule, status, visibility, event_type, conference_url, conference_provider, preparation_reminder_minutes, travel_buffer_minutes) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19) RETURNING event_id, source_event_id, account_id, source_id, title, description, location, start_at, end_at, timezone, all_day, recurrence_rule, status, visibility, event_type, importance_score, readiness_score, sync_status, conference_url, conference_provider, preparation_reminder_minutes, travel_buffer_minutes, created_at, updated_at"
        ).bind(&event_id).bind(req.source_event_id.as_deref()).bind(req.account_id.as_deref()).bind(req.source_id.as_deref()).bind(&req.title).bind(req.description.as_deref()).bind(req.location.as_deref()).bind(req.start_at).bind(req.end_at).bind(req.timezone.as_deref()).bind(req.all_day.unwrap_or(false)).bind(req.recurrence_rule.as_deref()).bind(req.status.as_deref().unwrap_or("scheduled")).bind(req.visibility.as_deref().unwrap_or("private")).bind(req.event_type.as_deref()).bind(req.conference_url.as_deref()).bind(req.conference_provider.as_deref()).bind(req.preparation_reminder_minutes).bind(req.travel_buffer_minutes).fetch_one(&mut **transaction).await?;
        row_to_event(row).map_err(CalendarError::from)
    }

    pub async fn get(&self, event_id: &str) -> Result<Option<CalendarEvent>, CalendarError> {
        let row = sqlx::query("SELECT event_id, source_event_id, account_id, source_id, title, description, location, start_at, end_at, timezone, all_day, recurrence_rule, status, visibility, event_type, importance_score, readiness_score, sync_status, conference_url, conference_provider, preparation_reminder_minutes, travel_buffer_minutes, created_at, updated_at FROM calendar_events WHERE event_id=$1")
            .bind(event_id).fetch_optional(&self.pool).await?;
        row.map(row_to_event)
            .transpose()
            .map_err(CalendarError::from)
    }

    pub async fn list(
        &self,
        query: &CalendarEventListQuery,
    ) -> Result<Vec<CalendarEvent>, CalendarError> {
        let limit = query.limit.unwrap_or(100).clamp(1, 500);
        let rows = sqlx::query(
            "SELECT event_id, source_event_id, account_id, source_id, title, description, location, start_at, end_at, timezone, all_day, recurrence_rule, status, visibility, event_type, importance_score, readiness_score, sync_status, conference_url, conference_provider, preparation_reminder_minutes, travel_buffer_minutes, created_at, updated_at FROM calendar_events WHERE ($1::text IS NULL OR account_id=$1) AND ($2::text IS NULL OR source_id=$2) AND ($3::timestamptz IS NULL OR end_at>=$3) AND ($4::timestamptz IS NULL OR start_at<=$4) AND ($5::text IS NULL OR status=$5) AND ($6::text IS NULL OR event_type=$6) ORDER BY start_at ASC LIMIT $7"
        ).bind(query.account_id.as_deref()).bind(query.source_id.as_deref()).bind(query.from).bind(query.to).bind(query.status.as_deref()).bind(query.event_type.as_deref()).bind(limit).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(row_to_event)
            .collect::<Result<Vec<_>, _>>()
            .map_err(CalendarError::from)
    }

    pub async fn update(
        &self,
        event_id: &str,
        update: &CalendarEventUpdate,
    ) -> Result<CalendarEvent, CalendarError> {
        let row = sqlx::query(
            "UPDATE calendar_events SET title=COALESCE($2,title), description=COALESCE($3,description), location=COALESCE($4,location), start_at=COALESCE($5,start_at), end_at=COALESCE($6,end_at), timezone=COALESCE($7,timezone), all_day=COALESCE($8,all_day), recurrence_rule=COALESCE($9,recurrence_rule), status=COALESCE($10,status), visibility=COALESCE($11,visibility), event_type=COALESCE($12,event_type), importance_score=COALESCE($13,importance_score), readiness_score=COALESCE($14,readiness_score), conference_url=COALESCE($15,conference_url), conference_provider=COALESCE($16,conference_provider), preparation_reminder_minutes=COALESCE($17,preparation_reminder_minutes), travel_buffer_minutes=COALESCE($18,travel_buffer_minutes), updated_at=now() WHERE event_id=$1 RETURNING event_id, source_event_id, account_id, source_id, title, description, location, start_at, end_at, timezone, all_day, recurrence_rule, status, visibility, event_type, importance_score, readiness_score, sync_status, conference_url, conference_provider, preparation_reminder_minutes, travel_buffer_minutes, created_at, updated_at"
        ).bind(event_id).bind(update.title.as_deref()).bind(update.description.as_deref()).bind(update.location.as_deref()).bind(update.start_at).bind(update.end_at).bind(update.timezone.as_deref()).bind(update.all_day).bind(update.recurrence_rule.as_deref()).bind(update.status.as_deref()).bind(update.visibility.as_deref()).bind(update.event_type.as_deref()).bind(update.importance_score).bind(update.readiness_score).bind(update.conference_url.as_deref()).bind(update.conference_provider.as_deref()).bind(update.preparation_reminder_minutes).bind(update.travel_buffer_minutes).fetch_one(&self.pool).await?;
        row_to_event(row).map_err(CalendarError::from)
    }

    pub async fn delete(&self, event_id: &str) -> Result<(), CalendarError> {
        sqlx::query("DELETE FROM calendar_events WHERE event_id=$1")
            .bind(event_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn set_status(&self, event_id: &str, status: &str) -> Result<(), CalendarError> {
        sqlx::query(
            "UPDATE calendar_events SET status = $2, updated_at = now() WHERE event_id = $1",
        )
        .bind(event_id)
        .bind(status)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn reschedule(
        &self,
        event_id: &str,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
    ) -> Result<CalendarEvent, CalendarError> {
        let row = sqlx::query("UPDATE calendar_events SET start_at=$2, end_at=$3, status='rescheduled', updated_at=now() WHERE event_id=$1 RETURNING event_id, source_event_id, account_id, source_id, title, description, location, start_at, end_at, timezone, all_day, recurrence_rule, status, visibility, event_type, importance_score, readiness_score, sync_status, conference_url, conference_provider, preparation_reminder_minutes, travel_buffer_minutes, created_at, updated_at")
            .bind(event_id).bind(start_at).bind(end_at).fetch_one(&self.pool).await?;
        row_to_event(row).map_err(CalendarError::from)
    }
}
