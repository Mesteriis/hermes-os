pub mod audit;
pub mod communications;
pub mod config;
pub mod contacts;
pub mod documents;
pub mod email_account_setup;
pub mod email_fixture_export;
pub mod email_fixture_pipeline;
pub mod email_import;
pub mod email_provider_network;
pub mod email_sources;
pub mod email_sync;
pub mod event_log;
pub mod graph;
pub mod graph_projection;
pub mod mail_storage;
pub mod messages;
pub mod projections;
pub mod search;
pub mod secret_vault;
pub mod secrets;
pub mod storage;

use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};

use axum::extract::{Path, Query, RawQuery, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header};
use axum::response::Html;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber::EnvFilter;
use url::form_urlencoded;

use crate::audit::{ApiAuditError, ApiAuditLog, ApiAuditRecord, NewApiAuditRecord};
use crate::communications::{CommunicationIngestionStore, EmailProviderKind};
use crate::config::AppConfig;
use crate::email_account_setup::{
    EmailAccountSetupError, EmailAccountSetupService, GmailOAuthPendingGrant,
    GmailOAuthSetupRequest, ImapAccountSetupRequest,
};
use crate::event_log::{
    EventEnvelope, EventEnvelopeError, EventStore, EventStoreError, NewEventEnvelope,
};
use crate::secret_vault::EncryptedSecretVault;
use crate::secrets::{SecretKind, SecretReferenceStore};
use crate::storage::{
    Database, DatabaseReadiness, MigrationReadiness, ReadinessStatus, StorageError,
};

const LOCAL_API_ACTOR_ID_HEADER: &str = "x-hermes-actor-id";
const MAX_LOCAL_API_ACTOR_ID_LENGTH: usize = 128;

#[derive(Clone)]
struct AppState {
    config: AppConfig,
    database: Database,
    account_setup: AccountSetupState,
}

#[derive(Clone, Default)]
struct AccountSetupState {
    pending_gmail_oauth: Arc<Mutex<HashMap<String, GmailOAuthPendingGrant>>>,
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
        .route("/api/v2/graph/summary", get(get_graph_summary))
        .route("/api/v2/graph/neighborhood", get(get_graph_neighborhood))
        .route("/api/v2/graph/search", get(get_graph_search))
        .route(
            "/api/v1/email-accounts/gmail/oauth/start",
            post(post_gmail_oauth_start),
        )
        .route(
            "/api/v1/email-accounts/gmail/oauth/complete",
            post(post_gmail_oauth_complete),
        )
        .route(
            "/api/v1/email-accounts/gmail/oauth/callback",
            get(get_gmail_oauth_callback),
        )
        .route("/api/v1/email-accounts/imap", post(post_imap_account_setup))
        .route("/api/audit/events", get(get_audit_events))
        .route("/api/events", post(post_event))
        .route("/api/events/{event_id}", get(get_event))
        .with_state(AppState {
            config,
            database,
            account_setup: AccountSetupState::default(),
        })
        .layer(local_frontend_cors_layer())
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

fn local_frontend_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _| {
            origin
                .to_str()
                .map(is_allowed_local_frontend_origin)
                .unwrap_or(false)
        }))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            HeaderName::from_static(LOCAL_API_ACTOR_ID_HEADER),
        ])
}

fn is_allowed_local_frontend_origin(origin: &str) -> bool {
    let Ok(url) = url::Url::parse(origin) else {
        return false;
    };

    matches!(
        (url.scheme(), url.host_str()),
        (
            "http" | "https",
            Some("localhost" | "127.0.0.1" | "::1" | "[::1]")
        ) | ("http" | "https", Some("tauri.localhost"))
            | ("tauri", Some("localhost"))
    )
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
            account_setup: true,
        },
    }))
}

async fn get_graph_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<crate::graph::GraphSummary>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    Ok(Json(graph_store(&state)?.summary().await?))
}

async fn get_graph_neighborhood(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<crate::graph::GraphNeighborhood>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let query = parse_graph_neighborhood_query(raw_query.as_deref())?;
    if query.depth.unwrap_or(1) != 1 {
        return Err(ApiError::InvalidGraphQuery("depth supports only 1"));
    }
    let Some(node_id) = query
        .node_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    else {
        return Err(ApiError::GraphNotFound);
    };
    let Some(neighborhood) = graph_store(&state)?.neighborhood(node_id).await? else {
        return Err(ApiError::GraphNotFound);
    };
    Ok(Json(neighborhood))
}

async fn get_graph_search(
    State(state): State<AppState>,
    headers: HeaderMap,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<Vec<crate::graph::GraphNode>>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let query = parse_graph_search_query(raw_query.as_deref())?;
    let search = query.q.as_deref().unwrap_or_default().trim();
    if search.is_empty() {
        return Err(ApiError::InvalidGraphQuery("q must not be empty"));
    }
    let limit = query.limit.unwrap_or(20).clamp(1, 50);
    Ok(Json(
        graph_store(&state)?.search_nodes(search, limit).await?,
    ))
}

async fn post_gmail_oauth_start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<GmailOAuthStartApiRequest>,
) -> Result<Json<GmailOAuthStartApiResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let service = EmailAccountSetupService::new_for_vault_only(
        encrypted_vault(&state.config).ok_or(ApiError::SecretVaultNotConfigured)?,
    );
    let pending = service.start_gmail_oauth(request.into_setup_request())?;
    let response = GmailOAuthStartApiResponse {
        setup_id: pending.setup_id.clone(),
        authorization_url: pending.authorization_url.clone(),
        state: pending.state.clone(),
        redirect_uri: pending.request.redirect_uri.clone(),
    };
    let mut pending_map = state
        .account_setup
        .pending_gmail_oauth
        .lock()
        .map_err(|_| ApiError::AccountSetupState)?;
    pending_map.insert(pending.setup_id.clone(), pending);

    Ok(Json(response))
}

async fn post_gmail_oauth_complete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<GmailOAuthCompleteApiRequest>,
) -> Result<Json<EmailAccountSetupApiResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let pending = {
        let mut pending_map = state
            .account_setup
            .pending_gmail_oauth
            .lock()
            .map_err(|_| ApiError::AccountSetupState)?;
        pending_map
            .remove(&request.setup_id)
            .ok_or(ApiError::AccountSetupPendingGrantNotFound)?
    };
    if pending.state != request.state {
        return Err(ApiError::AccountSetupStateMismatch);
    }

    let service = account_setup_service(&state)?;
    let result = service
        .complete_gmail_oauth(pending, &request.authorization_code)
        .await?;

    Ok(Json(result.into()))
}

async fn get_gmail_oauth_callback(Query(query): Query<GmailOAuthCallbackQuery>) -> Html<String> {
    let code = html_escape(&query.code.unwrap_or_default());
    let state = html_escape(&query.state.unwrap_or_default());

    Html(format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Hermes Hub OAuth</title>
  <style>
    body {{ margin: 0; font-family: system-ui, sans-serif; color: #182033; background: #f5f6f8; }}
    main {{ max-width: 720px; margin: 48px auto; background: #fff; border: 1px solid #d9dee7; border-radius: 8px; padding: 24px; }}
    code {{ display: block; overflow-wrap: anywhere; background: #f8fafc; border: 1px solid #d9dee7; border-radius: 6px; padding: 10px; }}
  </style>
</head>
<body>
  <main>
    <h1>Gmail OAuth callback</h1>
    <p>Authorization code</p>
    <code>{code}</code>
    <p>State</p>
    <code>{state}</code>
  </main>
</body>
</html>"#
    ))
}

async fn post_imap_account_setup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ImapAccountSetupApiRequest>,
) -> Result<Json<EmailAccountSetupApiResponse>, ApiError> {
    verify_local_api_capability(&state.config, &headers)?;
    let service = account_setup_service(&state)?;
    let result = service
        .setup_imap_account(request.into_setup_request()?)
        .await?;

    Ok(Json(result.into()))
}

fn event_store(state: &AppState) -> Result<EventStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(EventStore::new(pool.clone()))
}

fn graph_store(state: &AppState) -> Result<crate::graph::GraphStore, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(crate::graph::GraphStore::new(pool.clone()))
}

fn api_audit_log(state: &AppState) -> Result<ApiAuditLog, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(ApiAuditLog::new(pool.clone()))
}

fn account_setup_service(state: &AppState) -> Result<EmailAccountSetupService, ApiError> {
    let vault = encrypted_vault(&state.config).ok_or(ApiError::SecretVaultNotConfigured)?;
    let Some(pool) = state.database.pool() else {
        return Err(ApiError::DatabaseNotConfigured);
    };

    Ok(EmailAccountSetupService::new(
        CommunicationIngestionStore::new(pool.clone()),
        SecretReferenceStore::new(pool.clone()),
        vault,
    ))
}

fn encrypted_vault(config: &AppConfig) -> Option<EncryptedSecretVault> {
    Some(EncryptedSecretVault::new(
        config.secret_vault_path()?.to_path_buf(),
        config.secret_vault_key()?.clone(),
    ))
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
    account_setup: bool,
}

struct GraphNeighborhoodQuery {
    node_id: Option<String>,
    depth: Option<u8>,
}

struct GraphSearchQuery {
    q: Option<String>,
    limit: Option<i64>,
}

fn parse_graph_neighborhood_query(
    raw_query: Option<&str>,
) -> Result<GraphNeighborhoodQuery, ApiError> {
    let mut query = GraphNeighborhoodQuery {
        node_id: None,
        depth: None,
    };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            match key.as_ref() {
                "node_id" => query.node_id = Some(value.into_owned()),
                "depth" => {
                    query.depth = Some(
                        value
                            .parse::<u8>()
                            .map_err(|_| ApiError::InvalidGraphQuery("depth supports only 1"))?,
                    );
                }
                _ => {}
            }
        }
    }

    Ok(query)
}

fn parse_graph_search_query(raw_query: Option<&str>) -> Result<GraphSearchQuery, ApiError> {
    let mut query = GraphSearchQuery {
        q: None,
        limit: None,
    };

    if let Some(raw_query) = raw_query {
        for (key, value) in form_urlencoded::parse(raw_query.as_bytes()) {
            match key.as_ref() {
                "q" => query.q = Some(value.into_owned()),
                "limit" => {
                    query.limit =
                        Some(value.parse::<i64>().map_err(|_| {
                            ApiError::InvalidGraphQuery("limit must be an integer")
                        })?);
                }
                _ => {}
            }
        }
    }

    Ok(query)
}

enum ApiError {
    ApiTokenNotConfigured,
    InvalidApiToken,
    InvalidActorId,
    DatabaseNotConfigured,
    InvalidEnvelope(EventEnvelopeError),
    Audit(ApiAuditError),
    Store(EventStoreError),
    Graph(crate::graph::GraphStoreError),
    InvalidGraphQuery(&'static str),
    SecretVaultNotConfigured,
    AccountSetup(EmailAccountSetupError),
    AccountSetupState,
    AccountSetupPendingGrantNotFound,
    AccountSetupStateMismatch,
    GraphNotFound,
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
            Self::SecretVaultNotConfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                "secret_vault_not_configured",
                "HERMES_SECRET_VAULT_PATH and HERMES_SECRET_VAULT_KEY are required for account setup".to_owned(),
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
            Self::Graph(error) => {
                tracing::error!(error = %error, "graph store operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "graph_store_error",
                    "graph store operation failed".to_owned(),
                    false,
                )
            }
            Self::InvalidGraphQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_graph_query",
                message.to_owned(),
                false,
            ),
            Self::AccountSetup(error) => {
                let status = if matches!(
                    error,
                    EmailAccountSetupError::InvalidRequest { .. }
                        | EmailAccountSetupError::MissingProviderField { .. }
                ) {
                    StatusCode::BAD_REQUEST
                } else {
                    tracing::error!(error = %error, "account setup failed");
                    StatusCode::INTERNAL_SERVER_ERROR
                };
                (
                    status,
                    "account_setup_error",
                    if status == StatusCode::BAD_REQUEST {
                        error.to_string()
                    } else {
                        "account setup failed".to_owned()
                    },
                    false,
                )
            }
            Self::AccountSetupState => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "account_setup_state_error",
                "account setup state is unavailable".to_owned(),
                false,
            ),
            Self::AccountSetupPendingGrantNotFound => (
                StatusCode::NOT_FOUND,
                "account_setup_pending_grant_not_found",
                "pending Gmail OAuth setup was not found".to_owned(),
                false,
            ),
            Self::AccountSetupStateMismatch => (
                StatusCode::BAD_REQUEST,
                "account_setup_state_mismatch",
                "Gmail OAuth state does not match the pending setup".to_owned(),
                false,
            ),
            Self::GraphNotFound => (
                StatusCode::NOT_FOUND,
                "graph_node_not_found",
                "graph node was not found".to_owned(),
                false,
            ),
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

impl From<crate::graph::GraphStoreError> for ApiError {
    fn from(error: crate::graph::GraphStoreError) -> Self {
        Self::Graph(error)
    }
}

impl From<ApiAuditError> for ApiError {
    fn from(error: ApiAuditError) -> Self {
        Self::Audit(error)
    }
}

impl From<EmailAccountSetupError> for ApiError {
    fn from(error: EmailAccountSetupError) -> Self {
        Self::AccountSetup(error)
    }
}

#[derive(Deserialize)]
struct GmailOAuthStartApiRequest {
    account_id: String,
    display_name: String,
    external_account_id: String,
    client_id: String,
    client_secret: Option<String>,
    redirect_uri: String,
    authorization_endpoint: Option<String>,
    token_endpoint: Option<String>,
}

impl GmailOAuthStartApiRequest {
    fn into_setup_request(self) -> GmailOAuthSetupRequest {
        let mut request = GmailOAuthSetupRequest::new(
            self.account_id,
            self.display_name,
            self.external_account_id,
            self.client_id,
            self.redirect_uri,
        );
        if let Some(client_secret) = self.client_secret {
            request = request.client_secret(client_secret);
        }
        if let Some(authorization_endpoint) = self.authorization_endpoint {
            request = request.authorization_endpoint(authorization_endpoint);
        }
        if let Some(token_endpoint) = self.token_endpoint {
            request = request.token_endpoint(token_endpoint);
        }

        request
    }
}

#[derive(Serialize)]
struct GmailOAuthStartApiResponse {
    setup_id: String,
    authorization_url: String,
    state: String,
    redirect_uri: String,
}

#[derive(Deserialize)]
struct GmailOAuthCompleteApiRequest {
    setup_id: String,
    state: String,
    authorization_code: String,
}

#[derive(Deserialize)]
struct GmailOAuthCallbackQuery {
    code: Option<String>,
    state: Option<String>,
}

#[derive(Deserialize)]
struct ImapAccountSetupApiRequest {
    account_id: String,
    provider_kind: String,
    display_name: String,
    external_account_id: String,
    host: String,
    port: u16,
    tls: bool,
    mailbox: String,
    username: String,
    password: String,
    secret_kind: Option<String>,
}

impl ImapAccountSetupApiRequest {
    fn into_setup_request(self) -> Result<ImapAccountSetupRequest, ApiError> {
        let provider_kind = match self.provider_kind.trim() {
            "icloud" => EmailProviderKind::Icloud,
            "imap" => EmailProviderKind::Imap,
            _ => {
                return Err(EmailAccountSetupError::InvalidRequest {
                    field: "provider_kind",
                    message: "must be icloud or imap",
                }
                .into());
            }
        };
        let secret_kind = match self.secret_kind.as_deref().unwrap_or("password").trim() {
            "app_password" => SecretKind::AppPassword,
            "password" => SecretKind::Password,
            _ => {
                return Err(EmailAccountSetupError::InvalidRequest {
                    field: "secret_kind",
                    message: "must be app_password or password",
                }
                .into());
            }
        };

        Ok(ImapAccountSetupRequest::new(
            self.account_id,
            provider_kind,
            self.display_name,
            self.external_account_id,
            self.host,
            self.port,
            self.tls,
            self.mailbox,
            self.username,
            self.password,
        )
        .secret_kind(secret_kind))
    }
}

#[derive(Serialize)]
struct EmailAccountSetupApiResponse {
    account_id: String,
    secret_ref: String,
    secret_kind: SecretKind,
    store_kind: crate::secrets::SecretStoreKind,
}

impl From<crate::email_account_setup::EmailAccountSetupResult> for EmailAccountSetupApiResponse {
    fn from(result: crate::email_account_setup::EmailAccountSetupResult) -> Self {
        Self {
            account_id: result.account_id,
            secret_ref: result.secret_ref,
            secret_kind: result.secret_kind,
            store_kind: result.store_kind,
        }
    }
}

fn default_schema_version() -> i32 {
    1
}

fn empty_json_object() -> Value {
    json!({})
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    Io(#[from] io::Error),
}
