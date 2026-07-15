use chrono::{DateTime, Duration, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::domains::calendar::core::context_packs::EventContextPackStore;
use crate::domains::calendar::core::errors::CalendarCoreError;

pub struct CalendarBrainService;

impl CalendarBrainService {
    /// Answer a natural-language question about calendar
    pub async fn answer(pool: &PgPool, question: &str) -> Result<Value, CalendarBrainError> {
        let q = question.to_lowercase();
        if q.contains("недел") || q.contains("week") || q.contains("brief") {
            return Self::weekly_overview(pool).await;
        }
        // Default: search events by keyword
        Self::search_events(pool, question).await
    }

    pub async fn weekly_overview(pool: &PgPool) -> Result<Value, CalendarBrainError> {
        let now = Utc::now();
        let week_end = now + Duration::days(7);

        let important = sqlx::query(
            "SELECT event_id, title, start_at, event_type, importance_score FROM calendar_events WHERE start_at BETWEEN $1 AND $2 AND (importance_score > 0.5 OR event_type IN ('meeting','deadline','tax','legal')) ORDER BY start_at ASC LIMIT 10"
        ).bind(now).bind(week_end).fetch_all(pool).await?;
        let items: Vec<Value> = important
            .iter()
            .map(|r| {
                json!({
                    "event_id": r.try_get::<String, _>("event_id").unwrap_or_default(),
                    "title": r.try_get::<String, _>("title").unwrap_or_default(),
                    "start_at": r.try_get::<DateTime<Utc>, _>("start_at").ok(),
                    "event_type": r.try_get::<Option<String>, _>("event_type").unwrap_or(None),
                    "importance": r.try_get::<Option<f64>, _>("importance_score").unwrap_or(None),
                })
            })
            .collect();

        Ok(json!({"period": "next_7_days", "important_events": items}))
    }

    pub async fn search_events(pool: &PgPool, query: &str) -> Result<Value, CalendarBrainError> {
        let pattern = format!("%{query}%");
        let rows = sqlx::query(
            "SELECT event_id, title, description, start_at, event_type FROM calendar_events WHERE title ILIKE $1 OR description ILIKE $1 ORDER BY start_at DESC LIMIT 20"
        ).bind(&pattern).fetch_all(pool).await?;
        let items: Vec<Value> = rows
            .iter()
            .map(|r| {
                json!({
                    "event_id": r.try_get::<String, _>("event_id").unwrap_or_default(),
                    "title": r.try_get::<String, _>("title").unwrap_or_default(),
                    "start_at": r.try_get::<DateTime<Utc>, _>("start_at").ok(),
                    "event_type": r.try_get::<Option<String>, _>("event_type").unwrap_or(None),
                })
            })
            .collect();
        Ok(json!({"query": query, "results": items}))
    }

    pub async fn meeting_brief(pool: &PgPool, event_id: &str) -> Result<Value, CalendarBrainError> {
        // Get event
        let event = sqlx::query("SELECT event_id, title, description, start_at, location FROM calendar_events WHERE event_id=$1")
            .bind(event_id).fetch_optional(pool).await?;
        if event.is_none() {
            return Err(CalendarBrainError::NotFound);
        }

        // Get participants
        let parts = sqlx::query(
            "SELECT display_name, email, role FROM event_participants WHERE event_id=$1",
        )
        .bind(event_id)
        .fetch_all(pool)
        .await?;

        // Get context pack
        let ctx = EventContextPackStore::new(pool.clone())
            .get(event_id)
            .await?;

        Ok(json!({
            "event": event.map(|r| json!({
                "title": r.try_get::<String, _>("title").unwrap_or_default(),
                "description": r.try_get::<Option<String>, _>("description").unwrap_or(None),
                "start_at": r.try_get::<DateTime<Utc>, _>("start_at").ok(),
                "location": r.try_get::<Option<String>, _>("location").unwrap_or(None),
            })),
            "participants": parts.iter().map(|r| json!({
                "name": r.try_get::<Option<String>, _>("display_name").unwrap_or(None),
                "email": r.try_get::<String, _>("email").unwrap_or_default(),
                "role": r.try_get::<String, _>("role").unwrap_or_default(),
            })).collect::<Vec<_>>(),
            "context": ctx.map(|r| json!({
                "summary": r.summary,
                "participants_summary": r.participants_summary,
                "open_questions": r.open_questions,
                "risks": r.risks,
            })),
        }))
    }

    pub async fn generate_agenda(
        pool: &PgPool,
        event_id: &str,
    ) -> Result<Value, CalendarBrainError> {
        let event = sqlx::query("SELECT title, event_type FROM calendar_events WHERE event_id=$1")
            .bind(event_id)
            .fetch_optional(pool)
            .await?;
        let event_type = event
            .as_ref()
            .and_then(|r| r.try_get::<Option<String>, _>("event_type").unwrap_or(None))
            .unwrap_or_default();
        let title = event
            .as_ref()
            .map(|r| r.try_get::<String, _>("title").unwrap_or_default())
            .unwrap_or_default();

        let items: Vec<String> =
            if event_type == "meeting" || title.to_lowercase().contains("meeting") {
                vec![
                    "Confirm current scope.".into(),
                    "Discuss open questions.".into(),
                    "Review deadlines.".into(),
                    "Agree on next steps.".into(),
                    "Document decisions and owners.".into(),
                ]
            } else if event_type == "review" {
                vec![
                    "Review progress.".into(),
                    "Identify blockers.".into(),
                    "Adjust timeline.".into(),
                    "Assign follow-ups.".into(),
                ]
            } else if event_type == "planning" {
                vec![
                    "Define objectives.".into(),
                    "Break down tasks.".into(),
                    "Estimate effort.".into(),
                    "Assign owners.".into(),
                    "Set milestones.".into(),
                ]
            } else {
                vec![
                    "Open discussion.".into(),
                    "Review action items.".into(),
                    "Next steps.".into(),
                ]
            };
        Ok(json!({"event_id": event_id, "suggested_agenda": items}))
    }
}

#[derive(Debug, Error)]
pub enum CalendarBrainError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    CalendarCore(#[from] CalendarCoreError),
    #[error("not found")]
    NotFound,
}
