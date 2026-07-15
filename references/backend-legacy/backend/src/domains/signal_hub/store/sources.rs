use super::validation::validate_non_empty;
use super::{SignalHubError, SignalHubStore, SignalSource};
use sqlx::Row;
use sqlx::postgres::PgRow;

impl SignalHubStore {
    pub async fn list_sources(&self) -> Result<Vec<SignalSource>, SignalHubError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, code, display_name, category, source_kind, default_enabled,
                supports_connections, supports_runtime, supports_replay, supports_pause,
                supports_mute, capability_schema_version, created_at, updated_at
            FROM signal_sources
            ORDER BY code ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(row_to_source).collect()
    }

    pub async fn get_source(&self, source_code: &str) -> Result<SignalSource, SignalHubError> {
        let source_code = validate_non_empty("source_code", source_code)?;
        self.source_by_code(&source_code)
            .await?
            .ok_or(SignalHubError::SourceNotFound(source_code))
    }

    pub(crate) async fn source_by_code(
        &self,
        code: &str,
    ) -> Result<Option<SignalSource>, SignalHubError> {
        let row = sqlx::query(
            r#"
            SELECT
                id, code, display_name, category, source_kind, default_enabled,
                supports_connections, supports_runtime, supports_replay, supports_pause,
                supports_mute, capability_schema_version, created_at, updated_at
            FROM signal_sources
            WHERE code = $1
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;
        row.map(row_to_source).transpose()
    }
}

fn row_to_source(row: PgRow) -> Result<SignalSource, SignalHubError> {
    Ok(SignalSource {
        id: row.try_get("id")?,
        code: row.try_get("code")?,
        display_name: row.try_get("display_name")?,
        category: row.try_get("category")?,
        source_kind: row.try_get("source_kind")?,
        default_enabled: row.try_get("default_enabled")?,
        supports_connections: row.try_get("supports_connections")?,
        supports_runtime: row.try_get("supports_runtime")?,
        supports_replay: row.try_get("supports_replay")?,
        supports_pause: row.try_get("supports_pause")?,
        supports_mute: row.try_get("supports_mute")?,
        capability_schema_version: row.try_get("capability_schema_version")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
