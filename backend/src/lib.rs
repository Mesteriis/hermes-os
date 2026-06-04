pub mod audit;
pub mod communications;
pub mod config;
pub mod contacts;
pub mod documents;
pub mod email_import;
pub mod email_provider_network;
pub mod email_sources;
pub mod email_sync;
pub mod event_log;
pub mod messages;
pub mod projections;
pub mod search;
pub mod secrets;
pub mod storage;

use std::io;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use crate::audit::{ApiAuditError, ApiAuditLog, ApiAuditRecord, NewApiAuditRecord};
use crate::config::AppConfig;
use crate::event_log::{
    EventEnvelope, EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope,
};
use crate::storage::{
    Database, DatabaseReadiness, MigrationReadiness, ReadinessStatus, StorageError,
};

const LOCAL_API_ACTOR_ID_HEADER: &str = "x-hermes-actor-id";
const MAX_LOCAL_API_ACTOR_ID_LENGTH: usize = 128;

#[derive(Clone)]
struct AppState {
    config: AppConfig,
    database: Database,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: String,
}

pub fn build_router(config: AppConfig) -> Router {
    build_router_with_database(config, Database::disabled())
}

pub fn build_router_with_database(config: AppConfig, database: Database) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/api/v1/status", get(get_v1_status))
        .route("/api/audit/events", get(get_audit_events))
        .route("/api/events", post(post_event))
        .route("/api/events/{event_id}", get(get_event))
        .with_state(AppState { config, database })
}

pub async fn run(config: AppConfig) -> Result<(), AppError> {
    let http_addr = config.http_addr();
    let database = Database::connect(config.database_url()).await?;
    let listener = TcpListener::bind(http_addr).await?;

    tracing::info!(%http_addr, "starting Hermes Hub backend");

    axum::serve(listener, build_router_with_database(config, database)).await?;

    Ok(())
}

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

async fn healthz(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: state.config.service_name().to_owned(),
    })
}

async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<ReadinessResponse>) {
    let database = state.database.readiness().await;
    let migrations = state.database.migration_readiness().await;
    let is_ready =
        database.status() == ReadinessStatus::Ok && migrations.status() == ReadinessStatus::Ok;

    let status_code = if is_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(ReadinessResponse {
            status: if is_ready { "ok" } else { "degraded" },
            service: state.config.service_name().to_owned(),
            checks: ReadinessChecks {
                database,
                migrations,
            },
        }),
    )
}

async fn post_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AppendEventRequest>,
) -> Result<(StatusCode, Json<AppendEventResponse>), ApiError> {
    let actor = verify_local_api_capability(&state.config, &headers)?;

    let store = event_store(&state)?;
    let event = request.into_new_event()?;
    let audit_log = api_audit_log(&state)?;
    audit_log
        .record(&NewApiAuditRecord::event_append(
            actor.actor_id,
            event.event_id.clone(),
        ))
        .await?;
    let position = store.append(&event).await?;

    Ok((
        StatusCode::CREATED,
        Json(AppendEventResponse {
            event_id: event.event_id,
            position,
        }),
    ))
}

async fn get_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(event_id): Path<String>,
) -> Result<Json<EventEnvelope>, ApiError> {
    let actor = verify_local_api_capability(&state.config, &headers)?;

    let store = event_store(&state)?;
    let audit_log = api_audit_log(&state)?;
    audit_log
        .record(&NewApiAuditRecord::event_get(
            actor.actor_id,
            event_id.clone(),
        ))
        .await?;
    let Some(event) = store.get_by_id(&event_id).await? else {
        return Err(ApiError::NotFound);
    };

    Ok(Json(event))
}

async fn get_audit_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuditEventsQuery>,
) -> Result<Json<AuditEventsResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;

    let audit_log = api_audit_log(&state)?;
    let items = audit_log
        .list_event_records(
            query.target_id.as_deref(),
            query.actor_id.as_deref(),
            query.after_audit_id.unwrap_or(0),
            query.limit.unwrap_or(100),
        )
        .await?;

    Ok(Json(AuditEventsResponse { items }))
}

async fn get_v1_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<V1StatusResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let Some(_pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(Json(V1StatusResponse {
        version: "1.0",
        surfaces: V1Surfaces {
            messages: true,
            contacts: true,
            search: true,
            documents: true,
        },
    }))
}

fn event_store(state: &AppState) -> Result<EventStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(EventStore::new(pool.clone()))
}

fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApiAuditLog::new(pool.clone()))
}

fn verify_local_api_capability(
    config: &AppConfig,
    headers: &HeaderMap,
) -> Result<LocalApiActor, ApiError> {
    let Some(expected_token) = config.local_api_token() else {
        return Err(ApiError::ApiTokenNotConfigured);
    };

    let Some(raw_authorization) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    else {
        return Err(ApiError::InvalidApiToken);
    };

    let Some((scheme, token)) = raw_authorization.split_once(' ') else {
        return Err(ApiError::InvalidApiToken);
    };

    if !scheme.eq_ignore_ascii_case("Bearer") || token != expected_token {
        return Err(ApiError::InvalidApiToken);
    }

    local_api_actor(headers)
}

fn local_api_actor(headers: &HeaderMap) -> Result<LocalApiActor, ApiError> {
    let Some(raw_actor_id) = headers
        .get(LOCAL_API_ACTOR_ID_HEADER)
        .and_then(|value| value.to_str().ok())
    else {
        return Err(ApiError::InvalidActorId);
    };

    let actor_id = raw_actor_id.trim();
    if actor_id.is_empty()
        || actor_id.len() > MAX_LOCAL_API_ACTOR_ID_LENGTH
        || !actor_id.bytes().all(is_valid_actor_id_byte)
    {
        return Err(ApiError::InvalidActorId);
    }

    Ok(LocalApiActor {
        actor_id: actor_id.to_owned(),
    })
}

fn is_valid_actor_id_byte(byte: u8) -> bool {
    matches!(
        byte,
        b'a'..=b'z'
            | b'A'..=b'Z'
            | b'0'..=b'9'
            | b'.'
            | b'_'
            | b'-'
            | b':'
            | b'@'
            | b'/'
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LocalApiActor {
    actor_id: String,
}

#[derive(Serialize)]
struct ReadinessResponse {
    status: &'static str,
    service: String,
    checks: ReadinessChecks,
}

#[derive(Serialize)]
struct ReadinessChecks {
    database: DatabaseReadiness,
    migrations: MigrationReadiness,
}

#[derive(Deserialize)]
struct AppendEventRequest {
    event_id: String,
    event_type: String,
    #[serde(default = "default_schema_version")]
    schema_version: i32,
    occurred_at: DateTime<Utc>,
    source: Value,
    actor: Option<Value>,
    subject: Value,
    #[serde(default = "empty_json_object")]
    payload: Value,
    #[serde(default = "empty_json_object")]
    provenance: Value,
    causation_id: Option<String>,
    correlation_id: Option<String>,
}

impl AppendEventRequest {
    fn into_new_event(self) -> Result<NewEventEnvelope, EventEnvelopeError> {
        let mut builder = NewEventEnvelope::builder(
            self.event_id,
            self.event_type,
            self.occurred_at,
            self.source,
            self.subject,
        )
        .schema_version(self.schema_version)
        .payload(self.payload)
        .provenance(self.provenance);

        if let Some(actor) = self.actor {
            builder = builder.actor(actor);
        }

        if let Some(causation_id) = self.causation_id {
            builder = builder.causation_id(causation_id);
        }

        if let Some(correlation_id) = self.correlation_id {
            builder = builder.correlation_id(correlation_id);
        }

        builder.build()
    }
}

#[derive(Serialize)]
struct AppendEventResponse {
    event_id: String,
    position: i64,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: &'static str,
    message: String,
}

#[derive(Deserialize)]
struct AuditEventsQuery {
    target_id: Option<String>,
    actor_id: Option<String>,
    after_audit_id: Option<i64>,
    limit: Option<u32>,
}

#[derive(Serialize)]
struct AuditEventsResponse {
    items: Vec<ApiAuditRecord>,
}

#[derive(Serialize)]
struct V1StatusResponse {
    version: &'static str,
    surfaces: V1Surfaces,
}

#[derive(Serialize)]
struct V1Surfaces {
    messages: bool,
    contacts: bool,
    search: bool,
    documents: bool,
}

enum ApiError {
    ApiTokenNotConfigured,
    InvalidApiToken,
    InvalidActorId,
    DatabaseNotConfigured,
    InvalidEnvelope(EventEnvelopeError),
    Audit(ApiAuditError),
    Store(EventStoreError),
    NotFound,
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error, message, authenticate) = match self {
            Self::ApiTokenNotConfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                "api_token_not_configured",
                "HERMES_LOCAL_API_TOKEN is not configured".to_owned(),
                false,
            ),
            Self::InvalidApiToken => (
                StatusCode::UNAUTHORIZED,
                "invalid_api_token",
                "missing or invalid bearer token".to_owned(),
                true,
            ),
            Self::InvalidActorId => (
                StatusCode::BAD_REQUEST,
                "invalid_actor_id",
                format!("missing or invalid {LOCAL_API_ACTOR_ID_HEADER} header"),
                false,
            ),
            Self::DatabaseNotConfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                "database_not_configured",
                "DATABASE_URL is not configured".to_owned(),
                false,
            ),
            Self::InvalidEnvelope(error) => (
                StatusCode::BAD_REQUEST,
                "invalid_event_envelope",
                error.to_string(),
                false,
            ),
            Self::Audit(error) => {
                tracing::error!(error = %error, "event API audit operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "api_audit_error",
                    "API audit operation failed".to_owned(),
                    false,
                )
            }
            Self::Store(error) if error.is_unique_violation() => (
                StatusCode::CONFLICT,
                "event_conflict",
                "event already exists or violates idempotency constraints".to_owned(),
                false,
            ),
            Self::Store(error) => {
                tracing::error!(error = %error, "event API store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "event_store_error",
                    "event store operation failed".to_owned(),
                    false,
                )
            }
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                "event_not_found",
                "event was not found".to_owned(),
                false,
            ),
        };

        let mut response = (status, Json(ErrorResponse { error, message })).into_response();
        if authenticate {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        response
    }
}

impl From<EventEnvelopeError> for ApiError {
    fn from(error: EventEnvelopeError) -> Self {
        Self::InvalidEnvelope(error)
    }
}

impl From<EventStoreError> for ApiError {
    fn from(error: EventStoreError) -> Self {
        Self::Store(error)
    }
}

impl From<ApiAuditError> for ApiError {
    fn from(error: ApiAuditError) -> Self {
        Self::Audit(error)
    }
}

fn default_schema_version() -> i32 {
    1
}

fn empty_json_object() -> Value {
    json!({})
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    Io(#[from] io::Error),
}
