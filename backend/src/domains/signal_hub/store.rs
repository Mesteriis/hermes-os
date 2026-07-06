use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;
use uuid::Uuid;

use super::fixtures::{
    SystemProfileFixture, SystemSourceFixture, system_profile_fixtures, system_source_fixtures,
};
use super::policies::{SignalPolicy, SignalPolicyMode, SignalPolicyScope};
use crate::platform::events::{EventEnvelope, EventEnvelopeError, EventStoreError};
use crate::platform::settings::SettingsError;

#[derive(Debug, Error)]
pub enum SignalHubError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    EventStore(#[from] EventStoreError),

    #[error(transparent)]
    Envelope(#[from] EventEnvelopeError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[error(transparent)]
    Settings(#[from] SettingsError),

    #[error("invalid raw signal event type: {0}")]
    InvalidRawSignalEventType(String),

    #[error("signal source_code is missing")]
    MissingSourceCode,

    #[error("invalid signal policy scope: {0}")]
    InvalidPolicyScope(String),

    #[error("invalid signal policy mode: {0}")]
    InvalidPolicyMode(String),

    #[error("invalid signal connection id: {0}")]
    InvalidConnectionId(String),

    #[error("invalid signal connection status: {0}")]
    InvalidConnectionStatus(String),

    #[error("signal source not found: {0}")]
    SourceNotFound(String),

    #[error("signal source does not support connections: {0}")]
    SourceDoesNotSupportConnections(String),

    #[error("signal connection not found: {0}")]
    ConnectionNotFound(String),

    #[error("invalid signal runtime state: {0}")]
    InvalidRuntimeState(String),

    #[error("invalid signal runtime id: {0}")]
    InvalidRuntimeId(String),

    #[error("invalid signal health id: {0}")]
    InvalidHealthId(String),

    #[error("invalid signal replay request: {0}")]
    InvalidReplayRequest(String),

    #[error("signal fixture not found: {0}")]
    FixtureNotFound(String),

    #[error("invalid signal fixture catalog: {0}")]
    InvalidFixtureCatalog(String),

    #[error("signal profile not found: {0}")]
    ProfileNotFound(String),

    #[error("invalid signal profile definition: {0}")]
    InvalidProfileDefinition(String),

    #[error("system signal profile is immutable: {0}")]
    SystemProfileImmutable(String),

    #[error("{0} must not be empty")]
    EmptyField(&'static str),
}

impl SignalHubError {
    pub fn is_invalid_request(&self) -> bool {
        matches!(
            self,
            Self::InvalidRawSignalEventType(_)
                | Self::MissingSourceCode
                | Self::InvalidPolicyScope(_)
                | Self::InvalidPolicyMode(_)
                | Self::InvalidConnectionId(_)
                | Self::InvalidConnectionStatus(_)
                | Self::InvalidRuntimeState(_)
                | Self::InvalidRuntimeId(_)
                | Self::InvalidHealthId(_)
                | Self::InvalidReplayRequest(_)
                | Self::InvalidFixtureCatalog(_)
                | Self::InvalidProfileDefinition(_)
                | Self::SystemProfileImmutable(_)
                | Self::EmptyField(_)
        )
    }

    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            Self::SourceNotFound(_)
                | Self::ConnectionNotFound(_)
                | Self::FixtureNotFound(_)
                | Self::ProfileNotFound(_)
        )
    }

    pub fn is_failed_precondition(&self) -> bool {
        matches!(self, Self::SourceDoesNotSupportConnections(_))
    }
}

#[derive(Clone)]
pub struct SignalHubStore {
    pool: PgPool,
}

impl SignalHubStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn restore_system_sources(&self) -> Result<FixtureRestoreReport, SignalHubError> {
        let mut report = FixtureRestoreReport::default();

        for fixture in system_source_fixtures() {
            let existing = self.source_by_code(fixture.code).await?;
            match existing {
                Some(source) if source_matches_fixture(&source, fixture) => {}
                Some(source) => {
                    sqlx::query(
                        r#"
                        UPDATE signal_sources
                        SET
                            display_name = $2,
                            category = $3,
                            source_kind = $4,
                            default_enabled = $5,
                            supports_connections = $6,
                            supports_runtime = $7,
                            supports_replay = $8,
                            supports_pause = $9,
                            supports_mute = $10,
                            capability_schema_version = 1,
                            updated_at = now()
                        WHERE id = $1
                        "#,
                    )
                    .bind(source.id)
                    .bind(fixture.display_name)
                    .bind(fixture.category)
                    .bind(fixture.source_kind)
                    .bind(fixture.default_enabled)
                    .bind(fixture.supports_connections)
                    .bind(fixture.supports_runtime)
                    .bind(fixture.supports_replay)
                    .bind(fixture.supports_pause)
                    .bind(fixture.supports_mute)
                    .execute(&self.pool)
                    .await?;
                    report.sources_repaired += 1;
                }
                None => {
                    match sqlx::query(
                        r#"
                        INSERT INTO signal_sources (
                            id,
                            code,
                            display_name,
                            category,
                            source_kind,
                            default_enabled,
                            supports_connections,
                            supports_runtime,
                            supports_replay,
                            supports_pause,
                            supports_mute,
                            capability_schema_version
                        )
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 1)
                        "#,
                    )
                    .bind(Uuid::now_v7())
                    .bind(fixture.code)
                    .bind(fixture.display_name)
                    .bind(fixture.category)
                    .bind(fixture.source_kind)
                    .bind(fixture.default_enabled)
                    .bind(fixture.supports_connections)
                    .bind(fixture.supports_runtime)
                    .bind(fixture.supports_replay)
                    .bind(fixture.supports_pause)
                    .bind(fixture.supports_mute)
                    .execute(&self.pool)
                    .await
                    {
                        Ok(_) => {
                            report.sources_created += 1;
                        }
                        Err(error) if is_unique_violation(&error) => {
                            if let Some(source) = self.source_by_code(fixture.code).await?
                                && !source_matches_fixture(&source, fixture)
                            {
                                sqlx::query(
                                    r#"
                                        UPDATE signal_sources
                                        SET
                                            display_name = $2,
                                            category = $3,
                                            source_kind = $4,
                                            default_enabled = $5,
                                            supports_connections = $6,
                                            supports_runtime = $7,
                                            supports_replay = $8,
                                            supports_pause = $9,
                                            supports_mute = $10,
                                            capability_schema_version = 1,
                                            updated_at = now()
                                        WHERE id = $1
                                        "#,
                                )
                                .bind(source.id)
                                .bind(fixture.display_name)
                                .bind(fixture.category)
                                .bind(fixture.source_kind)
                                .bind(fixture.default_enabled)
                                .bind(fixture.supports_connections)
                                .bind(fixture.supports_runtime)
                                .bind(fixture.supports_replay)
                                .bind(fixture.supports_pause)
                                .bind(fixture.supports_mute)
                                .execute(&self.pool)
                                .await?;
                                report.sources_repaired += 1;
                            }
                        }
                        Err(error) => return Err(error.into()),
                    }
                }
            }
        }

        for fixture in system_profile_fixtures() {
            let existing = self.profile_by_code(fixture.code).await?;
            match existing {
                Some(profile) if profile_matches_fixture(&profile, fixture) => {}
                Some(profile) => {
                    sqlx::query(
                        r#"
                        UPDATE signal_profiles
                        SET
                            display_name = $2,
                            description = $3,
                            source_policies = $4,
                            is_system = $5,
                            updated_at = now()
                        WHERE id = $1
                        "#,
                    )
                    .bind(parse_required_uuid(&profile.id)?)
                    .bind(fixture.display_name)
                    .bind(fixture.description)
                    .bind(profile_policies_json(fixture)?)
                    .bind(fixture.is_system)
                    .execute(&self.pool)
                    .await?;
                    report.profiles_repaired += 1;
                }
                None => {
                    match sqlx::query(
                        r#"
                        INSERT INTO signal_profiles (
                            id,
                            code,
                            display_name,
                            description,
                            source_policies,
                            is_system
                        )
                        VALUES ($1, $2, $3, $4, $5, $6)
                        "#,
                    )
                    .bind(Uuid::now_v7())
                    .bind(fixture.code)
                    .bind(fixture.display_name)
                    .bind(fixture.description)
                    .bind(profile_policies_json(fixture)?)
                    .bind(fixture.is_system)
                    .execute(&self.pool)
                    .await
                    {
                        Ok(_) => {
                            report.profiles_created += 1;
                        }
                        Err(error) if is_unique_violation(&error) => {
                            if let Some(profile) = self.profile_by_code(fixture.code).await?
                                && !profile_matches_fixture(&profile, fixture)
                            {
                                sqlx::query(
                                    r#"
                                    UPDATE signal_profiles
                                    SET
                                        display_name = $2,
                                        description = $3,
                                        source_policies = $4,
                                        is_system = $5,
                                        updated_at = now()
                                    WHERE id = $1
                                    "#,
                                )
                                .bind(parse_required_uuid(&profile.id)?)
                                .bind(fixture.display_name)
                                .bind(fixture.description)
                                .bind(profile_policies_json(fixture)?)
                                .bind(fixture.is_system)
                                .execute(&self.pool)
                                .await?;
                                report.profiles_repaired += 1;
                            }
                        }
                        Err(error) => return Err(error.into()),
                    }
                }
            }
        }

        Ok(report)
    }

    pub async fn list_sources(&self) -> Result<Vec<SignalSource>, SignalHubError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                code,
                display_name,
                category,
                source_kind,
                default_enabled,
                supports_connections,
                supports_runtime,
                supports_replay,
                supports_pause,
                supports_mute,
                capability_schema_version,
                created_at,
                updated_at
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

    pub async fn list_profiles(&self) -> Result<Vec<SignalProfile>, SignalHubError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                code,
                display_name,
                description,
                source_policies,
                is_system,
                created_at,
                updated_at
            FROM signal_profiles
            ORDER BY is_system DESC, code ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_profile).collect()
    }

    pub async fn profile_by_code(
        &self,
        profile_code: &str,
    ) -> Result<Option<SignalProfile>, SignalHubError> {
        let profile_code = validate_non_empty("profile_code", profile_code)?;
        let row = sqlx::query(
            r#"
            SELECT
                id,
                code,
                display_name,
                description,
                source_policies,
                is_system,
                created_at,
                updated_at
            FROM signal_profiles
            WHERE code = $1
            "#,
        )
        .bind(profile_code)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_profile).transpose()
    }

    pub async fn list_capabilities(
        &self,
        source_code: Option<&str>,
        connection_id: Option<&str>,
    ) -> Result<Vec<SignalCapability>, SignalHubError> {
        let source_code = source_code
            .map(|value| validate_non_empty("source_code", value))
            .transpose()?;
        let connection_id = connection_id.map(parse_required_uuid).transpose()?;

        let rows = sqlx::query(
            r#"
            SELECT
                id,
                source_code,
                connection_id,
                capability,
                state,
                reason,
                requires_confirmation,
                action_class,
                updated_at
            FROM signal_capabilities
            WHERE ($1::text IS NULL OR source_code = $1)
              AND (
                ($2::uuid IS NULL AND connection_id IS NULL)
                OR connection_id = $2
                OR $2::uuid IS NULL
              )
            ORDER BY source_code ASC, capability ASC
            "#,
        )
        .bind(source_code)
        .bind(connection_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_capability).collect()
    }

    pub async fn replace_source_capabilities(
        &self,
        source_code: &str,
        connection_id: Option<&str>,
        capabilities: &[SignalCapabilityUpsert],
    ) -> Result<(), SignalHubError> {
        let source_code = validate_non_empty("source_code", source_code)?;
        let connection_id = connection_id.map(parse_required_uuid).transpose()?;

        sqlx::query(
            r#"
            DELETE FROM signal_capabilities
            WHERE source_code = $1
              AND (
                ($2::uuid IS NULL AND connection_id IS NULL)
                OR connection_id = $2
              )
            "#,
        )
        .bind(&source_code)
        .bind(connection_id)
        .execute(&self.pool)
        .await?;

        for capability in capabilities {
            let capability_name = validate_non_empty("capability", &capability.capability)?;
            let state = validate_non_empty("state", &capability.state)?;
            let action_class = validate_non_empty("action_class", &capability.action_class)?;
            let reason = capability
                .reason
                .as_deref()
                .map(|value| validate_non_empty("reason", value))
                .transpose()?;

            sqlx::query(
                r#"
                INSERT INTO signal_capabilities (
                    id,
                    source_code,
                    connection_id,
                    capability,
                    state,
                    reason,
                    requires_confirmation,
                    action_class,
                    updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
                "#,
            )
            .bind(Uuid::now_v7())
            .bind(&source_code)
            .bind(connection_id)
            .bind(capability_name)
            .bind(state)
            .bind(reason)
            .bind(capability.requires_confirmation)
            .bind(action_class)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn create_profile(
        &self,
        request: &SignalProfileCreate,
    ) -> Result<SignalProfile, SignalHubError> {
        let code = validate_non_empty("code", &request.code)?;
        let display_name = validate_non_empty("display_name", &request.display_name)?;
        let description = validate_non_empty("description", &request.description)?;
        let source_policies = validate_profile_policies(&request.source_policies)?;

        if self.profile_by_code(&code).await?.is_some() {
            return Err(SignalHubError::InvalidProfileDefinition(format!(
                "profile code already exists: {code}"
            )));
        }

        let id = Uuid::now_v7();
        sqlx::query(
            r#"
            INSERT INTO signal_profiles (
                id,
                code,
                display_name,
                description,
                source_policies,
                is_system
            )
            VALUES ($1, $2, $3, $4, $5, FALSE)
            "#,
        )
        .bind(id)
        .bind(&code)
        .bind(display_name)
        .bind(description)
        .bind(serde_json::to_value(&source_policies)?)
        .execute(&self.pool)
        .await?;

        self.profile_by_code(&code)
            .await?
            .ok_or_else(|| SignalHubError::ProfileNotFound(code))
    }

    pub async fn update_profile(
        &self,
        request: &SignalProfileUpdate,
    ) -> Result<SignalProfile, SignalHubError> {
        let code = validate_non_empty("code", &request.code)?;
        let current = self
            .profile_by_code(&code)
            .await?
            .ok_or_else(|| SignalHubError::ProfileNotFound(code.clone()))?;

        if current.is_system {
            return Err(SignalHubError::SystemProfileImmutable(code));
        }

        let display_name = request
            .display_name
            .as_deref()
            .map(|value| validate_non_empty("display_name", value))
            .transpose()?
            .unwrap_or(current.display_name.clone());
        let description = request
            .description
            .as_deref()
            .map(|value| validate_non_empty("description", value))
            .transpose()?
            .unwrap_or(current.description.clone());
        let source_policies = match request.source_policies.as_ref() {
            Some(policies) => validate_profile_policies(policies)?,
            None => current.source_policies.clone(),
        };

        sqlx::query(
            r#"
            UPDATE signal_profiles
            SET
                display_name = $2,
                description = $3,
                source_policies = $4,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(parse_required_uuid(&current.id)?)
        .bind(display_name)
        .bind(description)
        .bind(serde_json::to_value(&source_policies)?)
        .execute(&self.pool)
        .await?;

        self.profile_by_code(&current.code)
            .await?
            .ok_or_else(|| SignalHubError::ProfileNotFound(current.code))
    }

    pub async fn delete_profile(
        &self,
        profile_code: &str,
    ) -> Result<SignalProfile, SignalHubError> {
        let code = validate_non_empty("profile_code", profile_code)?;
        let current = self
            .profile_by_code(&code)
            .await?
            .ok_or_else(|| SignalHubError::ProfileNotFound(code.clone()))?;

        if current.is_system {
            return Err(SignalHubError::SystemProfileImmutable(code));
        }

        sqlx::query(
            r#"
            DELETE FROM signal_profiles
            WHERE id = $1
            "#,
        )
        .bind(parse_required_uuid(&current.id)?)
        .execute(&self.pool)
        .await?;

        Ok(current)
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

    pub async fn list_health(&self) -> Result<Vec<SignalHealth>, SignalHubError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                source_code,
                connection_id,
                level,
                summary,
                last_ok_at,
                last_failure_at,
                failure_count,
                consecutive_failure_count,
                next_retry_at,
                evidence,
                updated_at
            FROM signal_health
            ORDER BY source_code ASC, updated_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_health).collect()
    }

    pub async fn run_health_check(
        &self,
        request: &SignalHealthCheckRequest,
    ) -> Result<SignalHealth, SignalHubError> {
        let source_code = validate_non_empty("source_code", &request.source_code)?;
        let source = self
            .source_by_code(&source_code)
            .await?
            .ok_or_else(|| SignalHubError::SourceNotFound(source_code.clone()))?;
        let connection = match request.connection_id.as_deref() {
            Some(connection_id) => {
                let id = Uuid::parse_str(connection_id.trim())
                    .map_err(|_| SignalHubError::InvalidConnectionId(connection_id.to_owned()))?;
                Some(
                    self.connection_by_id(id).await?.ok_or_else(|| {
                        SignalHubError::ConnectionNotFound(connection_id.to_owned())
                    })?,
                )
            }
            None => None,
        };
        if connection
            .as_ref()
            .is_some_and(|connection| connection.source_code != source_code)
        {
            return Err(SignalHubError::InvalidConnectionId(
                request.connection_id.clone().unwrap_or_default(),
            ));
        }

        let runtime = self
            .runtime_state(
                &source_code,
                request
                    .runtime_kind
                    .as_deref()
                    .unwrap_or("signal_health_check"),
            )
            .await?;
        let snapshot = SignalHealthSnapshotWrite::from(build_health_snapshot(
            &source,
            connection.as_ref(),
            runtime.as_ref(),
        ));
        self.upsert_health_snapshot(request, snapshot).await
    }

    pub async fn upsert_health_snapshot(
        &self,
        request: &SignalHealthCheckRequest,
        snapshot: SignalHealthSnapshotWrite,
    ) -> Result<SignalHealth, SignalHubError> {
        let source_code = validate_non_empty("source_code", &request.source_code)?;
        validate_object("evidence", &snapshot.evidence)?;
        let existing = self
            .health_by_scope(&source_code, request.connection_id.as_deref())
            .await?;
        let health_id = existing
            .as_ref()
            .map(|health| Uuid::parse_str(&health.id))
            .transpose()
            .map_err(|_| {
                SignalHubError::InvalidHealthId(existing.map(|item| item.id).unwrap_or_default())
            })?
            .unwrap_or_else(Uuid::now_v7);
        let connection_uuid = request
            .connection_id
            .as_deref()
            .map(parse_required_uuid)
            .transpose()?;

        sqlx::query(
            r#"
            INSERT INTO signal_health (
                id,
                source_code,
                connection_id,
                level,
                summary,
                last_ok_at,
                last_failure_at,
                failure_count,
                consecutive_failure_count,
                next_retry_at,
                evidence,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, now())
            ON CONFLICT (id) DO UPDATE
            SET
                level = EXCLUDED.level,
                summary = EXCLUDED.summary,
                last_ok_at = EXCLUDED.last_ok_at,
                last_failure_at = EXCLUDED.last_failure_at,
                failure_count = EXCLUDED.failure_count,
                consecutive_failure_count = EXCLUDED.consecutive_failure_count,
                next_retry_at = EXCLUDED.next_retry_at,
                evidence = EXCLUDED.evidence,
                updated_at = now()
            "#,
        )
        .bind(health_id)
        .bind(&source_code)
        .bind(connection_uuid)
        .bind(&snapshot.level)
        .bind(&snapshot.summary)
        .bind(snapshot.last_ok_at)
        .bind(snapshot.last_failure_at)
        .bind(snapshot.failure_count)
        .bind(snapshot.consecutive_failure_count)
        .bind(snapshot.next_retry_at)
        .bind(snapshot.evidence)
        .execute(&self.pool)
        .await?;

        self.health_by_id(health_id)
            .await?
            .ok_or_else(|| SignalHubError::InvalidHealthId(health_id.to_string()))
    }

    pub async fn list_runtime_states(&self) -> Result<Vec<SignalRuntimeState>, SignalHubError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                source_code,
                connection_id,
                runtime_kind,
                state,
                last_started_at,
                last_stopped_at,
                last_heartbeat_at,
                last_error_at,
                last_error_code,
                last_error_message_redacted,
                metadata,
                updated_at
            FROM signal_runtime_states
            ORDER BY source_code ASC, runtime_kind ASC, updated_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_runtime_state).collect()
    }

    pub async fn runtime_state(
        &self,
        source_code: &str,
        runtime_kind: &str,
    ) -> Result<Option<SignalRuntimeState>, SignalHubError> {
        let source_code = validate_non_empty("source_code", source_code)?;
        let runtime_kind = validate_non_empty("runtime_kind", runtime_kind)?;

        let row = sqlx::query(
            r#"
            SELECT
                id,
                source_code,
                connection_id,
                runtime_kind,
                state,
                last_started_at,
                last_stopped_at,
                last_heartbeat_at,
                last_error_at,
                last_error_code,
                last_error_message_redacted,
                metadata,
                updated_at
            FROM signal_runtime_states
            WHERE source_code = $1
              AND connection_id IS NULL
              AND runtime_kind = $2
            "#,
        )
        .bind(&source_code)
        .bind(&runtime_kind)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_runtime_state).transpose()
    }

    pub async fn ensure_runtime_state(
        &self,
        source_code: &str,
        runtime_kind: &str,
        default_state: &str,
        metadata: Value,
    ) -> Result<SignalRuntimeState, SignalHubError> {
        let source_code = validate_non_empty("source_code", source_code)?;
        let runtime_kind = validate_non_empty("runtime_kind", runtime_kind)?;
        let default_state = runtime_state_value(default_state)?;
        validate_object("metadata", &metadata)?;

        if let Some(runtime) = self.runtime_state(&source_code, &runtime_kind).await? {
            return Ok(runtime);
        }

        sqlx::query(
            r#"
            INSERT INTO signal_runtime_states (
                id,
                source_code,
                runtime_kind,
                state,
                last_started_at,
                metadata
            )
            VALUES ($1, $2, $3, $4, CASE WHEN $4 = 'running' THEN now() ELSE NULL END, $5)
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(&source_code)
        .bind(&runtime_kind)
        .bind(default_state)
        .bind(metadata)
        .execute(&self.pool)
        .await?;

        self.runtime_state(&source_code, &runtime_kind)
            .await?
            .ok_or(SignalHubError::InvalidRuntimeState(
                default_state.to_owned(),
            ))
    }

    pub async fn set_runtime_state(
        &self,
        request: &SignalRuntimeStateUpdate,
    ) -> Result<SignalRuntimeState, SignalHubError> {
        let source_code = validate_non_empty("source_code", &request.source_code)?;
        let runtime_kind = validate_non_empty("runtime_kind", &request.runtime_kind)?;
        let state = runtime_state_value(&request.state)?;
        validate_object("metadata", &request.metadata)?;

        let existing = self.runtime_state(&source_code, &runtime_kind).await?;
        if let Some(runtime) = existing {
            let runtime_id = Uuid::parse_str(&runtime.id)
                .map_err(|_| SignalHubError::InvalidRuntimeId(runtime.id.clone()))?;
            sqlx::query(
                r#"
                UPDATE signal_runtime_states
                SET
                    state = $2,
                    last_started_at = CASE
                        WHEN $2 = 'running' AND state <> 'running' THEN now()
                        ELSE last_started_at
                    END,
                    last_stopped_at = CASE
                        WHEN $2 IN ('stopped', 'paused', 'muted') AND state <> $2 THEN now()
                        ELSE last_stopped_at
                    END,
                    last_heartbeat_at = CASE
                        WHEN $2 = 'running' THEN now()
                        ELSE last_heartbeat_at
                    END,
                    metadata = $3,
                    updated_at = now()
                WHERE id = $1
                "#,
            )
            .bind(runtime_id)
            .bind(state)
            .bind(&request.metadata)
            .execute(&self.pool)
            .await?;
        } else {
            self.ensure_runtime_state(&source_code, &runtime_kind, state, request.metadata.clone())
                .await?;
            if state != "running" {
                sqlx::query(
                    r#"
                    UPDATE signal_runtime_states
                    SET
                        state = $2,
                        last_started_at = CASE WHEN $2 = 'running' THEN now() ELSE last_started_at END,
                        last_stopped_at = CASE
                            WHEN $2 IN ('stopped', 'paused', 'muted') THEN now()
                            ELSE last_stopped_at
                        END,
                        metadata = $3,
                        updated_at = now()
                    WHERE source_code = $1
                      AND connection_id IS NULL
                      AND runtime_kind = $4
                    "#,
                )
                .bind(&source_code)
                .bind(state)
                .bind(&request.metadata)
                .bind(&runtime_kind)
                .execute(&self.pool)
                .await?;
            }
        }

        let runtime = self
            .runtime_state(&source_code, &runtime_kind)
            .await?
            .ok_or(SignalHubError::InvalidRuntimeState(state.to_owned()))?;
        Ok(runtime)
    }

    pub async fn set_source_runtime_state(
        &self,
        source_code: &str,
        state: &str,
    ) -> Result<u64, SignalHubError> {
        let source_code = validate_non_empty("source_code", source_code)?;
        let state = runtime_state_value(state)?;

        let result = sqlx::query(
            r#"
            UPDATE signal_runtime_states
            SET
                state = $2,
                last_started_at = CASE
                    WHEN $2 = 'running' AND state <> 'running' THEN now()
                    ELSE last_started_at
                END,
                last_stopped_at = CASE
                    WHEN $2 IN ('stopped', 'paused', 'muted') AND state <> $2 THEN now()
                    ELSE last_stopped_at
                END,
                last_heartbeat_at = CASE
                    WHEN $2 = 'running' THEN now()
                    ELSE last_heartbeat_at
                END,
                updated_at = now()
            WHERE source_code = $1
              AND connection_id IS NULL
            "#,
        )
        .bind(source_code)
        .bind(state)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn list_replay_requests(&self) -> Result<Vec<SignalReplayRequest>, SignalHubError> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                source_code,
                connection_id,
                event_pattern,
                status,
                requested_by,
                requested_at,
                started_at,
                completed_at,
                last_error_redacted,
                replayed_count,
                metadata
            FROM signal_replay_requests
            ORDER BY requested_at DESC, id DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_replay_request).collect()
    }

    pub async fn create_replay_request(
        &self,
        request: &SignalReplayRequestCreate,
    ) -> Result<SignalReplayRequest, SignalHubError> {
        validate_object("metadata", &request.metadata)?;
        let requested_by = validate_non_empty("requested_by", &request.requested_by)?;
        let mut source_code = request
            .source_code
            .as_deref()
            .map(|value| validate_non_empty("source_code", value))
            .transpose()?;
        let target_consumer = request
            .target_consumer
            .as_deref()
            .map(|value| validate_non_empty("target_consumer", value))
            .transpose()?;
        let target_projection = request
            .target_projection
            .as_deref()
            .map(|value| validate_non_empty("target_projection", value))
            .transpose()?;
        let connection_id = match request.connection_id.as_deref() {
            Some(value) => Some(
                Uuid::parse_str(value.trim())
                    .map_err(|_| SignalHubError::InvalidConnectionId(value.to_owned()))?,
            ),
            None => None,
        };
        let event_pattern = request
            .event_pattern
            .as_deref()
            .map(|value| validate_non_empty("event_pattern", value))
            .transpose()?;
        let from_position = request.from_position;
        let to_position = request.to_position;
        let from_time = request.from_time;
        let to_time = request.to_time;

        let has_selector = source_code.is_some()
            || connection_id.is_some()
            || event_pattern.is_some()
            || from_position.is_some()
            || to_position.is_some()
            || from_time.is_some()
            || to_time.is_some();

        if target_consumer.is_some() && target_projection.is_some() {
            return Err(SignalHubError::InvalidReplayRequest(
                "target_consumer and target_projection are mutually exclusive".to_owned(),
            ));
        }

        if !has_selector && target_consumer.is_none() && target_projection.is_none() {
            return Err(SignalHubError::InvalidReplayRequest(
                "at least one replay selector is required".to_owned(),
            ));
        }

        if target_consumer.is_some() && !has_selector {
            return Err(SignalHubError::InvalidReplayRequest(
                "target_consumer replay requires at least one source, pattern, connection or range selector"
                    .to_owned(),
            ));
        }

        if let Some(target_projection) = target_projection.as_deref() {
            match target_projection {
                "communication_messages"
                | "timeline_event_log"
                | "person_derived_evidence"
                | "project_link_review_effects" => {}
                other => {
                    return Err(SignalHubError::InvalidReplayRequest(format!(
                        "unsupported target_projection: {other}"
                    )));
                }
            }
        }

        if let (Some(from_position), Some(to_position)) = (from_position, to_position) {
            if from_position < 0 || to_position < 0 || from_position > to_position {
                return Err(SignalHubError::InvalidReplayRequest(
                    "from_position and to_position must define a non-negative inclusive range"
                        .to_owned(),
                ));
            }
        } else if from_position.is_some_and(|value| value < 0)
            || to_position.is_some_and(|value| value < 0)
        {
            return Err(SignalHubError::InvalidReplayRequest(
                "replay positions must be non-negative".to_owned(),
            ));
        }

        if let (Some(from_time), Some(to_time)) = (from_time, to_time)
            && from_time > to_time
        {
            return Err(SignalHubError::InvalidReplayRequest(
                "from_time must be earlier than or equal to to_time".to_owned(),
            ));
        }

        if let Some(connection_id) = connection_id {
            let connection = self
                .connection_by_id(connection_id)
                .await?
                .ok_or_else(|| SignalHubError::ConnectionNotFound(connection_id.to_string()))?;
            if source_code
                .as_deref()
                .is_some_and(|value| value != connection.source_code)
            {
                return Err(SignalHubError::InvalidReplayRequest(
                    "connection_id source does not match source_code".to_owned(),
                ));
            }
            source_code = Some(connection.source_code);
        }

        if let Some(source_code_value) = source_code.as_deref() {
            let source = self
                .source_by_code(source_code_value)
                .await?
                .ok_or_else(|| SignalHubError::SourceNotFound(source_code_value.to_owned()))?;
            if !source.supports_replay {
                return Err(SignalHubError::InvalidReplayRequest(format!(
                    "signal source does not support replay: {}",
                    source_code_value
                )));
            }
        }

        let metadata = build_replay_metadata(
            request.metadata.clone(),
            from_position,
            to_position,
            from_time,
            to_time,
            target_consumer.as_deref(),
            target_projection.as_deref(),
        )?;

        let id = Uuid::now_v7();
        sqlx::query(
            r#"
            INSERT INTO signal_replay_requests (
                id,
                source_code,
                connection_id,
                event_pattern,
                status,
                requested_by,
                metadata
            )
            VALUES ($1, $2, $3, $4, 'queued', $5, $6)
            "#,
        )
        .bind(id)
        .bind(&source_code)
        .bind(connection_id)
        .bind(&event_pattern)
        .bind(requested_by)
        .bind(&metadata)
        .execute(&self.pool)
        .await?;

        self.replay_request_by_id(id)
            .await?
            .ok_or_else(|| SignalHubError::InvalidReplayRequest(id.to_string()))
    }

    pub async fn claim_next_replay_request(
        &self,
    ) -> Result<Option<SignalReplayRequest>, SignalHubError> {
        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            WITH candidate AS (
                SELECT id
                FROM signal_replay_requests
                WHERE status IN ('queued', 'requested')
                ORDER BY requested_at ASC, id ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            UPDATE signal_replay_requests AS replay
            SET
                status = 'running',
                started_at = now(),
                completed_at = NULL,
                last_error_redacted = NULL
            FROM candidate
            WHERE replay.id = candidate.id
            RETURNING
                replay.id,
                replay.source_code,
                replay.connection_id,
                replay.event_pattern,
                replay.status,
                replay.requested_by,
                replay.requested_at,
                replay.started_at,
                replay.completed_at,
                replay.last_error_redacted,
                replay.replayed_count,
                replay.metadata
            "#,
        )
        .fetch_optional(&mut *transaction)
        .await?;
        transaction.commit().await?;

        row.map(row_to_replay_request).transpose()
    }

    pub async fn mark_replay_request_completed(
        &self,
        replay_request_id: &str,
        replayed_count: i32,
    ) -> Result<(), SignalHubError> {
        let replay_request_id = Uuid::parse_str(replay_request_id.trim())
            .map_err(|_| SignalHubError::InvalidReplayRequest(replay_request_id.to_owned()))?;

        sqlx::query(
            r#"
            UPDATE signal_replay_requests
            SET
                status = 'completed',
                replayed_count = $2,
                completed_at = now(),
                last_error_redacted = NULL
            WHERE id = $1
            "#,
        )
        .bind(replay_request_id)
        .bind(replayed_count.max(0))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn mark_replay_request_failed(
        &self,
        replay_request_id: &str,
        error: &str,
    ) -> Result<(), SignalHubError> {
        let replay_request_id = Uuid::parse_str(replay_request_id.trim())
            .map_err(|_| SignalHubError::InvalidReplayRequest(replay_request_id.to_owned()))?;

        sqlx::query(
            r#"
            UPDATE signal_replay_requests
            SET
                status = 'failed',
                completed_at = now(),
                last_error_redacted = $2
            WHERE id = $1
            "#,
        )
        .bind(replay_request_id)
        .bind(truncate_redacted_error(error))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_paused_events_for_replay(
        &self,
        request: &SignalReplayRequest,
        limit: u32,
    ) -> Result<Vec<PausedSignalEvent>, SignalHubError> {
        let limit = i64::from(limit.clamp(1, 1000));
        let rows = sqlx::query(
            r#"
            SELECT id, event_id, source_code, raw_event_type, event_envelope, paused_at
            FROM signal_paused_events
            WHERE released_at IS NULL
              AND ($1::text IS NULL OR source_code = $1)
              AND ($2::uuid IS NULL OR connection_id = $2)
            ORDER BY paused_at ASC, id ASC
            LIMIT $3
            "#,
        )
        .bind(&request.source_code)
        .bind(parse_optional_uuid(request.connection_id.as_deref())?)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut paused_events = Vec::with_capacity(rows.len());
        for row in rows {
            let paused_event = row_to_paused_signal_event(row)?;
            if request.event_pattern.as_deref().is_some_and(|pattern| {
                !event_type_pattern_matches(pattern, &paused_event.raw_event_type)
            }) {
                continue;
            }
            paused_events.push(paused_event);
        }

        Ok(paused_events)
    }

    pub async fn release_paused_event(&self, event_id: &str) -> Result<(), SignalHubError> {
        sqlx::query(
            r#"
            UPDATE signal_paused_events
            SET released_at = now()
            WHERE event_id = $1
              AND released_at IS NULL
            "#,
        )
        .bind(event_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_policy(&self, policy: &SignalPolicy) -> Result<Uuid, SignalHubError> {
        let id = Uuid::now_v7();
        let connection_id = match policy.connection_id.as_deref() {
            Some(value) => Some(
                Uuid::parse_str(value)
                    .map_err(|_| SignalHubError::InvalidConnectionId(value.to_owned()))?,
            ),
            None => None,
        };

        sqlx::query(
            r#"
            INSERT INTO signal_policies (
                id,
                scope,
                source_code,
                connection_id,
                event_pattern,
                mode,
                reason,
                created_by,
                expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'hermes-frontend', $8)
            "#,
        )
        .bind(id)
        .bind(scope_to_str(&policy.scope))
        .bind(&policy.source_code)
        .bind(connection_id)
        .bind(&policy.event_pattern)
        .bind(mode_to_str(&policy.mode))
        .bind(&policy.reason)
        .bind(policy.expires_at)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn expire_matching_policies(
        &self,
        policy: &SignalPolicy,
        modes: &[SignalPolicyMode],
    ) -> Result<u64, SignalHubError> {
        let connection_id = parse_optional_uuid(policy.connection_id.as_deref())?;
        let mode_values: Vec<&str> = modes.iter().map(mode_to_str).collect();
        let result = sqlx::query(
            r#"
            UPDATE signal_policies
            SET expires_at = now()
            WHERE (expires_at IS NULL OR expires_at > now())
              AND mode = ANY($1)
              AND scope = $2
              AND (
                ($3::text IS NULL AND source_code IS NULL)
                OR source_code = $3
              )
              AND (
                ($4::uuid IS NULL AND connection_id IS NULL)
                OR connection_id = $4
              )
              AND (
                ($5::text IS NULL AND event_pattern IS NULL)
                OR event_pattern = $5
              )
            "#,
        )
        .bind(mode_values)
        .bind(scope_to_str(&policy.scope))
        .bind(&policy.source_code)
        .bind(connection_id)
        .bind(&policy.event_pattern)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn create_profile_managed_policy(
        &self,
        profile_code: &str,
        policy: &SignalProfilePolicy,
    ) -> Result<Uuid, SignalHubError> {
        let id = Uuid::now_v7();
        let connection_id = match policy.connection_id.as_deref() {
            Some(value) => Some(
                Uuid::parse_str(value)
                    .map_err(|_| SignalHubError::InvalidConnectionId(value.to_owned()))?,
            ),
            None => None,
        };

        sqlx::query(
            r#"
            INSERT INTO signal_policies (
                id,
                scope,
                source_code,
                connection_id,
                event_pattern,
                mode,
                reason,
                created_by,
                expires_at,
                metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NULL, $9)
            "#,
        )
        .bind(id)
        .bind(scope_to_str(&policy.scope))
        .bind(&policy.source_code)
        .bind(connection_id)
        .bind(&policy.event_pattern)
        .bind(mode_to_str(&policy.mode))
        .bind(&policy.reason)
        .bind(format!("signal_profile:{profile_code}"))
        .bind(serde_json::json!({
            "managed_by": "signal_profile",
            "profile_code": profile_code,
        }))
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn expire_managed_profile_policies(&self) -> Result<u64, SignalHubError> {
        let result = sqlx::query(
            r#"
            UPDATE signal_policies
            SET expires_at = now()
            WHERE (metadata->>'managed_by') = 'signal_profile'
              AND (expires_at IS NULL OR expires_at > now())
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn list_active_policies(&self) -> Result<Vec<SignalPolicy>, SignalHubError> {
        let rows = sqlx::query(
            r#"
            SELECT scope, source_code, connection_id, event_pattern, mode, reason, expires_at
            FROM signal_policies
            WHERE expires_at IS NULL OR expires_at > now()
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(row_to_policy).collect()
    }

    pub async fn record_paused_event(
        &self,
        event: &EventEnvelope,
        source_code: &str,
        connection_id: Option<&str>,
        reason: &str,
    ) -> Result<(), SignalHubError> {
        let connection_id = parse_optional_uuid(connection_id)?;
        sqlx::query(
            r#"
            INSERT INTO signal_paused_events (
                id,
                event_id,
                source_code,
                connection_id,
                raw_event_type,
                event_envelope,
                reason
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (event_id) DO NOTHING
            "#,
        )
        .bind(Uuid::now_v7())
        .bind(&event.event_id)
        .bind(source_code)
        .bind(connection_id)
        .bind(&event.event_type)
        .bind(serde_json::to_value(event)?)
        .bind(reason)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn paused_event_count(&self, source_code: &str) -> Result<i64, SignalHubError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT count(*)
            FROM signal_paused_events
            WHERE source_code = $1
              AND released_at IS NULL
            "#,
        )
        .bind(source_code)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    pub async fn resolve_connection_id_for_event(
        &self,
        source_code: &str,
        event: &EventEnvelope,
    ) -> Result<Option<String>, SignalHubError> {
        let Some(account_id) = raw_signal_account_id(event) else {
            return Ok(None);
        };

        Ok(self
            .find_connection_by_account(source_code, &account_id)
            .await?
            .map(|connection| connection.id))
    }

    async fn source_by_code(&self, code: &str) -> Result<Option<SignalSource>, SignalHubError> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                code,
                display_name,
                category,
                source_kind,
                default_enabled,
                supports_connections,
                supports_runtime,
                supports_replay,
                supports_pause,
                supports_mute,
                capability_schema_version,
                created_at,
                updated_at
            FROM signal_sources
            WHERE code = $1
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_source).transpose()
    }

    async fn connection_by_id(&self, id: Uuid) -> Result<Option<SignalConnection>, SignalHubError> {
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

    async fn health_by_scope(
        &self,
        source_code: &str,
        connection_id: Option<&str>,
    ) -> Result<Option<SignalHealth>, SignalHubError> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                source_code,
                connection_id,
                level,
                summary,
                last_ok_at,
                last_failure_at,
                failure_count,
                consecutive_failure_count,
                next_retry_at,
                evidence,
                updated_at
            FROM signal_health
            WHERE source_code = $1
              AND ($2::uuid IS NULL OR connection_id = $2)
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(source_code)
        .bind(connection_id.map(parse_required_uuid).transpose()?)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_health).transpose()
    }

    async fn health_by_id(&self, id: Uuid) -> Result<Option<SignalHealth>, SignalHubError> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                source_code,
                connection_id,
                level,
                summary,
                last_ok_at,
                last_failure_at,
                failure_count,
                consecutive_failure_count,
                next_retry_at,
                evidence,
                updated_at
            FROM signal_health
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_health).transpose()
    }

    async fn replay_request_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<SignalReplayRequest>, SignalHubError> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                source_code,
                connection_id,
                event_pattern,
                status,
                requested_by,
                requested_at,
                started_at,
                completed_at,
                last_error_redacted,
                replayed_count,
                metadata
            FROM signal_replay_requests
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(row_to_replay_request).transpose()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FixtureRestoreReport {
    pub sources_created: usize,
    pub sources_repaired: usize,
    pub profiles_created: usize,
    pub profiles_repaired: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalSource {
    pub id: Uuid,
    pub code: String,
    pub display_name: String,
    pub category: String,
    pub source_kind: String,
    pub default_enabled: bool,
    pub supports_connections: bool,
    pub supports_runtime: bool,
    pub supports_replay: bool,
    pub supports_pause: bool,
    pub supports_mute: bool,
    pub capability_schema_version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalConnection {
    pub id: String,
    pub source_code: String,
    pub display_name: String,
    pub status: String,
    pub profile: Option<String>,
    pub settings: Value,
    pub secret_ref: Option<String>,
    pub connected_at: Option<DateTime<Utc>>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub last_signal_at: Option<DateTime<Utc>>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalProfile {
    pub id: String,
    pub code: String,
    pub display_name: String,
    pub description: String,
    pub source_policies: Vec<SignalProfilePolicy>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalProfileSummary {
    pub id: String,
    pub code: String,
    pub display_name: String,
    pub description: String,
    pub policy_count: usize,
    pub source_policies: Vec<SignalProfilePolicy>,
    pub is_system: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, serde::Deserialize)]
pub struct SignalProfilePolicy {
    pub scope: SignalPolicyScope,
    pub source_code: Option<String>,
    pub connection_id: Option<String>,
    pub event_pattern: Option<String>,
    pub mode: SignalPolicyMode,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalProfileCreate {
    pub code: String,
    pub display_name: String,
    pub description: String,
    pub source_policies: Vec<SignalProfilePolicy>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalProfileUpdate {
    pub code: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub source_policies: Option<Vec<SignalProfilePolicy>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalCapability {
    pub id: String,
    pub source_code: String,
    pub connection_id: Option<String>,
    pub capability: String,
    pub state: String,
    pub reason: Option<String>,
    pub requires_confirmation: bool,
    pub action_class: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalCapabilityUpsert {
    pub source_code: String,
    pub connection_id: Option<String>,
    pub capability: String,
    pub state: String,
    pub reason: Option<String>,
    pub requires_confirmation: bool,
    pub action_class: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalConnectionCreate {
    pub source_code: String,
    pub display_name: String,
    pub status: String,
    pub profile: Option<String>,
    pub settings: Value,
    pub secret_ref: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalConnectionUpdate {
    pub id: String,
    pub display_name: Option<String>,
    pub status: Option<String>,
    pub profile: Option<String>,
    pub settings: Option<Value>,
    pub secret_ref: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalHealth {
    pub id: String,
    pub source_code: String,
    pub connection_id: Option<String>,
    pub level: String,
    pub summary: String,
    pub last_ok_at: Option<DateTime<Utc>>,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub failure_count: i32,
    pub consecutive_failure_count: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub evidence: Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalHealthCheckRequest {
    pub source_code: String,
    pub connection_id: Option<String>,
    pub runtime_kind: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalHealthSnapshotWrite {
    pub level: String,
    pub summary: String,
    pub last_ok_at: Option<DateTime<Utc>>,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub failure_count: i32,
    pub consecutive_failure_count: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub evidence: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalRuntimeState {
    pub id: String,
    pub source_code: String,
    pub connection_id: Option<String>,
    pub runtime_kind: String,
    pub state: String,
    pub last_started_at: Option<DateTime<Utc>>,
    pub last_stopped_at: Option<DateTime<Utc>>,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub last_error_at: Option<DateTime<Utc>>,
    pub last_error_code: Option<String>,
    pub last_error_message_redacted: Option<String>,
    pub metadata: Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalRuntimeStateUpdate {
    pub source_code: String,
    pub runtime_kind: String,
    pub state: String,
    pub metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SignalReplayRequest {
    pub id: String,
    pub source_code: Option<String>,
    pub connection_id: Option<String>,
    pub event_pattern: Option<String>,
    pub from_position: Option<i64>,
    pub to_position: Option<i64>,
    pub from_time: Option<DateTime<Utc>>,
    pub to_time: Option<DateTime<Utc>>,
    pub target_consumer: Option<String>,
    pub target_projection: Option<String>,
    pub status: String,
    pub requested_by: String,
    pub requested_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub last_error_redacted: Option<String>,
    pub replayed_count: i32,
    pub metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalReplayRequestCreate {
    pub source_code: Option<String>,
    pub connection_id: Option<String>,
    pub event_pattern: Option<String>,
    pub from_position: Option<i64>,
    pub to_position: Option<i64>,
    pub from_time: Option<DateTime<Utc>>,
    pub to_time: Option<DateTime<Utc>>,
    pub target_consumer: Option<String>,
    pub target_projection: Option<String>,
    pub requested_by: String,
    pub metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PausedSignalEvent {
    pub id: String,
    pub event_id: String,
    pub source_code: String,
    pub raw_event_type: String,
    pub event: EventEnvelope,
    pub paused_at: DateTime<Utc>,
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

fn row_to_profile(row: PgRow) -> Result<SignalProfile, SignalHubError> {
    Ok(SignalProfile {
        id: row.try_get::<Uuid, _>("id")?.to_string(),
        code: row.try_get("code")?,
        display_name: row.try_get("display_name")?,
        description: row.try_get("description")?,
        source_policies: serde_json::from_value(row.try_get("source_policies")?)?,
        is_system: row.try_get("is_system")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_capability(row: PgRow) -> Result<SignalCapability, SignalHubError> {
    let connection_id = row.try_get::<Option<Uuid>, _>("connection_id")?;
    Ok(SignalCapability {
        id: row.try_get::<Uuid, _>("id")?.to_string(),
        source_code: row.try_get("source_code")?,
        connection_id: connection_id.map(|value| value.to_string()),
        capability: row.try_get("capability")?,
        state: row.try_get("state")?,
        reason: row.try_get("reason")?,
        requires_confirmation: row.try_get("requires_confirmation")?,
        action_class: row.try_get("action_class")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_health(row: PgRow) -> Result<SignalHealth, SignalHubError> {
    let connection_id = row.try_get::<Option<Uuid>, _>("connection_id")?;
    Ok(SignalHealth {
        id: row.try_get::<Uuid, _>("id")?.to_string(),
        source_code: row.try_get("source_code")?,
        connection_id: connection_id.map(|value| value.to_string()),
        level: row.try_get("level")?,
        summary: row.try_get("summary")?,
        last_ok_at: row.try_get("last_ok_at")?,
        last_failure_at: row.try_get("last_failure_at")?,
        failure_count: row.try_get("failure_count")?,
        consecutive_failure_count: row.try_get("consecutive_failure_count")?,
        next_retry_at: row.try_get("next_retry_at")?,
        evidence: row.try_get("evidence")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_runtime_state(row: PgRow) -> Result<SignalRuntimeState, SignalHubError> {
    let connection_id = row.try_get::<Option<Uuid>, _>("connection_id")?;
    Ok(SignalRuntimeState {
        id: row.try_get::<Uuid, _>("id")?.to_string(),
        source_code: row.try_get("source_code")?,
        connection_id: connection_id.map(|value| value.to_string()),
        runtime_kind: row.try_get("runtime_kind")?,
        state: row.try_get("state")?,
        last_started_at: row.try_get("last_started_at")?,
        last_stopped_at: row.try_get("last_stopped_at")?,
        last_heartbeat_at: row.try_get("last_heartbeat_at")?,
        last_error_at: row.try_get("last_error_at")?,
        last_error_code: row.try_get("last_error_code")?,
        last_error_message_redacted: row.try_get("last_error_message_redacted")?,
        metadata: row.try_get("metadata")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn row_to_replay_request(row: PgRow) -> Result<SignalReplayRequest, SignalHubError> {
    let connection_id = row.try_get::<Option<Uuid>, _>("connection_id")?;
    let metadata: Value = row.try_get("metadata")?;
    let selector = replay_selector_from_metadata(&metadata)?;
    Ok(SignalReplayRequest {
        id: row.try_get::<Uuid, _>("id")?.to_string(),
        source_code: row.try_get("source_code")?,
        connection_id: connection_id.map(|value| value.to_string()),
        event_pattern: row.try_get("event_pattern")?,
        from_position: selector.from_position,
        to_position: selector.to_position,
        from_time: selector.from_time,
        to_time: selector.to_time,
        target_consumer: selector.target_consumer,
        target_projection: selector.target_projection,
        status: row.try_get("status")?,
        requested_by: row.try_get("requested_by")?,
        requested_at: row.try_get("requested_at")?,
        started_at: row.try_get("started_at")?,
        completed_at: row.try_get("completed_at")?,
        last_error_redacted: row.try_get("last_error_redacted")?,
        replayed_count: row.try_get("replayed_count")?,
        metadata,
    })
}

#[derive(Clone, Debug, Default)]
struct ReplaySelector {
    from_position: Option<i64>,
    to_position: Option<i64>,
    from_time: Option<DateTime<Utc>>,
    to_time: Option<DateTime<Utc>>,
    target_consumer: Option<String>,
    target_projection: Option<String>,
}

fn replay_selector_from_metadata(metadata: &Value) -> Result<ReplaySelector, SignalHubError> {
    let from_position = metadata.get("from_position").and_then(Value::as_i64);
    let to_position = metadata.get("to_position").and_then(Value::as_i64);
    let from_time = metadata
        .get("from_time")
        .and_then(Value::as_str)
        .map(parse_replay_timestamp)
        .transpose()?;
    let to_time = metadata
        .get("to_time")
        .and_then(Value::as_str)
        .map(parse_replay_timestamp)
        .transpose()?;
    let target_consumer = metadata
        .get("target_consumer")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let target_projection = metadata
        .get("target_projection")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    Ok(ReplaySelector {
        from_position,
        to_position,
        from_time,
        to_time,
        target_consumer,
        target_projection,
    })
}

fn build_replay_metadata(
    mut metadata: Value,
    from_position: Option<i64>,
    to_position: Option<i64>,
    from_time: Option<DateTime<Utc>>,
    to_time: Option<DateTime<Utc>>,
    target_consumer: Option<&str>,
    target_projection: Option<&str>,
) -> Result<Value, SignalHubError> {
    let metadata_object = metadata.as_object_mut().ok_or_else(|| {
        SignalHubError::InvalidReplayRequest("metadata must be a JSON object".to_owned())
    })?;

    if let Some(from_position) = from_position {
        metadata_object.insert("from_position".to_owned(), Value::from(from_position));
    }
    if let Some(to_position) = to_position {
        metadata_object.insert("to_position".to_owned(), Value::from(to_position));
    }
    if let Some(from_time) = from_time {
        metadata_object.insert(
            "from_time".to_owned(),
            Value::String(from_time.to_rfc3339()),
        );
    }
    if let Some(to_time) = to_time {
        metadata_object.insert("to_time".to_owned(), Value::String(to_time.to_rfc3339()));
    }
    if let Some(target_consumer) = target_consumer {
        metadata_object.insert(
            "target_consumer".to_owned(),
            Value::String(target_consumer.to_owned()),
        );
    }
    if let Some(target_projection) = target_projection {
        metadata_object.insert(
            "target_projection".to_owned(),
            Value::String(target_projection.to_owned()),
        );
    }

    Ok(metadata)
}

fn parse_replay_timestamp(value: &str) -> Result<DateTime<Utc>, SignalHubError> {
    DateTime::parse_from_rfc3339(value.trim())
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| {
            SignalHubError::InvalidReplayRequest(format!(
                "invalid replay timestamp `{value}`: {error}"
            ))
        })
}

fn raw_signal_account_id(event: &EventEnvelope) -> Option<String> {
    string_field(&event.source, "account_id")
        .or_else(|| string_field(&event.subject, "account_id"))
        .or_else(|| string_field(&event.provenance, "account_id"))
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn row_to_paused_signal_event(row: PgRow) -> Result<PausedSignalEvent, SignalHubError> {
    let envelope: Value = row.try_get("event_envelope")?;
    Ok(PausedSignalEvent {
        id: row.try_get::<Uuid, _>("id")?.to_string(),
        event_id: row.try_get("event_id")?,
        source_code: row.try_get("source_code")?,
        raw_event_type: row.try_get("raw_event_type")?,
        event: serde_json::from_value(envelope)?,
        paused_at: row.try_get("paused_at")?,
    })
}

fn source_matches_fixture(source: &SignalSource, fixture: &SystemSourceFixture) -> bool {
    source.display_name == fixture.display_name
        && source.category == fixture.category
        && source.source_kind == fixture.source_kind
        && source.default_enabled == fixture.default_enabled
        && source.supports_connections == fixture.supports_connections
        && source.supports_runtime == fixture.supports_runtime
        && source.supports_replay == fixture.supports_replay
        && source.supports_pause == fixture.supports_pause
        && source.supports_mute == fixture.supports_mute
        && source.capability_schema_version == 1
}

fn profile_matches_fixture(profile: &SignalProfile, fixture: &SystemProfileFixture) -> bool {
    match profile_policies_json(fixture) {
        Ok(policies) => {
            profile.display_name == fixture.display_name
                && profile.description == fixture.description
                && profile.is_system == fixture.is_system
                && serde_json::to_value(&profile.source_policies).ok() == Some(policies)
        }
        Err(_) => false,
    }
}

fn profile_policies_json(fixture: &SystemProfileFixture) -> Result<Value, SignalHubError> {
    let policies = fixture
        .policies
        .iter()
        .map(|policy| SignalProfilePolicy {
            scope: policy.scope.clone(),
            source_code: policy.source_code.map(ToOwned::to_owned),
            connection_id: None,
            event_pattern: policy.event_pattern.map(ToOwned::to_owned),
            mode: policy.mode.clone(),
            reason: policy.reason.to_owned(),
        })
        .collect::<Vec<_>>();

    Ok(serde_json::to_value(policies)?)
}

fn row_to_policy(row: PgRow) -> Result<SignalPolicy, SignalHubError> {
    let scope: String = row.try_get("scope")?;
    let mode: String = row.try_get("mode")?;
    let connection_id: Option<Uuid> = row.try_get("connection_id")?;

    Ok(SignalPolicy {
        scope: scope_from_str(&scope)?,
        source_code: row.try_get("source_code")?,
        connection_id: connection_id.map(|value| value.to_string()),
        event_pattern: row.try_get("event_pattern")?,
        mode: mode_from_str(&mode)?,
        reason: row.try_get("reason")?,
        expires_at: row.try_get("expires_at")?,
    })
}

fn scope_to_str(scope: &SignalPolicyScope) -> &'static str {
    scope.as_str()
}

fn scope_from_str(value: &str) -> Result<SignalPolicyScope, SignalHubError> {
    SignalPolicyScope::parse(value)
        .ok_or_else(|| SignalHubError::InvalidPolicyScope(value.to_owned()))
}

fn mode_to_str(mode: &SignalPolicyMode) -> &'static str {
    mode.as_str()
}

fn mode_from_str(value: &str) -> Result<SignalPolicyMode, SignalHubError> {
    SignalPolicyMode::parse(value)
        .ok_or_else(|| SignalHubError::InvalidPolicyMode(value.to_owned()))
}

struct SignalHealthSnapshot {
    level: String,
    summary: String,
    last_ok_at: Option<DateTime<Utc>>,
    last_failure_at: Option<DateTime<Utc>>,
    failure_count: i32,
    consecutive_failure_count: i32,
    next_retry_at: Option<DateTime<Utc>>,
    evidence: Value,
}

impl From<SignalHealthSnapshot> for SignalHealthSnapshotWrite {
    fn from(value: SignalHealthSnapshot) -> Self {
        Self {
            level: value.level,
            summary: value.summary,
            last_ok_at: value.last_ok_at,
            last_failure_at: value.last_failure_at,
            failure_count: value.failure_count,
            consecutive_failure_count: value.consecutive_failure_count,
            next_retry_at: value.next_retry_at,
            evidence: value.evidence,
        }
    }
}

fn build_health_snapshot(
    source: &SignalSource,
    connection: Option<&SignalConnection>,
    runtime: Option<&SignalRuntimeState>,
) -> SignalHealthSnapshot {
    if let Some(connection) = connection {
        match connection.status.as_str() {
            "disabled" => {
                return SignalHealthSnapshot {
                    level: "disabled".to_owned(),
                    summary: format!("{} connection is disabled", source.display_name),
                    last_ok_at: None,
                    last_failure_at: Some(Utc::now()),
                    failure_count: 1,
                    consecutive_failure_count: 1,
                    next_retry_at: None,
                    evidence: serde_json::json!({
                        "source_status": connection.status,
                        "connection_id": connection.id,
                    }),
                };
            }
            "error" | "disconnected" | "degraded" => {
                return SignalHealthSnapshot {
                    level: "degraded".to_owned(),
                    summary: format!("{} connection requires attention", source.display_name),
                    last_ok_at: None,
                    last_failure_at: Some(Utc::now()),
                    failure_count: 1,
                    consecutive_failure_count: 1,
                    next_retry_at: Some(Utc::now() + chrono::Duration::minutes(5)),
                    evidence: serde_json::json!({
                        "source_status": connection.status,
                        "connection_id": connection.id,
                    }),
                };
            }
            _ => {}
        }
    }

    if let Some(runtime) = runtime {
        return match runtime.state.as_str() {
            "running" => SignalHealthSnapshot {
                level: "healthy".to_owned(),
                summary: format!("{} runtime is healthy", source.display_name),
                last_ok_at: Some(Utc::now()),
                last_failure_at: None,
                failure_count: 0,
                consecutive_failure_count: 0,
                next_retry_at: None,
                evidence: serde_json::json!({
                    "runtime_kind": runtime.runtime_kind,
                    "runtime_state": runtime.state,
                    "source_code": source.code,
                }),
            },
            "paused" | "muted" | "stopped" => SignalHealthSnapshot {
                level: "degraded".to_owned(),
                summary: format!("{} runtime is {}", source.display_name, runtime.state),
                last_ok_at: None,
                last_failure_at: Some(Utc::now()),
                failure_count: 1,
                consecutive_failure_count: 1,
                next_retry_at: None,
                evidence: serde_json::json!({
                    "runtime_kind": runtime.runtime_kind,
                    "runtime_state": runtime.state,
                    "source_code": source.code,
                }),
            },
            _ => SignalHealthSnapshot {
                level: "unknown".to_owned(),
                summary: format!("{} runtime state is {}", source.display_name, runtime.state),
                last_ok_at: None,
                last_failure_at: None,
                failure_count: 0,
                consecutive_failure_count: 0,
                next_retry_at: None,
                evidence: serde_json::json!({
                    "runtime_kind": runtime.runtime_kind,
                    "runtime_state": runtime.state,
                    "source_code": source.code,
                }),
            },
        };
    }

    SignalHealthSnapshot {
        level: if source.supports_runtime {
            "unknown".to_owned()
        } else {
            "healthy".to_owned()
        },
        summary: if source.supports_runtime {
            format!("{} runtime is not registered", source.display_name)
        } else {
            format!("{} source has no runtime checks", source.display_name)
        },
        last_ok_at: (!source.supports_runtime).then(Utc::now),
        last_failure_at: None,
        failure_count: 0,
        consecutive_failure_count: 0,
        next_retry_at: None,
        evidence: serde_json::json!({
            "source_code": source.code,
            "supports_runtime": source.supports_runtime,
        }),
    }
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<String, SignalHubError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(SignalHubError::EmptyField(field));
    }
    Ok(value.to_owned())
}

fn validate_object(field: &'static str, value: &Value) -> Result<(), SignalHubError> {
    if value.is_object() {
        return Ok(());
    }
    Err(SignalHubError::EmptyField(field))
}

fn parse_required_uuid(value: &str) -> Result<Uuid, SignalHubError> {
    Uuid::parse_str(value.trim()).map_err(|_| SignalHubError::InvalidConnectionId(value.to_owned()))
}

fn validate_profile_policies(
    policies: &[SignalProfilePolicy],
) -> Result<Vec<SignalProfilePolicy>, SignalHubError> {
    policies
        .iter()
        .map(|policy| {
            if policy.scope == SignalPolicyScope::Profile {
                return Err(SignalHubError::InvalidProfileDefinition(
                    "profile-managed policies cannot use profile scope".to_owned(),
                ));
            }

            let reason = validate_non_empty("reason", &policy.reason)?;
            let source_code = policy
                .source_code
                .as_deref()
                .map(|value| validate_non_empty("source_code", value))
                .transpose()?;
            let connection_id = match policy.connection_id.as_deref() {
                Some(value) => {
                    let normalized = validate_non_empty("connection_id", value)?;
                    parse_required_uuid(&normalized)?;
                    Some(normalized)
                }
                None => None,
            };
            let event_pattern = policy
                .event_pattern
                .as_deref()
                .map(|value| validate_non_empty("event_pattern", value))
                .transpose()?;

            match policy.scope {
                SignalPolicyScope::Global => {}
                SignalPolicyScope::Source => {
                    if source_code.is_none() {
                        return Err(SignalHubError::InvalidProfileDefinition(
                            "source profile policy requires source_code".to_owned(),
                        ));
                    }
                }
                SignalPolicyScope::Connection => {
                    if source_code.is_none() || connection_id.is_none() {
                        return Err(SignalHubError::InvalidProfileDefinition(
                            "connection profile policy requires source_code and connection_id"
                                .to_owned(),
                        ));
                    }
                }
                SignalPolicyScope::EventPattern => {
                    if event_pattern.is_none() {
                        return Err(SignalHubError::InvalidProfileDefinition(
                            "event_pattern profile policy requires event_pattern".to_owned(),
                        ));
                    }
                }
                SignalPolicyScope::Profile => unreachable!(),
            }

            Ok(SignalProfilePolicy {
                scope: policy.scope.clone(),
                source_code,
                connection_id,
                event_pattern,
                mode: policy.mode.clone(),
                reason,
            })
        })
        .collect()
}

fn runtime_state_value(value: &str) -> Result<&str, SignalHubError> {
    match value.trim() {
        "stopped" | "starting" | "running" | "reconnecting" | "paused" | "muted" | "stopping"
        | "error" => Ok(value.trim()),
        other => Err(SignalHubError::InvalidRuntimeState(other.to_owned())),
    }
}

fn connection_status_value(value: &str) -> Result<&str, SignalHubError> {
    match value.trim() {
        "not_configured"
        | "connecting"
        | "awaiting_user_action"
        | "connected"
        | "degraded"
        | "disconnected"
        | "paused"
        | "muted"
        | "disabled"
        | "error"
        | "removed" => Ok(value.trim()),
        other => Err(SignalHubError::InvalidConnectionStatus(other.to_owned())),
    }
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::Database(db_error) if db_error.is_unique_violation())
}

fn parse_optional_uuid(value: Option<&str>) -> Result<Option<Uuid>, SignalHubError> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            Uuid::parse_str(value)
                .map_err(|_| SignalHubError::InvalidConnectionId(value.to_owned()))
        })
        .transpose()
}

fn truncate_redacted_error(error: &str) -> String {
    let trimmed = error.trim();
    if trimmed.chars().count() <= 500 {
        return trimmed.to_owned();
    }

    trimmed.chars().take(500).collect()
}

pub(crate) fn event_type_pattern_matches(pattern: &str, event_type: &str) -> bool {
    if pattern == event_type {
        return true;
    }

    let Some(prefix) = pattern.strip_suffix(".*") else {
        return false;
    };

    event_type.starts_with(prefix)
}
