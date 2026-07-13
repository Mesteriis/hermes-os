use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header};
use axum::response::Response;
use serde::Deserialize;

use super::errors::remote_image_fetch_api_error;
use super::fetcher::fetch_remote_image;
use super::reference::message_html_references_url;
use super::url_policy::parse_remote_image_url;
use crate::app::api_support::stores::domain_stores::message_store;
use crate::app::handlers::communications::communication_messages::rich_body_html_for_message;
use crate::app::{ApiError, AppState};

#[derive(Deserialize)]
pub(crate) struct RemoteImageQuery {
    url: String,
}

pub(crate) async fn get_v1_communication_message_remote_image(
    State(state): State<AppState>,
    Path(message_id): Path<String>,
    Query(query): Query<RemoteImageQuery>,
) -> Result<Response, ApiError> {
    let image_url = parse_remote_image_url(&query.url)?;
    let Some(message) = message_store(&state)?.message(&message_id).await? else {
        return Err(ApiError::CommunicationMessageNotFound);
    };
    let Some(body_html) = rich_body_html_for_message(&state, &message).await? else {
        return Err(ApiError::CommunicationMessageNotFound);
    };
    if !message_html_references_url(&body_html, image_url.as_str()) {
        return Err(ApiError::InvalidCommunicationQuery(
            "remote image is not referenced by this message",
        ));
    }

    let image = fetch_remote_image(&image_url)
        .await
        .map_err(remote_image_fetch_api_error)?;

    let mut response = Response::new(Body::from(image.body));
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, image.content_type);
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=600"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    Ok(response)
}
