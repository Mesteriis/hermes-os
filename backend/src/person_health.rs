use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

#[derive(Clone, Debug, Serialize)]
pub struct PersonHealth {
    pub person_id: String,
    pub health_status: String,
    pub last_health_check: Option<DateTime<Utc>>,
    pub communication_gap_days: i32,
    pub watchlist: bool,
    pub interaction_count: i32,
    pub last_interaction_at: Option<DateTime<Utc>>,
    pub trust_score: Option<i16>,
    pub open_promises: i64,
    pub open_risks: i64,
}

#[derive(Clone)]
pub struct PersonHealthStore {
    pool: PgPool,
}

impl PersonHealthStore {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    pub async fn get(&self, person_id: &str) -> Result<Option<PersonHealth>, PersonHealthError> {
        let row = sqlx::query(
            r#"SELECT p.person_id, p.health_status, p.last_health_check, p.communication_gap_days,
               p.watchlist, p.interaction_count, p.last_interaction_at, p.trust_score,
               (SELECT count(*) FROM person_promises pp WHERE pp.person_id = p.person_id AND pp.status = 'pending') as open_promises,
               (SELECT count(*) FROM person_risks pr WHERE pr.person_id = p.person_id AND pr.resolved_at IS NULL) as open_risks
               FROM persons p WHERE p.person_id = $1"#
        ).bind(person_id).fetch_optional(&self.pool).await?;
        row.map(|r| PersonHealth {
            person_id: r.try_get("person_id").unwrap_or_default(),
            health_status: r.try_get("health_status").unwrap_or_else(|_| "healthy".into()),
            last_health_check: r.try_get("last_health_check").ok(),
            communication_gap_days: r.try_get("communication_gap_days").unwrap_or(0),
            watchlist: r.try_get("watchlist").unwrap_or(false),
            interaction_count: r.try_get("interaction_count").unwrap_or(0),
            last_interaction_at: r.try_get("last_interaction_at").ok(),
            trust_score: r.try_get("trust_score").ok(),
            open_promises: r.try_get("open_promises").unwrap_or(0),
            open_risks: r.try_get("open_risks").unwrap_or(0),
        }).map_or(Ok(None), |h| Ok(Some(h)))
    }

    pub async fn list_health(&self) -> Result<Vec<PersonHealth>, PersonHealthError> {
        let rows = sqlx::query(
            r#"SELECT p.person_id, p.health_status, p.last_health_check, p.communication_gap_days,
               p.watchlist, p.interaction_count, p.last_interaction_at, p.trust_score,
               0::bigint as open_promises, 0::bigint as open_risks
               FROM persons p WHERE p.health_status != 'healthy' ORDER BY p.last_interaction_at DESC NULLS LAST LIMIT 50"#
        ).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| PersonHealth {
            person_id: r.try_get("person_id").unwrap_or_default(),
            health_status: r.try_get("health_status").unwrap_or_else(|_| "healthy".into()),
            last_health_check: r.try_get("last_health_check").ok(),
            communication_gap_days: r.try_get("communication_gap_days").unwrap_or(0),
            watchlist: r.try_get("watchlist").unwrap_or(false),
            interaction_count: r.try_get("interaction_count").unwrap_or(0),
            last_interaction_at: r.try_get("last_interaction_at").ok(),
            trust_score: r.try_get("trust_score").ok(),
            open_promises: r.try_get("open_promises").unwrap_or(0),
            open_risks: r.try_get("open_risks").unwrap_or(0),
        }).collect())
    }

    pub async fn list_watchlist(&self) -> Result<Vec<PersonHealth>, PersonHealthError> {
        let rows = sqlx::query(
            r#"SELECT p.person_id, p.health_status, p.last_health_check, p.communication_gap_days,
               p.watchlist, p.interaction_count, p.last_interaction_at, p.trust_score,
               0::bigint as open_promises, 0::bigint as open_risks
               FROM persons p WHERE p.watchlist = true ORDER BY p.trust_score DESC NULLS LAST"#
        ).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| PersonHealth {
            person_id: r.try_get("person_id").unwrap_or_default(),
            health_status: r.try_get("health_status").unwrap_or_else(|_| "healthy".into()),
            last_health_check: r.try_get("last_health_check").ok(),
            communication_gap_days: r.try_get("communication_gap_days").unwrap_or(0),
            watchlist: r.try_get("watchlist").unwrap_or(false),
            interaction_count: r.try_get("interaction_count").unwrap_or(0),
            last_interaction_at: r.try_get("last_interaction_at").ok(),
            trust_score: r.try_get("trust_score").ok(),
            open_promises: r.try_get("open_promises").unwrap_or(0),
            open_risks: r.try_get("open_risks").unwrap_or(0),
        }).collect())
    }

    pub async fn toggle_watchlist(&self, person_id: &str) -> Result<bool, PersonHealthError> {
        let row = sqlx::query("UPDATE persons SET watchlist = NOT watchlist WHERE person_id = $1 RETURNING watchlist")
            .bind(person_id).fetch_optional(&self.pool).await?;
        Ok(row.map(|r: sqlx::postgres::PgRow| r.try_get("watchlist").unwrap_or(false)).unwrap_or(false))
    }
}

#[derive(Debug, Error)]
pub enum PersonHealthError {
    #[error(transparent)] Sqlx(#[from] sqlx::Error),
}
