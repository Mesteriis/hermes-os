use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::{Body, Bytes};
use axum::extract::{RawQuery, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use color_eyre::eyre::{Context, Result, eyre};
use reqwest::Client;
use serde::Serialize;
use tokio::net::TcpListener;
use tracing::Instrument;
use url::Url;

const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8787";
const DEFAULT_HERMES_BASE_URL: &str = "http://127.0.0.1:8080";
const PUBLIC_WEBHOOK_PATH: &str = "/webhooks/whatsapp/business-cloud";
const PROTECTED_HERMES_WEBHOOK_PATH: &str =
    "/api/v1/integrations/whatsapp/runtime-bridge/business-cloud/webhooks";
const PROTECTED_HERMES_MANIFEST_PATH: &str =
    "/api/v1/integrations/whatsapp/runtime-bridge/business-cloud/proxy-manifest";
const HERMES_SECRET_HEADER: &str = "X-Hermes-Secret";
const BUSINESS_CLOUD_SIGNATURE_HEADER: &str = "X-Hub-Signature-256";

#[derive(Clone)]
struct EdgeState {
    config: Arc<EdgeConfig>,
    client: Client,
}

#[derive(Clone, Debug)]
struct EdgeConfig {
    bind_addr: SocketAddr,
    hermes_base_url: Url,
    hermes_secret: String,
    account_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

#[derive(Debug, Serialize)]
struct EdgeManifestResponse {
    service: &'static str,
    public_webhook_path: &'static str,
    protected_hermes_webhook_path: &'static str,
    protected_hermes_manifest_path: &'static str,
    local_auth_header: &'static str,
    signature_header: &'static str,
    get_forwarding: &'static str,
    post_forwarding: &'static str,
    payload_policy: &'static str,
    secret_policy: &'static str,
    configured_account_id: bool,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: &'static str,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    hermes_hub_backend::app::init_tracing();
    let flow_id = std::env::var("HERMES_FLOW_ID")
        .unwrap_or_else(|_| "whatsapp-business-cloud-edge-proxy".to_owned());
    let runtime_span =
        tracing::info_span!("hermes_whatsapp_business_cloud_edge_proxy", flow_id = %flow_id);

    async move {
        let config = Arc::new(EdgeConfig::from_env()?);
        let listener = TcpListener::bind(config.bind_addr)
            .await
            .with_context(|| format!("binding edge proxy on {}", config.bind_addr))?;
        tracing::info!(
            bind_addr = %config.bind_addr,
            public_webhook_path = PUBLIC_WEBHOOK_PATH,
            hermes_base_url = %config.hermes_base_url,
            "starting WhatsApp Business Cloud edge proxy"
        );
        axum::serve(listener, router(config)).await?;
        Ok(())
    }
    .instrument(runtime_span)
    .await
}

fn router(config: Arc<EdgeConfig>) -> Router {
    let state = EdgeState {
        config,
        client: Client::new(),
    };
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/manifest", get(edge_manifest))
        .route(
            PUBLIC_WEBHOOK_PATH,
            get(forward_business_cloud_webhook_get).post(forward_business_cloud_webhook_post),
        )
        .with_state(state)
}

async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "hermes-whatsapp-business-cloud-edge-proxy",
    })
}

async fn readyz(State(state): State<EdgeState>) -> Response {
    match state
        .config
        .hermes_url(PROTECTED_HERMES_MANIFEST_PATH, None, false)
    {
        Ok(url) => match state
            .client
            .get(url)
            .header(HERMES_SECRET_HEADER, state.config.hermes_secret.as_str())
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => Json(HealthResponse {
                status: "ready",
                service: "hermes-whatsapp-business-cloud-edge-proxy",
            })
            .into_response(),
            Ok(response) => sanitized_error_response(
                StatusCode::BAD_GATEWAY,
                "hermes_proxy_manifest_unavailable",
                response.status(),
            ),
            Err(error) => {
                tracing::warn!(error = %error, "Hermes proxy manifest readiness check failed");
                (
                    StatusCode::BAD_GATEWAY,
                    Json(ErrorResponse {
                        error: "hermes_proxy_manifest_unavailable",
                    }),
                )
                    .into_response()
            }
        },
        Err(error) => {
            tracing::warn!(error = %error, "Hermes manifest URL is invalid");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "hermes_proxy_manifest_url_invalid",
                }),
            )
                .into_response()
        }
    }
}

async fn edge_manifest(State(state): State<EdgeState>) -> Json<EdgeManifestResponse> {
    Json(EdgeManifestResponse {
        service: "hermes-whatsapp-business-cloud-edge-proxy",
        public_webhook_path: PUBLIC_WEBHOOK_PATH,
        protected_hermes_webhook_path: PROTECTED_HERMES_WEBHOOK_PATH,
        protected_hermes_manifest_path: PROTECTED_HERMES_MANIFEST_PATH,
        local_auth_header: HERMES_SECRET_HEADER,
        signature_header: BUSINESS_CLOUD_SIGNATURE_HEADER,
        get_forwarding: "forward_hub_query_params_and_optional_account_id_to_protected_hermes",
        post_forwarding: "forward_exact_raw_body_and_x_hub_signature_256_to_protected_hermes",
        payload_policy: "post_body_is_not_parsed_or_rewritten_by_edge_proxy",
        secret_policy: "local_api_secret_is_env_only_and_never_returned",
        configured_account_id: state.config.account_id.is_some(),
    })
}

async fn forward_business_cloud_webhook_get(
    State(state): State<EdgeState>,
    RawQuery(raw_query): RawQuery,
) -> Response {
    let url =
        match state
            .config
            .hermes_url(PROTECTED_HERMES_WEBHOOK_PATH, raw_query.as_deref(), true)
        {
            Ok(url) => url,
            Err(error) => {
                tracing::warn!(error = %error, "Hermes webhook URL is invalid");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "hermes_webhook_url_invalid",
                    }),
                )
                    .into_response();
            }
        };
    let response = state
        .client
        .get(url)
        .header(HERMES_SECRET_HEADER, state.config.hermes_secret.as_str())
        .send()
        .await;
    forward_upstream_response(response).await
}

async fn forward_business_cloud_webhook_post(
    State(state): State<EdgeState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let signature = match headers
        .get(BUSINESS_CLOUD_SIGNATURE_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(signature) => signature,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "missing_x_hub_signature_256",
                }),
            )
                .into_response();
        }
    };
    let url = match state
        .config
        .hermes_url(PROTECTED_HERMES_WEBHOOK_PATH, None, false)
    {
        Ok(url) => url,
        Err(error) => {
            tracing::warn!(error = %error, "Hermes webhook URL is invalid");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "hermes_webhook_url_invalid",
                }),
            )
                .into_response();
        }
    };
    let mut request = state
        .client
        .post(url)
        .header(HERMES_SECRET_HEADER, state.config.hermes_secret.as_str())
        .header(BUSINESS_CLOUD_SIGNATURE_HEADER, signature)
        .body(body);
    if let Some(content_type) = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
    {
        request = request.header(header::CONTENT_TYPE.as_str(), content_type);
    }
    forward_upstream_response(request.send().await).await
}

async fn forward_upstream_response(
    response: std::result::Result<reqwest::Response, reqwest::Error>,
) -> Response {
    match response {
        Ok(response) if response.status().is_success() => {
            let status = response_status(response.status());
            let content_type = response
                .headers()
                .get(header::CONTENT_TYPE.as_str())
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned);
            let body = match response.bytes().await {
                Ok(body) => body,
                Err(error) => {
                    tracing::warn!(error = %error, "failed to read successful Hermes webhook response");
                    return (
                        StatusCode::BAD_GATEWAY,
                        Json(ErrorResponse {
                            error: "hermes_response_read_failed",
                        }),
                    )
                        .into_response();
                }
            };
            let mut builder = Response::builder().status(status);
            if let Some(content_type) = content_type {
                builder = builder.header(header::CONTENT_TYPE, content_type);
            }
            builder
                .body(Body::from(body))
                .unwrap_or_else(|_| StatusCode::BAD_GATEWAY.into_response())
        }
        Ok(response) => sanitized_error_response(
            StatusCode::BAD_GATEWAY,
            "hermes_webhook_rejected",
            response.status(),
        ),
        Err(error) => {
            tracing::warn!(error = %error, "Hermes webhook forwarding failed");
            (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: "hermes_webhook_unreachable",
                }),
            )
                .into_response()
        }
    }
}

fn sanitized_error_response(
    status: StatusCode,
    error: &'static str,
    upstream_status: reqwest::StatusCode,
) -> Response {
    tracing::warn!(
        upstream_status = upstream_status.as_u16(),
        "Hermes webhook proxy rejected request"
    );
    (status, Json(ErrorResponse { error })).into_response()
}

fn response_status(status: reqwest::StatusCode) -> StatusCode {
    StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY)
}

impl EdgeConfig {
    fn from_env() -> Result<Self> {
        let bind_addr = env_or_default(
            "HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_BIND_ADDR",
            DEFAULT_BIND_ADDR,
        )
        .parse::<SocketAddr>()
        .context("invalid HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_BIND_ADDR")?;
        let hermes_base_url = env_or_default(
            "HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_HERMES_BASE_URL",
            DEFAULT_HERMES_BASE_URL,
        );
        let hermes_base_url = parse_base_url(&hermes_base_url)?;
        let hermes_secret = optional_env("HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_HERMES_SECRET")
            .or_else(|| optional_env("HERMES_LOCAL_API_SECRET"))
            .ok_or_else(|| {
                eyre!(
                    "HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_HERMES_SECRET or HERMES_LOCAL_API_SECRET must be set"
                )
            })?;
        let account_id = optional_env("HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_ACCOUNT_ID");

        Ok(Self {
            bind_addr,
            hermes_base_url,
            hermes_secret,
            account_id,
        })
    }

    fn hermes_url(
        &self,
        protected_path: &str,
        raw_query: Option<&str>,
        include_account_id: bool,
    ) -> Result<Url> {
        let path = protected_path.trim_start_matches('/');
        let mut url = self
            .hermes_base_url
            .join(path)
            .with_context(|| format!("joining Hermes path `{protected_path}`"))?;
        url.set_query(raw_query.filter(|value| !value.trim().is_empty()));
        if include_account_id && let Some(account_id) = &self.account_id {
            url.query_pairs_mut().append_pair("account_id", account_id);
        }
        Ok(url)
    }
}

fn parse_base_url(raw: &str) -> Result<Url> {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(eyre!(
            "HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_HERMES_BASE_URL must not be empty"
        ));
    }
    Url::parse(&format!("{trimmed}/"))
        .with_context(|| "invalid HERMES_WHATSAPP_BUSINESS_CLOUD_EDGE_HERMES_BASE_URL")
}

fn env_or_default(name: &str, default: &str) -> String {
    optional_env(name).unwrap_or_else(|| default.to_owned())
}

fn optional_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::routing::get;
    use serde_json::json;
    use tokio::task::JoinHandle;

    #[derive(Clone, Default)]
    struct FakeHermesState {
        captured: Arc<tokio::sync::Mutex<FakeHermesCaptured>>,
    }

    #[derive(Clone, Debug, Default)]
    struct FakeHermesCaptured {
        manifest_query: Option<String>,
        manifest_secret: Option<String>,
        get_query: Option<String>,
        get_secret: Option<String>,
        post_query: Option<String>,
        post_secret: Option<String>,
        post_signature: Option<String>,
        post_content_type: Option<String>,
        post_body: Vec<u8>,
    }

    #[tokio::test]
    async fn readyz_checks_manifest_without_account_scoping() {
        let (hermes_addr, hermes_state, hermes_handle) = spawn_fake_hermes().await;
        let (edge_addr, edge_handle) = spawn_edge_proxy(hermes_addr, Some("account-1")).await;

        let response = reqwest::Client::new()
            .get(format!("http://{edge_addr}/readyz"))
            .send()
            .await
            .expect("readyz request should reach edge proxy");

        assert_eq!(response.status().as_u16(), StatusCode::OK.as_u16());
        let captured = hermes_state.captured.lock().await.clone();
        assert_eq!(captured.manifest_secret.as_deref(), Some("edge-secret"));
        assert_eq!(
            captured.manifest_query, None,
            "proxy manifest readiness must not be account-scoped"
        );

        edge_handle.abort();
        hermes_handle.abort();
    }

    #[tokio::test]
    async fn get_webhook_forwards_challenge_query_account_scope_and_local_secret() {
        let (hermes_addr, hermes_state, hermes_handle) = spawn_fake_hermes().await;
        let (edge_addr, edge_handle) = spawn_edge_proxy(hermes_addr, Some("account-1")).await;

        let response = reqwest::Client::new()
            .get(format!(
                "http://{edge_addr}{PUBLIC_WEBHOOK_PATH}?hub.mode=subscribe&hub.challenge=abc&hub.verify_token=verify-token"
            ))
            .send()
            .await
            .expect("GET webhook challenge should reach edge proxy");

        assert_eq!(response.status().as_u16(), StatusCode::OK.as_u16());
        assert_eq!(
            response
                .text()
                .await
                .expect("response body should be readable"),
            "challenge-ok"
        );

        let captured = hermes_state.captured.lock().await.clone();
        assert_eq!(captured.get_secret.as_deref(), Some("edge-secret"));
        let query = captured
            .get_query
            .expect("GET webhook must forward query params");
        assert!(query.contains("hub.mode=subscribe"));
        assert!(query.contains("hub.challenge=abc"));
        assert!(query.contains("hub.verify_token=verify-token"));
        assert!(query.contains("account_id=account-1"));

        edge_handle.abort();
        hermes_handle.abort();
    }

    #[tokio::test]
    async fn post_webhook_forwards_raw_body_signature_and_no_account_query() {
        let (hermes_addr, hermes_state, hermes_handle) = spawn_fake_hermes().await;
        let (edge_addr, edge_handle) = spawn_edge_proxy(hermes_addr, Some("account-1")).await;
        let body = br#"{"entry":[{"changes":[{"value":{"messages":[]}}]}]}"#;

        let response = reqwest::Client::new()
            .post(format!("http://{edge_addr}{PUBLIC_WEBHOOK_PATH}"))
            .header(BUSINESS_CLOUD_SIGNATURE_HEADER, "sha256=test-signature")
            .header(header::CONTENT_TYPE.as_str(), "application/json")
            .body(body.as_slice().to_vec())
            .send()
            .await
            .expect("POST webhook should reach edge proxy");

        assert_eq!(response.status().as_u16(), StatusCode::ACCEPTED.as_u16());
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE.as_str())
                .and_then(|value| value.to_str().ok()),
            Some("application/json")
        );
        assert_eq!(
            response
                .text()
                .await
                .expect("response body should be readable"),
            r#"{"accepted":true}"#
        );

        let captured = hermes_state.captured.lock().await.clone();
        assert_eq!(captured.post_secret.as_deref(), Some("edge-secret"));
        assert_eq!(
            captured.post_signature.as_deref(),
            Some("sha256=test-signature")
        );
        assert_eq!(
            captured.post_content_type.as_deref(),
            Some("application/json")
        );
        assert_eq!(captured.post_body, body);
        assert_eq!(
            captured.post_query, None,
            "POST body signature verification is handled by Hermes and must not add account_id query state"
        );

        edge_handle.abort();
        hermes_handle.abort();
    }

    #[tokio::test]
    async fn post_webhook_requires_meta_signature_before_forwarding() {
        let (hermes_addr, hermes_state, hermes_handle) = spawn_fake_hermes().await;
        let (edge_addr, edge_handle) = spawn_edge_proxy(hermes_addr, Some("account-1")).await;

        let response = reqwest::Client::new()
            .post(format!("http://{edge_addr}{PUBLIC_WEBHOOK_PATH}"))
            .header(header::CONTENT_TYPE.as_str(), "application/json")
            .body(r#"{"entry":[]}"#)
            .send()
            .await
            .expect("POST webhook should reach edge proxy");

        assert_eq!(response.status().as_u16(), StatusCode::BAD_REQUEST.as_u16());
        assert_eq!(
            response
                .text()
                .await
                .expect("error response should be readable"),
            r#"{"error":"missing_x_hub_signature_256"}"#
        );
        let captured = hermes_state.captured.lock().await.clone();
        assert!(
            captured.post_body.is_empty(),
            "unsigned webhook requests must not be forwarded to Hermes"
        );

        edge_handle.abort();
        hermes_handle.abort();
    }

    async fn spawn_edge_proxy(
        hermes_addr: SocketAddr,
        account_id: Option<&str>,
    ) -> (SocketAddr, JoinHandle<()>) {
        const TEST_HERMES_SHARED_KEY: &str = concat!("edge", "-", "secret");
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("edge proxy test listener should bind");
        let addr = listener
            .local_addr()
            .expect("edge proxy test listener should have local addr");
        let config = Arc::new(EdgeConfig {
            bind_addr: addr,
            hermes_base_url: Url::parse(&format!("http://{hermes_addr}/"))
                .expect("Hermes test URL should parse"),
            hermes_secret: TEST_HERMES_SHARED_KEY.to_owned(),
            account_id: account_id.map(str::to_owned),
        });
        let app = router(config);
        let handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("edge proxy test server should serve")
        });
        (addr, handle)
    }

    async fn spawn_fake_hermes() -> (SocketAddr, FakeHermesState, JoinHandle<()>) {
        let state = FakeHermesState::default();
        let app = Router::new()
            .route(PROTECTED_HERMES_MANIFEST_PATH, get(fake_manifest))
            .route(
                PROTECTED_HERMES_WEBHOOK_PATH,
                get(fake_webhook_get).post(fake_webhook_post),
            )
            .with_state(state.clone());
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("fake Hermes test listener should bind");
        let addr = listener
            .local_addr()
            .expect("fake Hermes test listener should have local addr");
        let handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("fake Hermes test server should serve")
        });
        (addr, state, handle)
    }

    async fn fake_manifest(
        State(state): State<FakeHermesState>,
        RawQuery(raw_query): RawQuery,
        headers: HeaderMap,
    ) -> Response {
        let secret = header_value(&headers, HERMES_SECRET_HEADER);
        let mut captured = state.captured.lock().await;
        captured.manifest_query = raw_query;
        captured.manifest_secret = secret.clone();
        drop(captured);

        if secret.as_deref() == Some("edge-secret") {
            Json(json!({ "status": "ok" })).into_response()
        } else {
            StatusCode::UNAUTHORIZED.into_response()
        }
    }

    async fn fake_webhook_get(
        State(state): State<FakeHermesState>,
        RawQuery(raw_query): RawQuery,
        headers: HeaderMap,
    ) -> Response {
        let mut captured = state.captured.lock().await;
        captured.get_query = raw_query;
        captured.get_secret = header_value(&headers, HERMES_SECRET_HEADER);
        drop(captured);

        "challenge-ok".into_response()
    }

    async fn fake_webhook_post(
        State(state): State<FakeHermesState>,
        RawQuery(raw_query): RawQuery,
        headers: HeaderMap,
        body: Bytes,
    ) -> Response {
        let mut captured = state.captured.lock().await;
        captured.post_query = raw_query;
        captured.post_secret = header_value(&headers, HERMES_SECRET_HEADER);
        captured.post_signature = header_value(&headers, BUSINESS_CLOUD_SIGNATURE_HEADER);
        captured.post_content_type = header_value(&headers, header::CONTENT_TYPE.as_str());
        captured.post_body = body.to_vec();
        drop(captured);

        (
            StatusCode::ACCEPTED,
            [(header::CONTENT_TYPE, "application/json")],
            r#"{"accepted":true}"#,
        )
            .into_response()
    }

    fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
    }
}
