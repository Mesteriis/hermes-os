use chrono::{DateTime, Duration, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

pub struct CalendarWatchtowerService;

impl CalendarWatchtowerService {
    pub async fn events_needing_preparation(pool: &PgPool) -> Result<Value, CalendarHealthError> {
        let soon = Utc::now() + Duration::hours(24);
        let now = Utc::now();
        let rows = sqlx::query(
            "SELECT ce.event_id, ce.title, ce.start_at, ce.status, ce.readiness_score FROM calendar_events ce WHERE ce.start_at BETWEEN $1 AND $2 AND ce.status = 'scheduled' AND (ce.readiness_score IS NULL OR ce.readiness_score < 0.5) ORDER BY ce.start_at ASC LIMIT 20"
        ).bind(now).bind(soon).fetch_all(pool).await?;
        let items: Vec<Value> = rows.iter().map(|r| json!({
            "event_id": r.try_get::<String, _>("event_id").unwrap_or_default(),
            "title": r.try_get::<String, _>("title").unwrap_or_default(),
            "start_at": r.try_get::<DateTime<Utc>, _>("start_at").ok(),
            "status": r.try_get::<String, _>("status").unwrap_or_default(),
            "readiness_score": r.try_get::<Option<f64>, _>("readiness_score").unwrap_or(None),
        })).collect();
        Ok(json!({"events_needing_preparation": items}))
    }

    pub async fn events_without_outcomes(pool: &PgPool) -> Result<Value, CalendarHealthError> {
        let now = Utc::now();
        let rows = sqlx::query(
            "SELECT ce.event_id, ce.title, ce.start_at FROM calendar_events ce LEFT JOIN meeting_notes mn ON ce.event_id = mn.event_id WHERE ce.end_at < $1 AND ce.status IN ('completed', 'in_progress') AND mn.id IS NULL ORDER BY ce.start_at DESC LIMIT 20"
        ).bind(now).fetch_all(pool).await?;
        let items: Vec<Value> = rows
            .iter()
            .map(|r| {
                json!({
                    "event_id": r.try_get::<String, _>("event_id").unwrap_or_default(),
                    "title": r.try_get::<String, _>("title").unwrap_or_default(),
                    "start_at": r.try_get::<DateTime<Utc>, _>("start_at").ok(),
                })
            })
            .collect();
        Ok(json!({"events_without_notes": items}))
    }

    pub async fn weekly_brief(pool: &PgPool) -> Result<Value, CalendarHealthError> {
        let now = Utc::now();
        let week_end = now + Duration::days(7);

        // Upcoming events this week
        let events = sqlx::query("SELECT COUNT(*) as cnt FROM calendar_events WHERE start_at BETWEEN $1 AND $2 AND status='scheduled'")
            .bind(now).bind(week_end).fetch_one(pool).await?;
        let event_count: i64 = events.try_get("cnt")?;

        // Overdue deadlines
        let deadlines = sqlx::query(
            "SELECT COUNT(*) as cnt FROM deadline_events WHERE due_at < $1 AND status='active'",
        )
        .bind(now)
        .fetch_one(pool)
        .await?;
        let overdue_count: i64 = deadlines.try_get("cnt")?;

        // Past events without outcomes
        let past_no_outcomes = sqlx::query(
            "SELECT COUNT(*) as cnt FROM calendar_events ce LEFT JOIN meeting_notes mn ON ce.event_id=mn.event_id WHERE ce.end_at < $1 AND ce.status IN ('completed','in_progress') AND mn.id IS NULL"
        ).bind(now).fetch_one(pool).await?;
        let no_notes_count: i64 = past_no_outcomes.try_get("cnt")?;

        Ok(json!({
            "upcoming_events_this_week": event_count,
            "overdue_deadlines": overdue_count,
            "past_events_without_notes": no_notes_count,
            "week_start": now,
            "week_end": week_end,
        }))
    }

    pub async fn meeting_load_analysis(pool: &PgPool) -> Result<Value, CalendarHealthError> {
        let now = Utc::now();
        let week_ago = now - Duration::days(7);
        let rows = sqlx::query(
            "SELECT date_trunc('day', start_at) as day, COUNT(*) as cnt, COALESCE(SUM(EXTRACT(EPOCH FROM (end_at - start_at))/3600),0) as hours FROM calendar_events WHERE start_at >= $1 AND start_at <= $2 GROUP BY day ORDER BY day"
        ).bind(week_ago).bind(now).fetch_all(pool).await?;
        let days: Vec<Value> = rows
            .iter()
            .map(|r| {
                json!({
                    "day": r.try_get::<Option<DateTime<Utc>>, _>("day").ok(),
                    "event_count": r.try_get::<Option<i64>, _>("cnt").unwrap_or(Some(0)),
                    "hours": r.try_get::<Option<f64>, _>("hours").unwrap_or(Some(0.0)),
                })
            })
            .collect();
        Ok(json!({"daily_load": days}))
    }
}

#[derive(Debug, Error)]
pub enum CalendarHealthError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}

// ── Expanded Analytics ────────────────────────────────────────────────────

impl CalendarWatchtowerService {
    /// Time distribution by category for a given week
    pub async fn time_distribution(
        pool: &PgPool,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Value, CalendarHealthError> {
        let rows = sqlx::query(
            "SELECT COALESCE(event_type, 'other') as cat, COUNT(*) as cnt, COALESCE(SUM(EXTRACT(EPOCH FROM (end_at - start_at))/3600), 0) as hours FROM calendar_events WHERE start_at >= $1 AND start_at <= $2 AND status NOT IN ('cancelled', 'no_show') GROUP BY cat ORDER BY hours DESC"
        ).bind(from).bind(to).fetch_all(pool).await?;
        let categories: Vec<Value> = rows.iter().map(|r| json!({
            "category": crate::domains::calendar::intelligence::CalendarIntelligenceService::categorize_time(
                r.try_get::<String, _>("cat").unwrap_or_default().as_str(),
                ""
            ),
            "event_count": r.try_get::<Option<i64>, _>("cnt").unwrap_or(Some(0)),
            "hours": r.try_get::<Option<f64>, _>("hours").unwrap_or(Some(0.0)),
        })).collect();
        Ok(json!({"time_distribution": categories, "from": from, "to": to}))
    }

    /// Focus vs meetings balance
    pub async fn focus_balance(
        pool: &PgPool,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Value, CalendarHealthError> {
        let rows = sqlx::query(
            "SELECT COALESCE(event_type, 'other') as cat, COALESCE(SUM(EXTRACT(EPOCH FROM (end_at - start_at))/3600), 0) as hours FROM calendar_events WHERE start_at >= $1 AND start_at <= $2 AND status NOT IN ('cancelled', 'no_show') GROUP BY cat"
        ).bind(from).bind(to).fetch_all(pool).await?;
        let mut meetings_h = 0f64;
        let mut focus_h = 0f64;
        let mut other_h = 0f64;
        for r in &rows {
            let cat: String = r.try_get("cat").unwrap_or_default();
            let h: f64 = r.try_get("hours").unwrap_or(0.0);
            match cat.as_str() {
                "meeting" => meetings_h += h,
                "focus" => focus_h += h,
                _ => other_h += h,
            }
        }
        let total = meetings_h + focus_h + other_h;
        let focus_pct = if total > 0.0 {
            (focus_h / total * 100.0).round()
        } else {
            0.0
        };
        let warning = if focus_h < meetings_h * 0.3 {
            Some("Low focus time relative to meetings")
        } else {
            None
        };
        Ok(json!({
            "meetings_hours": meetings_h, "focus_hours": focus_h, "other_hours": other_h,
            "total_hours": total, "focus_percentage": focus_pct, "warning": warning,
            "from": from, "to": to,
        }))
    }

    /// Back-to-back meeting detection
    pub async fn back_to_back_meetings(
        pool: &PgPool,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Value, CalendarHealthError> {
        let rows = sqlx::query(
            "SELECT title, start_at, end_at FROM calendar_events WHERE start_at >= $1 AND start_at <= $2 AND event_type = 'meeting' AND status NOT IN ('cancelled', 'no_show') ORDER BY start_at"
        ).bind(from).bind(to).fetch_all(pool).await?;
        let events: Vec<(DateTime<Utc>, DateTime<Utc>, String)> = rows
            .iter()
            .map(|r| {
                (
                    r.try_get("start_at").unwrap_or(Utc::now()),
                    r.try_get("end_at").unwrap_or(Utc::now()),
                    r.try_get("title").unwrap_or_default(),
                )
            })
            .collect();
        let groups =
            crate::domains::calendar::intelligence::CalendarIntelligenceService::detect_back_to_back(&events);
        Ok(json!({"back_to_back_groups": groups, "from": from, "to": to}))
    }
}
