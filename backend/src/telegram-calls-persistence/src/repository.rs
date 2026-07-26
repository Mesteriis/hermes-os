use hermes_telegram_calls_core::{
    TelegramCallDirection, TelegramCallDiscardReason, TelegramCallFailureCategory,
    TelegramCallProjectionError, TelegramCallSession, TelegramProviderCallState,
    TelegramProviderCallUpdate, project_provider_call_update,
};
use sqlx::{PgPool, Postgres, Row, Transaction};

#[derive(Clone)]
pub struct TelegramCallsPersistence {
    pool: PgPool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersistedCallUpdate {
    pub session: TelegramCallSession,
    pub frame_sequence: Option<u64>,
    pub replayed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TelegramCallFrame {
    pub sequence: u64,
    pub session: TelegramCallSession,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TelegramCallsPersistenceError {
    Database,
    InvalidRow,
    IdentityConflict,
    StateRegression,
    TerminalConflict,
    InvalidRequest(&'static str),
}

impl From<TelegramCallProjectionError> for TelegramCallsPersistenceError {
    fn from(value: TelegramCallProjectionError) -> Self {
        match value {
            TelegramCallProjectionError::InvalidRequest(field) => Self::InvalidRequest(field),
            TelegramCallProjectionError::IdentityConflict => Self::IdentityConflict,
            TelegramCallProjectionError::StateRegression => Self::StateRegression,
            TelegramCallProjectionError::TerminalConflict => Self::TerminalConflict,
        }
    }
}

impl TelegramCallsPersistence {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn apply_schema_for_conformance(&self) -> Result<(), TelegramCallsPersistenceError> {
        sqlx::raw_sql(crate::schema::TELEGRAM_CALLS_SCHEMA_V1)
            .execute(&self.pool)
            .await
            .map_err(|_| TelegramCallsPersistenceError::Database)?;
        Ok(())
    }

    pub async fn ingest_provider_update(
        &self,
        new_call_session_id: &str,
        update: &TelegramProviderCallUpdate,
    ) -> Result<PersistedCallUpdate, TelegramCallsPersistenceError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| TelegramCallsPersistenceError::Database)?;
        let current = load_for_update(&mut transaction, update).await?;
        let projected =
            project_provider_call_update(current.as_ref(), new_call_session_id, update)?;

        if !projected.changed {
            transaction
                .commit()
                .await
                .map_err(|_| TelegramCallsPersistenceError::Database)?;
            return Ok(PersistedCallUpdate {
                session: projected.session,
                frame_sequence: None,
                replayed: true,
            });
        }

        persist_session(&mut transaction, current.as_ref(), &projected.session).await?;
        persist_history(&mut transaction, &projected.session).await?;
        let frame_sequence = persist_frame(&mut transaction, &projected.session).await?;
        transaction
            .commit()
            .await
            .map_err(|_| TelegramCallsPersistenceError::Database)?;

        Ok(PersistedCallUpdate {
            session: projected.session,
            frame_sequence: Some(frame_sequence),
            replayed: false,
        })
    }

    pub async fn call(
        &self,
        account_id: &str,
        call_session_id: &str,
    ) -> Result<Option<TelegramCallSession>, TelegramCallsPersistenceError> {
        let row = sqlx::query(
            "SELECT call_session_id, account_id, runtime_generation, tdlib_call_id, \
             provider_call_unique_id, provider_user_id, direction, provider_state, \
             pending_created, pending_received, discard_reason, failure_category, revision, \
             created_at_unix_seconds, updated_at_unix_seconds, ended_at_unix_seconds \
             FROM hermes_data.telegram_call_sessions \
             WHERE account_id = $1 AND call_session_id = $2",
        )
        .bind(account_id)
        .bind(call_session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramCallsPersistenceError::Database)?;
        row.as_ref().map(session_from_row).transpose()
    }

    pub async fn active_call(
        &self,
        account_id: &str,
    ) -> Result<Option<TelegramCallSession>, TelegramCallsPersistenceError> {
        let row = sqlx::query(
            "SELECT call_session_id, account_id, runtime_generation, tdlib_call_id, \
             provider_call_unique_id, provider_user_id, direction, provider_state, \
             pending_created, pending_received, discard_reason, failure_category, revision, \
             created_at_unix_seconds, updated_at_unix_seconds, ended_at_unix_seconds \
             FROM hermes_data.telegram_call_sessions \
             WHERE account_id = $1 AND provider_state NOT IN ('discarded', 'error') \
             ORDER BY created_at_unix_seconds DESC, call_session_id DESC LIMIT 1",
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| TelegramCallsPersistenceError::Database)?;
        row.as_ref().map(session_from_row).transpose()
    }

    pub async fn list_calls(
        &self,
        account_id: &str,
        after_call_session_id: &str,
        limit: u32,
    ) -> Result<Vec<TelegramCallSession>, TelegramCallsPersistenceError> {
        let limit = validated_limit(limit)?;
        let rows = sqlx::query(
            "SELECT call_session_id, account_id, runtime_generation, tdlib_call_id, \
             provider_call_unique_id, provider_user_id, direction, provider_state, \
             pending_created, pending_received, discard_reason, failure_category, revision, \
             created_at_unix_seconds, updated_at_unix_seconds, ended_at_unix_seconds \
             FROM hermes_data.telegram_call_sessions \
             WHERE account_id = $1 AND ($2 = '' OR call_session_id > $2) \
             ORDER BY call_session_id ASC LIMIT $3",
        )
        .bind(account_id)
        .bind(after_call_session_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramCallsPersistenceError::Database)?;
        rows.iter().map(session_from_row).collect()
    }

    pub async fn realtime_after(
        &self,
        account_id: &str,
        after_sequence: u64,
        limit: u32,
    ) -> Result<Vec<TelegramCallFrame>, TelegramCallsPersistenceError> {
        let limit = validated_limit(limit)?;
        let after_sequence = i64::try_from(after_sequence)
            .map_err(|_| TelegramCallsPersistenceError::InvalidRequest("after_sequence"))?;
        let rows = sqlx::query(
            "SELECT f.frame_sequence, s.call_session_id, s.account_id, s.runtime_generation, \
             s.tdlib_call_id, s.provider_call_unique_id, s.provider_user_id, s.direction, \
             f.provider_state, f.pending_created, f.pending_received, f.discard_reason, \
             f.failure_category, f.call_revision AS revision, s.created_at_unix_seconds, \
             f.observed_at_unix_seconds AS updated_at_unix_seconds, \
             CASE WHEN f.provider_state IN ('discarded', 'error') \
                  THEN f.observed_at_unix_seconds ELSE NULL END AS ended_at_unix_seconds \
             FROM hermes_data.telegram_call_realtime_frames f \
             JOIN hermes_data.telegram_call_sessions s \
               ON s.call_session_id = f.call_session_id \
             WHERE f.account_id = $1 AND f.frame_sequence > $2 \
             ORDER BY f.frame_sequence ASC LIMIT $3",
        )
        .bind(account_id)
        .bind(after_sequence)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|_| TelegramCallsPersistenceError::Database)?;

        rows.iter()
            .map(|row| {
                Ok(TelegramCallFrame {
                    sequence: from_i64(row.try_get("frame_sequence").map_err(database_error)?)?,
                    session: session_from_row(row)?,
                })
            })
            .collect()
    }
}

async fn load_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    update: &TelegramProviderCallUpdate,
) -> Result<Option<TelegramCallSession>, TelegramCallsPersistenceError> {
    let runtime_generation = as_i64(update.runtime_generation)?;
    let row = sqlx::query(
        "SELECT call_session_id, account_id, runtime_generation, tdlib_call_id, \
         provider_call_unique_id, provider_user_id, direction, provider_state, \
         pending_created, pending_received, discard_reason, failure_category, revision, \
         created_at_unix_seconds, updated_at_unix_seconds, ended_at_unix_seconds \
         FROM hermes_data.telegram_call_sessions \
         WHERE account_id = $1 AND ( \
           (runtime_generation = $2 AND tdlib_call_id = $3) OR \
           ($4::BIGINT IS NOT NULL AND provider_call_unique_id = $4) \
         ) \
         ORDER BY provider_call_unique_id IS NOT NULL DESC \
         LIMIT 1 FOR UPDATE",
    )
    .bind(&update.account_id)
    .bind(runtime_generation)
    .bind(update.tdlib_call_id)
    .bind(update.provider_call_unique_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsPersistenceError::Database)?;
    row.as_ref().map(session_from_row).transpose()
}

async fn persist_session(
    transaction: &mut Transaction<'_, Postgres>,
    current: Option<&TelegramCallSession>,
    session: &TelegramCallSession,
) -> Result<(), TelegramCallsPersistenceError> {
    let direction = match session.direction {
        TelegramCallDirection::Incoming => "incoming",
        TelegramCallDirection::Outgoing => "outgoing",
    };
    let discard_reason = session
        .discard_reason
        .map(TelegramCallDiscardReason::storage_name);
    let failure_category = session
        .failure_category
        .map(TelegramCallFailureCategory::storage_name);

    if let Some(current) = current {
        let result = sqlx::query(
            "UPDATE hermes_data.telegram_call_sessions SET \
             provider_call_unique_id = $1, provider_state = $2, pending_created = $3, \
             pending_received = $4, discard_reason = $5, failure_category = $6, revision = $7, \
             updated_at_unix_seconds = $8, ended_at_unix_seconds = $9 \
             WHERE call_session_id = $10 AND revision = $11",
        )
        .bind(session.provider_call_unique_id)
        .bind(session.state.storage_name())
        .bind(session.pending_created)
        .bind(session.pending_received)
        .bind(discard_reason)
        .bind(failure_category)
        .bind(as_i64(session.revision)?)
        .bind(as_i64(session.updated_at_unix_seconds)?)
        .bind(optional_i64(session.ended_at_unix_seconds)?)
        .bind(&session.call_session_id)
        .bind(as_i64(current.revision)?)
        .execute(&mut **transaction)
        .await
        .map_err(|_| TelegramCallsPersistenceError::Database)?;
        if result.rows_affected() != 1 {
            return Err(TelegramCallsPersistenceError::StateRegression);
        }
    } else {
        sqlx::query(
            "INSERT INTO hermes_data.telegram_call_sessions ( \
             call_session_id, account_id, runtime_generation, tdlib_call_id, \
             provider_call_unique_id, provider_user_id, direction, provider_state, \
             pending_created, pending_received, discard_reason, failure_category, revision, \
             created_at_unix_seconds, updated_at_unix_seconds, ended_at_unix_seconds \
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)",
        )
        .bind(&session.call_session_id)
        .bind(&session.account_id)
        .bind(as_i64(session.runtime_generation)?)
        .bind(session.tdlib_call_id)
        .bind(session.provider_call_unique_id)
        .bind(&session.provider_user_id)
        .bind(direction)
        .bind(session.state.storage_name())
        .bind(session.pending_created)
        .bind(session.pending_received)
        .bind(discard_reason)
        .bind(failure_category)
        .bind(as_i64(session.revision)?)
        .bind(as_i64(session.created_at_unix_seconds)?)
        .bind(as_i64(session.updated_at_unix_seconds)?)
        .bind(optional_i64(session.ended_at_unix_seconds)?)
        .execute(&mut **transaction)
        .await
        .map_err(|_| TelegramCallsPersistenceError::Database)?;
    }
    Ok(())
}

async fn persist_history(
    transaction: &mut Transaction<'_, Postgres>,
    session: &TelegramCallSession,
) -> Result<(), TelegramCallsPersistenceError> {
    sqlx::query(
        "INSERT INTO hermes_data.telegram_call_state_history ( \
         call_session_id, revision, provider_state, pending_created, pending_received, \
         discard_reason, failure_category, observed_at_unix_seconds \
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(&session.call_session_id)
    .bind(as_i64(session.revision)?)
    .bind(session.state.storage_name())
    .bind(session.pending_created)
    .bind(session.pending_received)
    .bind(
        session
            .discard_reason
            .map(TelegramCallDiscardReason::storage_name),
    )
    .bind(
        session
            .failure_category
            .map(TelegramCallFailureCategory::storage_name),
    )
    .bind(as_i64(session.updated_at_unix_seconds)?)
    .execute(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsPersistenceError::Database)?;
    Ok(())
}

async fn persist_frame(
    transaction: &mut Transaction<'_, Postgres>,
    session: &TelegramCallSession,
) -> Result<u64, TelegramCallsPersistenceError> {
    let row = sqlx::query(
        "INSERT INTO hermes_data.telegram_call_realtime_frames ( \
         account_id, call_session_id, call_revision, provider_state, pending_created, \
         pending_received, discard_reason, failure_category, observed_at_unix_seconds \
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING frame_sequence",
    )
    .bind(&session.account_id)
    .bind(&session.call_session_id)
    .bind(as_i64(session.revision)?)
    .bind(session.state.storage_name())
    .bind(session.pending_created)
    .bind(session.pending_received)
    .bind(
        session
            .discard_reason
            .map(TelegramCallDiscardReason::storage_name),
    )
    .bind(
        session
            .failure_category
            .map(TelegramCallFailureCategory::storage_name),
    )
    .bind(as_i64(session.updated_at_unix_seconds)?)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| TelegramCallsPersistenceError::Database)?;
    from_i64(row.try_get("frame_sequence").map_err(database_error)?)
}

fn session_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<TelegramCallSession, TelegramCallsPersistenceError> {
    let direction = match row
        .try_get::<String, _>("direction")
        .map_err(database_error)?
        .as_str()
    {
        "incoming" => TelegramCallDirection::Incoming,
        "outgoing" => TelegramCallDirection::Outgoing,
        _ => return Err(TelegramCallsPersistenceError::InvalidRow),
    };
    let state_name = row
        .try_get::<String, _>("provider_state")
        .map_err(database_error)?;
    let state = TelegramProviderCallState::from_storage_name(&state_name)
        .ok_or(TelegramCallsPersistenceError::InvalidRow)?;
    let discard_reason = row
        .try_get::<Option<String>, _>("discard_reason")
        .map_err(database_error)?
        .map(|value| {
            TelegramCallDiscardReason::from_storage_name(&value)
                .ok_or(TelegramCallsPersistenceError::InvalidRow)
        })
        .transpose()?;
    let failure_category = row
        .try_get::<Option<String>, _>("failure_category")
        .map_err(database_error)?
        .map(|value| {
            TelegramCallFailureCategory::from_storage_name(&value)
                .ok_or(TelegramCallsPersistenceError::InvalidRow)
        })
        .transpose()?;

    Ok(TelegramCallSession {
        call_session_id: row.try_get("call_session_id").map_err(database_error)?,
        account_id: row.try_get("account_id").map_err(database_error)?,
        runtime_generation: from_i64(row.try_get("runtime_generation").map_err(database_error)?)?,
        tdlib_call_id: row.try_get("tdlib_call_id").map_err(database_error)?,
        provider_call_unique_id: row
            .try_get("provider_call_unique_id")
            .map_err(database_error)?,
        provider_user_id: row.try_get("provider_user_id").map_err(database_error)?,
        direction,
        state,
        pending_created: row.try_get("pending_created").map_err(database_error)?,
        pending_received: row.try_get("pending_received").map_err(database_error)?,
        discard_reason,
        failure_category,
        revision: from_i64(row.try_get("revision").map_err(database_error)?)?,
        created_at_unix_seconds: from_i64(
            row.try_get("created_at_unix_seconds")
                .map_err(database_error)?,
        )?,
        updated_at_unix_seconds: from_i64(
            row.try_get("updated_at_unix_seconds")
                .map_err(database_error)?,
        )?,
        ended_at_unix_seconds: optional_u64(
            row.try_get("ended_at_unix_seconds")
                .map_err(database_error)?,
        )?,
    })
}

fn validated_limit(limit: u32) -> Result<u32, TelegramCallsPersistenceError> {
    if (1..=200).contains(&limit) {
        Ok(limit)
    } else {
        Err(TelegramCallsPersistenceError::InvalidRequest("limit"))
    }
}

fn as_i64(value: u64) -> Result<i64, TelegramCallsPersistenceError> {
    i64::try_from(value).map_err(|_| TelegramCallsPersistenceError::InvalidRow)
}

fn optional_i64(value: Option<u64>) -> Result<Option<i64>, TelegramCallsPersistenceError> {
    value.map(as_i64).transpose()
}

fn from_i64(value: i64) -> Result<u64, TelegramCallsPersistenceError> {
    u64::try_from(value).map_err(|_| TelegramCallsPersistenceError::InvalidRow)
}

fn optional_u64(value: Option<i64>) -> Result<Option<u64>, TelegramCallsPersistenceError> {
    value.map(from_i64).transpose()
}

fn database_error(_: sqlx::Error) -> TelegramCallsPersistenceError {
    TelegramCallsPersistenceError::Database
}
