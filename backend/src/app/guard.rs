use axum::Json;
use axum::extract::{Request, State};
use axum::http::{StatusCode, Uri};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use url::form_urlencoded;

#[derive(Serialize)]
struct SecretErrorResponse {
    error: &'static str,
    message: &'static str,
}

pub async fn require_secret(
    State(expected_secret): State<String>,
    req: Request,
    next: Next,
) -> Result<Response, Response> {
    if expected_secret.is_empty() {
        return Err(secret_error_response());
    }

    let ok = has_valid_secret(req.headers(), req.uri(), expected_secret.as_str());

    if ok {
        Ok(next.run(req).await)
    } else {
        Err(secret_error_response())
    }
}

fn has_valid_secret(headers: &axum::http::HeaderMap, uri: &Uri, expected_secret: &str) -> bool {
    has_valid_secret_header(headers, expected_secret) || has_valid_websocket_secret_query(uri, expected_secret)
}

fn has_valid_secret_header(headers: &axum::http::HeaderMap, expected_secret: &str) -> bool {
    headers
        .get("x-hermes-secret")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|secret| secret == expected_secret)
}

fn has_valid_websocket_secret_query(uri: &Uri, expected_secret: &str) -> bool {
    if uri.path() != "/api/events/ws" {
        return false;
    }

    let Some(query) = uri.query() else {
        return false;
    };

    form_urlencoded::parse(query.as_bytes())
        .any(|(name, value)| name == "hermes_secret" && value == expected_secret)
}

fn secret_error_response() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(SecretErrorResponse {
            error: "invalid_api_secret",
            message: "missing or invalid x-hermes-secret header",
        }),
    )
        .into_response()
}
