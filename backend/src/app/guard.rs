use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;

pub async fn require_secret(req: Request, next: Next) -> Result<Response, StatusCode> {
    let ok = req
        .headers()
        .get("x-hermes-secret")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| !v.is_empty());

    if ok {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}
