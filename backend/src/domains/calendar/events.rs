use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{Postgres, Row, Transaction};
use thiserror::Error;

// ── CalendarAccount ────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalendarAccount {
    pub account_id: String,
    pub provider: String,
    pub account_name: String,
    pub email: Option<String>,
    pub credentials_reference: Option<String>,
    pub sync_status: String,
    pub capabilities: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct CalendarAccountStore {
    pool: PgPool,
}

impl CalendarAccountStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        provider: &str,
        account_name: &str,
        email: Option<&str>,
    ) -> Result<CalendarAccount, CalendarError> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let account_id = format!("cal:v1:{:x}", ts);
        let row = sqlx::query(
            "INSERT INTO calendar_accounts (account_id, provider, account_name, email) VALUES ($1,$2,$3,$4) RETURNING account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at"
        ).bind(&account_id).bind(provider).bind(account_name).bind(email).fetch_one(&self.pool).await?;
        Ok(CalendarAccount {
            account_id: row.try_get("account_id")?,
            provider: row.try_get("provider")?,
            account_name: row.try_get("account_name")?,
            email: row.try_get("email")?,
            credentials_reference: row.try_get("credentials_reference")?,
            sync_status: row.try_get("sync_status")?,
            capabilities: row.try_get("capabilities")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    pub async fn get(&self, account_id: &str) -> Result<Option<CalendarAccount>, CalendarError> {
        let row = sqlx::query("SELECT account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at FROM calendar_accounts WHERE account_id=$1")
            .bind(account_id).fetch_optional(&self.pool).await?;
        row.map(|r| {
            Ok(CalendarAccount {
                account_id: r.try_get("account_id")?,
                provider: r.try_get("provider")?,
                account_name: r.try_get("account_name")?,
                email: r.try_get("email")?,
                credentials_reference: r.try_get("credentials_reference")?,
                sync_status: r.try_get("sync_status")?,
                capabilities: r.try_get("capabilities")?,
                created_at: r.try_get("created_at")?,
                updated_at: r.try_get("updated_at")?,
            })
        })
        .transpose()
    }

    pub async fn list(
        &self,
        provider: Option<&str>,
    ) -> Result<Vec<CalendarAccount>, CalendarError> {
        let rows = if let Some(p) = provider {
            sqlx::query("SELECT account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at FROM calendar_accounts WHERE provider=$1 ORDER BY account_name")
                .bind(p).fetch_all(&self.pool).await?
        } else {
            sqlx::query("SELECT account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at FROM calendar_accounts ORDER BY account_name")
                .fetch_all(&self.pool).await?
        };
        rows.into_iter()
            .map(|r| {
                Ok(CalendarAccount {
                    account_id: r.try_get("account_id")?,
                    provider: r.try_get("provider")?,
                    account_name: r.try_get("account_name")?,
                    email: r.try_get("email")?,
                    credentials_reference: r.try_get("credentials_reference")?,
                    sync_status: r.try_get("sync_status")?,
                    capabilities: r.try_get("capabilities")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn update(
        &self,
        account_id: &str,
        update: &CalendarAccountUpdate,
    ) -> Result<CalendarAccount, CalendarError> {
        let row = sqlx::query(
            "UPDATE calendar_accounts SET account_name=COALESCE($2,account_name), email=COALESCE($3,email), sync_status=COALESCE($4,sync_status), updated_at=now() WHERE account_id=$1 RETURNING account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at"
        ).bind(account_id).bind(update.account_name.as_deref()).bind(update.email.as_deref()).bind(update.sync_status.as_deref()).fetch_one(&self.pool).await?;
        Ok(CalendarAccount {
            account_id: row.try_get("account_id")?,
            provider: row.try_get("provider")?,
            account_name: row.try_get("account_name")?,
            email: row.try_get("email")?,
            credentials_reference: row.try_get("credentials_reference")?,
            sync_status: row.try_get("sync_status")?,
            capabilities: row.try_get("capabilities")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    pub async fn upsert_google_workspace_account(
        &self,
        mail_account_id: &str,
        account_name: &str,
        email: Option<&str>,
        credentials_reference: &str,
    ) -> Result<CalendarAccount, CalendarError> {
        self.upsert_linked_provider_account(
            &format!("google-calendar:{mail_account_id}"),
            "google",
            mail_account_id,
            account_name,
            email,
            credentials_reference,
            "gmail",
            "google_calendar_api",
        )
        .await
    }

    pub async fn upsert_apple_icloud_account(
        &self,
        mail_account_id: &str,
        account_name: &str,
        email: Option<&str>,
        credentials_reference: &str,
    ) -> Result<CalendarAccount, CalendarError> {
        self.upsert_linked_provider_account(
            &format!("icloud-calendar:{mail_account_id}"),
            "apple",
            mail_account_id,
            account_name,
            email,
            credentials_reference,
            "icloud",
            "apple_caldav",
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn upsert_linked_provider_account(
        &self,
        account_id: &str,
        provider: &str,
        mail_account_id: &str,
        account_name: &str,
        email: Option<&str>,
        credentials_reference: &str,
        source_provider: &str,
        sync_mode: &str,
    ) -> Result<CalendarAccount, CalendarError> {
        let capabilities = json!({
            "mail_account_id": mail_account_id,
            "source_provider": source_provider,
            "connected_services": ["calendar"],
            "sync_mode": sync_mode
        });
        let row = sqlx::query(
            r#"
            INSERT INTO calendar_accounts (
                account_id,
                provider,
                account_name,
                email,
                credentials_reference,
                capabilities,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, now())
            ON CONFLICT (account_id)
            DO UPDATE SET
                provider = EXCLUDED.provider,
                account_name = EXCLUDED.account_name,
                email = EXCLUDED.email,
                credentials_reference = EXCLUDED.credentials_reference,
                capabilities = EXCLUDED.capabilities,
                updated_at = now()
            RETURNING account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at
            "#,
        )
        .bind(account_id)
        .bind(provider)
        .bind(account_name)
        .bind(email)
        .bind(credentials_reference)
        .bind(&capabilities)
        .fetch_one(&self.pool)
        .await?;
        Ok(CalendarAccount {
            account_id: row.try_get("account_id")?,
            provider: row.try_get("provider")?,
            account_name: row.try_get("account_name")?,
            email: row.try_get("email")?,
            credentials_reference: row.try_get("credentials_reference")?,
            sync_status: row.try_get("sync_status")?,
            capabilities: row.try_get("capabilities")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    pub async fn delete(&self, account_id: &str) -> Result<(), CalendarError> {
        sqlx::query("DELETE FROM calendar_accounts WHERE account_id=$1")
            .bind(account_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct CalendarAccountUpdate {
    pub account_name: Option<String>,
    pub email: Option<String>,
    pub sync_status: Option<String>,
}

// ── CalendarSource ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalendarSource {
    pub source_id: String,
    pub account_id: String,
    pub provider_calendar_id: Option<String>,
    pub name: String,
    pub color: Option<String>,
    pub timezone: Option<String>,
    pub visibility: String,
    pub read_only: bool,
    pub sync_enabled: bool,
    pub capabilities: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

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
        let source_id = format!("src:v1:{:x}", ts);
        let row = sqlx::query(
            "INSERT INTO calendar_sources (source_id, account_id, provider_calendar_id, name, color, timezone) VALUES ($1,$2,$3,$4,$5,$6) RETURNING source_id, account_id, provider_calendar_id, name, color, timezone, visibility, read_only, sync_enabled, capabilities, created_at, updated_at"
        ).bind(&source_id).bind(account_id).bind(provider_calendar_id).bind(name).bind(color).bind(timezone).fetch_one(&self.pool).await?;
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

    pub async fn list_by_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<CalendarSource>, CalendarError> {
        let rows = sqlx::query("SELECT source_id, account_id, provider_calendar_id, name, color, timezone, visibility, read_only, sync_enabled, capabilities, created_at, updated_at FROM calendar_sources WHERE account_id=$1 ORDER BY name")
            .bind(account_id).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                Ok(CalendarSource {
                    source_id: r.try_get("source_id")?,
                    account_id: r.try_get("account_id")?,
                    provider_calendar_id: r.try_get("provider_calendar_id")?,
                    name: r.try_get("name")?,
                    color: r.try_get("color")?,
                    timezone: r.try_get("timezone")?,
                    visibility: r.try_get("visibility")?,
                    read_only: r.try_get("read_only")?,
                    sync_enabled: r.try_get("sync_enabled")?,
                    capabilities: r.try_get("capabilities")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect()
    }

    pub async fn get(&self, source_id: &str) -> Result<Option<CalendarSource>, CalendarError> {
        let row = sqlx::query("SELECT source_id, account_id, provider_calendar_id, name, color, timezone, visibility, read_only, sync_enabled, capabilities, created_at, updated_at FROM calendar_sources WHERE source_id=$1")
            .bind(source_id).fetch_optional(&self.pool).await?;
        row.map(|r| {
            Ok(CalendarSource {
                source_id: r.try_get("source_id")?,
                account_id: r.try_get("account_id")?,
                provider_calendar_id: r.try_get("provider_calendar_id")?,
                name: r.try_get("name")?,
                color: r.try_get("color")?,
                timezone: r.try_get("timezone")?,
                visibility: r.try_get("visibility")?,
                read_only: r.try_get("read_only")?,
                sync_enabled: r.try_get("sync_enabled")?,
                capabilities: r.try_get("capabilities")?,
                created_at: r.try_get("created_at")?,
                updated_at: r.try_get("updated_at")?,
            })
        })
        .transpose()
    }
}

// ── CalendarEvent ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub event_id: String,
    pub source_event_id: Option<String>,
    pub account_id: Option<String>,
    pub source_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub timezone: Option<String>,
    pub all_day: bool,
    pub recurrence_rule: Option<String>,
    pub status: String,
    pub visibility: String,
    pub event_type: Option<String>,
    pub importance_score: Option<f64>,
    pub readiness_score: Option<f64>,
    pub sync_status: String,
    pub conference_url: Option<String>,
    pub conference_provider: Option<String>,
    pub preparation_reminder_minutes: Option<i32>,
    pub travel_buffer_minutes: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

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
        let event_id = format!("evt:v1:{:x}", ts);
        let row = sqlx::query(
            "INSERT INTO calendar_events (event_id, source_event_id, account_id, source_id, title, description, location, start_at, end_at, timezone, all_day, recurrence_rule, status, visibility, event_type, conference_url, conference_provider, preparation_reminder_minutes, travel_buffer_minutes) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19) RETURNING event_id, source_event_id, account_id, source_id, title, description, location, start_at, end_at, timezone, all_day, recurrence_rule, status, visibility, event_type, importance_score, readiness_score, sync_status, conference_url, conference_provider, preparation_reminder_minutes, travel_buffer_minutes, created_at, updated_at"
        ).bind(&event_id).bind(req.source_event_id.as_deref()).bind(req.account_id.as_deref()).bind(req.source_id.as_deref()).bind(&req.title).bind(req.description.as_deref()).bind(req.location.as_deref()).bind(req.start_at).bind(req.end_at).bind(req.timezone.as_deref()).bind(req.all_day.unwrap_or(false)).bind(req.recurrence_rule.as_deref()).bind(req.status.as_deref().unwrap_or("scheduled")).bind(req.visibility.as_deref().unwrap_or("private")).bind(req.event_type.as_deref()).bind(req.conference_url.as_deref()).bind(req.conference_provider.as_deref()).bind(req.preparation_reminder_minutes).bind(req.travel_buffer_minutes).fetch_one(&mut **transaction).await?;
        Ok(row_to_event(row)?)
    }

    pub async fn get(&self, event_id: &str) -> Result<Option<CalendarEvent>, CalendarError> {
        let row = sqlx::query("SELECT event_id, source_event_id, account_id, source_id, title, description, location, start_at, end_at, timezone, all_day, recurrence_rule, status, visibility, event_type, importance_score, readiness_score, sync_status, conference_url, conference_provider, preparation_reminder_minutes, travel_buffer_minutes, created_at, updated_at FROM calendar_events WHERE event_id=$1")
            .bind(event_id).fetch_optional(&self.pool).await?;
        row.map(|r| row_to_event(r).map_err(CalendarError::from))
            .transpose()
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
        Ok(row_to_event(row)?)
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
        let row = sqlx::query("UPDATE calendar_events SET start_at=$2, end_at=$3, status='rescheduled', updated_at=now() WHERE event_id=$1 RETURNING event_id, source_event_id, account_id, source_id, title, description, location, start_at, end_at, timezone, all_day, recurrence_rule, status, visibility, event_type, importance_score, readiness_score, sync_status, created_at, updated_at")
            .bind(event_id).bind(start_at).bind(end_at).fetch_one(&self.pool).await?;
        Ok(row_to_event(row)?)
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct NewCalendarEvent {
    pub source_event_id: Option<String>,
    pub account_id: Option<String>,
    pub source_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub timezone: Option<String>,
    pub all_day: Option<bool>,
    pub recurrence_rule: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub event_type: Option<String>,
    pub conference_url: Option<String>,
    pub conference_provider: Option<String>,
    pub preparation_reminder_minutes: Option<i32>,
    pub travel_buffer_minutes: Option<i32>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct CalendarEventUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub timezone: Option<String>,
    pub all_day: Option<bool>,
    pub recurrence_rule: Option<String>,
    pub status: Option<String>,
    pub visibility: Option<String>,
    pub event_type: Option<String>,
    pub importance_score: Option<f64>,
    pub readiness_score: Option<f64>,
    pub conference_url: Option<String>,
    pub conference_provider: Option<String>,
    pub preparation_reminder_minutes: Option<i32>,
    pub travel_buffer_minutes: Option<i32>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct CalendarEventListQuery {
    pub account_id: Option<String>,
    pub source_id: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub event_type: Option<String>,
    pub limit: Option<i64>,
}

fn row_to_event(row: PgRow) -> Result<CalendarEvent, sqlx::Error> {
    Ok(CalendarEvent {
        event_id: row.try_get("event_id")?,
        source_event_id: row.try_get("source_event_id")?,
        account_id: row.try_get("account_id")?,
        source_id: row.try_get("source_id")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        location: row.try_get("location")?,
        start_at: row.try_get("start_at")?,
        end_at: row.try_get("end_at")?,
        timezone: row.try_get("timezone")?,
        all_day: row.try_get("all_day")?,
        recurrence_rule: row.try_get("recurrence_rule")?,
        status: row.try_get("status")?,
        visibility: row.try_get("visibility")?,
        event_type: row.try_get("event_type")?,
        importance_score: row.try_get("importance_score")?,
        readiness_score: row.try_get("readiness_score")?,
        sync_status: row.try_get("sync_status")?,
        conference_url: row.try_get("conference_url")?,
        conference_provider: row.try_get("conference_provider")?,
        preparation_reminder_minutes: row.try_get("preparation_reminder_minutes")?,
        travel_buffer_minutes: row.try_get("travel_buffer_minutes")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Debug, Error)]
pub enum CalendarError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
}
