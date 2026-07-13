use super::validation::{
    connection_status_value, parse_required_uuid, validate_non_empty, validate_object,
};
use super::{
    SignalConnection, SignalConnectionCreate, SignalConnectionUpdate, SignalHubError,
    SignalHubStore,
};
use sqlx::Row;
use sqlx::postgres::PgRow;
use uuid::Uuid;

impl SignalHubStore {
    pub async fn list_connections(&self) -> Result<Vec<SignalConnection>, SignalHubError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                source_code,
                display_name,
                status,
                profile,
                settings,
                secret_ref,
                connected_at,
                last_seen_at,
                last_signal_at,
                last_sync_at,
                created_at,
                updated_at
            FROM signal_connections
            ORDER BY source_code ASC, display_name ASC, created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_connection).collect()
    }

    pub async fn create_connection(
        &self,
        request: &SignalConnectionCreate,
    ) -> Result<SignalConnection, SignalHubError> {
        let source_code = validate_non_empty("source_code", &request.source_code)?;
        let display_name = validate_non_empty("display_name", &request.display_name)?;
        let status = connection_status_value(&request.status)?;
        validate_object("settings", &request.settings)?;
        let profile = request
            .profile
            .as_deref()
            .map(|value| validate_non_empty("profile", value))
            .transpose()?;
        let secret_ref = request
            .secret_ref
            .as_deref()
            .map(|value| validate_non_empty("secret_ref", value))
            .transpose()?;

        let source = self
            .source_by_code(&source_code)
            .await?
            .ok_or_else(|| SignalHubError::SourceNotFound(source_code.clone()))?;
        if !source.supports_connections {
            return Err(SignalHubError::SourceDoesNotSupportConnections(source_code));
        }

        let id = Uuid::now_v7();
        sqlx::query(
            r#"
            INSERT INTO signal_connections (
                id,
                source_code,
                display_name,
                status,
                profile,
                settings,
                secret_ref,
                connected_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7,
                CASE WHEN $4 = 'connected' THEN now() ELSE NULL END
            )
            "#,
        )
        .bind(id)
        .bind(&source.code)
        .bind(display_name)
        .bind(status)
        .bind(profile)
        .bind(&request.settings)
        .bind(secret_ref)
        .execute(&self.pool)
        .await?;

        self.connection_by_id(id)
            .await?
            .ok_or_else(|| SignalHubError::ConnectionNotFound(id.to_string()))
    }

    pub async fn update_connection(
        &self,
        request: &SignalConnectionUpdate,
    ) -> Result<SignalConnection, SignalHubError> {
        let id = Uuid::parse_str(request.id.trim())
            .map_err(|_| SignalHubError::InvalidConnectionId(request.id.clone()))?;
        let current = self
            .connection_by_id(id)
            .await?
            .ok_or_else(|| SignalHubError::ConnectionNotFound(request.id.clone()))?;

        let display_name = request
            .display_name
            .as_deref()
            .map(|value| validate_non_empty("display_name", value))
            .transpose()?
            .unwrap_or(current.display_name.clone());
        let status = request
            .status
            .as_deref()
            .map(connection_status_value)
            .transpose()?
            .unwrap_or(current.status.as_str());
        let profile = match request.profile.as_ref() {
            Some(value) => Some(validate_non_empty("profile", value)?),
            None => current.profile.clone(),
        };
        let settings = request.settings.clone().unwrap_or(current.settings.clone());
        validate_object("settings", &settings)?;
        let secret_ref = match request.secret_ref.as_ref() {
            Some(value) => Some(validate_non_empty("secret_ref", value)?),
            None => current.secret_ref.clone(),
        };

        sqlx::query(
            r#"
            UPDATE signal_connections
            SET
                display_name = $2,
                status = $3,
                profile = $4,
                settings = $5,
                secret_ref = $6,
                connected_at = CASE
                    WHEN $3 = 'connected' AND connected_at IS NULL THEN now()
                    WHEN $3 <> 'connected' THEN connected_at
                    ELSE connected_at
                END,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(display_name)
        .bind(status)
        .bind(profile)
        .bind(settings)
        .bind(secret_ref)
        .execute(&self.pool)
        .await?;

        self.connection_by_id(id)
            .await?
            .ok_or_else(|| SignalHubError::ConnectionNotFound(request.id.clone()))
    }

    pub async fn remove_connection(
        &self,
        connection_id: &str,
    ) -> Result<SignalConnection, SignalHubError> {
        let id = Uuid::parse_str(connection_id.trim())
            .map_err(|_| SignalHubError::InvalidConnectionId(connection_id.to_owned()))?;
        let current = self
            .connection_by_id(id)
            .await?
            .ok_or_else(|| SignalHubError::ConnectionNotFound(connection_id.to_owned()))?;

        sqlx::query(
            r#"
            UPDATE signal_connections
            SET
                status = 'removed',
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.connection_by_id(id)
            .await?
            .ok_or_else(|| SignalHubError::ConnectionNotFound(current.id))
    }

    pub async fn get_connection(
        &self,
        connection_id: &str,
    ) -> Result<SignalConnection, SignalHubError> {
        let id = parse_required_uuid(connection_id)?;
        self.connection_by_id(id)
            .await?
            .ok_or_else(|| SignalHubError::ConnectionNotFound(connection_id.to_owned()))
    }

    pub async fn find_connection_by_account(
        &self,
        source_code: &str,
        account_id: &str,
    ) -> Result<Option<SignalConnection>, SignalHubError> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                source_code,
                display_name,
                status,
                profile,
                settings,
                secret_ref,
                connected_at,
                last_seen_at,
                last_signal_at,
                last_sync_at,
                created_at,
                updated_at
            FROM signal_connections
            WHERE source_code = $1
              AND status <> 'removed'
              AND settings->>'account_id' = $2
            ORDER BY
              CASE status
                WHEN 'connected' THEN 0
                WHEN 'paused' THEN 1
                WHEN 'muted' THEN 2
                WHEN 'disabled' THEN 3
                ELSE 4
              END,
              updated_at DESC,
              id DESC
            LIMIT 1
            "#,
        )
        .bind(validate_non_empty("source_code", source_code)?)
        .bind(validate_non_empty("account_id", account_id)?)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_connection).transpose()
    }

    pub(crate) async fn connection_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<SignalConnection>, SignalHubError> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                source_code,
                display_name,
                status,
                profile,
                settings,
                secret_ref,
                connected_at,
                last_seen_at,
                last_signal_at,
                last_sync_at,
                created_at,
                updated_at
            FROM signal_connections
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_connection).transpose()
    }
}

fn row_to_connection(row: PgRow) -> Result<SignalConnection, SignalHubError> {
    Ok(SignalConnection {
        id: row.try_get::<Uuid, _>("id")?.to_string(),
        source_code: row.try_get("source_code")?,
        display_name: row.try_get("display_name")?,
        status: row.try_get("status")?,
        profile: row.try_get("profile")?,
        settings: row.try_get("settings")?,
        secret_ref: row.try_get("secret_ref")?,
        connected_at: row.try_get("connected_at")?,
        last_seen_at: row.try_get("last_seen_at")?,
        last_signal_at: row.try_get("last_signal_at")?,
        last_sync_at: row.try_get("last_sync_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
