// ADR-0073: app router owns HTTP composition; route groups live in
// focused modules so endpoint registration remains auditable without a god file.
use std::io;

use axum::extract::State;
use axum::http::{HeaderName, Method, StatusCode, header};
use axum::{Json, Router};
use serde::Serialize;
use tokio::net::TcpListener;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber::EnvFilter;

use crate::app::vault_reconciliation::spawn_host_vault_manifest_reconciliation;
use crate::app::{AccountSetupState, AppError, AppState};
use crate::integrations::telegram::runtime::TelegramRuntimeManager;
use crate::platform::config::AppConfig;
use crate::platform::events::EventBus;
use crate::platform::storage::{Database, DatabaseReadiness, MigrationReadiness, ReadinessStatus};
use crate::vault::{HostVault, HostVaultConfig};

mod routes;

pub fn build_router(config: AppConfig) -> Router {
    build_router_with_database(config, Database::disabled())
}

pub fn build_router_with_database(config: AppConfig, database: Database) -> Router {
    let api_secret = config.local_api_secret().unwrap_or_default().to_owned();
    let vault = HostVault::new(HostVaultConfig {
        home: config.vault_home().to_path_buf(),
        dev_mode: config.dev_mode(),
        dev_key_path: config.dev_key_path().to_path_buf(),
    })
    .expect("host vault runtime must initialize");
    if let Err(error) = vault.unlock_existing() {
        tracing::warn!(error = %error, "host vault auto-unlock skipped");
    }
    let state = AppState {
        config,
        database,
        vault,
        account_setup: AccountSetupState::default(),
        telegram_runtime: TelegramRuntimeManager::default(),
        event_bus: EventBus::new(),
    };
    spawn_host_vault_manifest_reconciliation(&state);
    crate::application::bootstrap::start_background_services(
        crate::application::bootstrap::ApplicationBootstrapContext {
            pool: state.database.pool().cloned(),
            database_url: state.database.database_url().map(ToOwned::to_owned),
            vault: state.vault.clone(),
            telegram_runtime: state.telegram_runtime.clone(),
            event_bus: state.event_bus.clone(),
        },
    );

    Router::<AppState>::new()
        .merge(routes::public_routes())
        .merge(routes::protected_routes(api_secret))
        .with_state(state)
        .layer(local_frontend_cors_layer())
}

#[derive(Serialize)]
pub(crate) struct HealthResponse {
    status: &'static str,
    service: String,
}

#[derive(Serialize)]
pub(crate) struct ReadinessResponse {
    status: &'static str,
    service: String,
    checks: ReadinessChecks,
}

#[derive(Serialize)]
pub(crate) struct ReadinessChecks {
    database: DatabaseReadiness,
    migrations: MigrationReadiness,
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
    let log_format = std::env::var("HERMES_LOG_FORMAT").unwrap_or_else(|_| "plain".to_owned());

    if log_format.eq_ignore_ascii_case("json") {
        let _ = tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .with_current_span(true)
            .with_span_list(false)
            .flatten_event(true)
            .try_init();
        return;
    }

    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

pub(crate) fn local_frontend_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _| {
            origin
                .to_str()
                .map(is_allowed_local_frontend_origin)
                .unwrap_or(false)
        }))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            HeaderName::from_static("x-hermes-secret"),
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

pub(crate) async fn healthz(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: state.config.service_name().to_owned(),
    })
}

pub(crate) async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<ReadinessResponse>) {
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
