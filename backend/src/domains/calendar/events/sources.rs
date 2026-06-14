use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::CalendarError;
use super::models::CalendarSource;

#[derive(Clone)]
pub struct CalendarSourceStore {
    pool: PgPool,
}

impl CalendarSourceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        account_id: &str,
        name: &str,
        provider_calendar_id: Option<&str>,
        color: Option<&str>,
        timezone: Option<&str>,
    ) -> Result<CalendarSource, CalendarError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let source_id = format!("src:v1:{ts:x}");
        let row = sqlx::query(
            "INSERT INTO calendar_sources (source_id, account_id, provider_calendar_id, name, color, timezone) VALUES ($1,$2,$3,$4,$5,$6) RETURNING source_id, account_id, provider_calendar_id, name, color, timezone, visibility, read_only, sync_enabled, capabilities, created_at, updated_at"
        ).bind(&source_id).bind(account_id).bind(provider_calendar_id).bind(name).bind(color).bind(timezone).fetch_one(&self.pool).await?;
        row_to_source(row).map_err(CalendarError::from)
    }

    pub async fn list_by_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<CalendarSource>, CalendarError> {
        let rows = sqlx::query("SELECT source_id, account_id, provider_calendar_id, name, color, timezone, visibility, read_only, sync_enabled, capabilities, created_at, updated_at FROM calendar_sources WHERE account_id=$1 ORDER BY name")
            .bind(account_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(row_to_source)
            .collect::<Result<Vec<_>, _>>()
            .map_err(CalendarError::from)
    }

    pub async fn get(&self, source_id: &str) -> Result<Option<CalendarSource>, CalendarError> {
        let row = sqlx::query("SELECT source_id, account_id, provider_calendar_id, name, color, timezone, visibility, read_only, sync_enabled, capabilities, created_at, updated_at FROM calendar_sources WHERE source_id=$1")
            .bind(source_id).fetch_optional(&self.pool).await?;
        row.map(row_to_source)
            .transpose()
            .map_err(CalendarError::from)
    }
}

fn row_to_source(row: PgRow) -> Result<CalendarSource, sqlx::Error> {
    Ok(CalendarSource {
        source_id: row.try_get("source_id")?,
        account_id: row.try_get("account_id")?,
        provider_calendar_id: row.try_get("provider_calendar_id")?,
        name: row.try_get("name")?,
        color: row.try_get("color")?,
        timezone: row.try_get("timezone")?,
        visibility: row.try_get("visibility")?,
        read_only: row.try_get("read_only")?,
        sync_enabled: row.try_get("sync_enabled")?,
        capabilities: row.try_get("capabilities")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
