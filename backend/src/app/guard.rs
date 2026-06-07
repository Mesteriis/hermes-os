use axum::Json;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

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

    let ok = req
        .headers()
        .get("x-hermes-secret")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v == expected_secret);

    if ok {
        Ok(next.run(req).await)
    } else {
        Err(secret_error_response())
    }
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
