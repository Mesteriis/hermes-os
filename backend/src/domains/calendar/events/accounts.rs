use serde_json::json;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};

use super::errors::CalendarError;
use super::models::{CalendarAccount, CalendarAccountUpdate};

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
        let account_id = format!("cal:v1:{ts:x}");
        let row = sqlx::query(
            "INSERT INTO calendar_accounts (account_id, provider, account_name, email) VALUES ($1,$2,$3,$4) RETURNING account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at"
        ).bind(&account_id).bind(provider).bind(account_name).bind(email).fetch_one(&self.pool).await?;
        row_to_account(row).map_err(CalendarError::from)
    }

    pub async fn get(&self, account_id: &str) -> Result<Option<CalendarAccount>, CalendarError> {
        let row = sqlx::query("SELECT account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at FROM calendar_accounts WHERE account_id=$1")
            .bind(account_id).fetch_optional(&self.pool).await?;
        row.map(row_to_account)
            .transpose()
            .map_err(CalendarError::from)
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
            .map(row_to_account)
            .collect::<Result<Vec<_>, _>>()
            .map_err(CalendarError::from)
    }

    pub async fn update(
        &self,
        account_id: &str,
        update: &CalendarAccountUpdate,
    ) -> Result<CalendarAccount, CalendarError> {
        let row = sqlx::query(
            "UPDATE calendar_accounts SET account_name=COALESCE($2,account_name), email=COALESCE($3,email), sync_status=COALESCE($4,sync_status), updated_at=now() WHERE account_id=$1 RETURNING account_id, provider, account_name, email, credentials_reference, sync_status, capabilities, created_at, updated_at"
        ).bind(account_id).bind(update.account_name.as_deref()).bind(update.email.as_deref()).bind(update.sync_status.as_deref()).fetch_one(&self.pool).await?;
        row_to_account(row).map_err(CalendarError::from)
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
        row_to_account(row).map_err(CalendarError::from)
    }

    pub async fn delete(&self, account_id: &str) -> Result<(), CalendarError> {
        sqlx::query("DELETE FROM calendar_accounts WHERE account_id=$1")
            .bind(account_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

fn row_to_account(row: PgRow) -> Result<CalendarAccount, sqlx::Error> {
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
