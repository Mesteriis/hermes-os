//! Authenticated, replayable SSE transport for client-safe Gateway frames.

use std::convert::Infallible;
use std::sync::Arc;

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use bytes::Bytes;
use futures_util::{StreamExt, stream};
use hermes_gateway_protocol::{
    v1::{ClientRealtimeFrameV1, ClientReplayGapV1, client_realtime_frame_v1::Frame},
    validation::validate_client_realtime_frame,
};
use hermes_gateway_session::BrowserSession;
use hermes_gateway_session_contract::BrowserAuthenticationAuthority;
use http_body_util::{BodyExt, StreamBody};
use hyper::body::Frame as HttpFrame;
use hyper::header::{CACHE_CONTROL, CONTENT_TYPE, COOKIE, HeaderName};
use hyper::{Method, Request, Response, StatusCode};
use prost::Message;
use tokio::sync::broadcast;

use crate::{GatewayHttpResponse, SharedBrowserGatewaySessionService, full_gateway_body};

const REALTIME_PATH: &str = "/api/realtime/v1/events";
const MAX_CURSOR_BYTES: usize = 512;
const LAST_EVENT_ID: HeaderName = HeaderName::from_static("last-event-id");

/// Implementations must authorize and filter frames for the exact session.
pub trait BrowserRealtimeSubscriptionSource: Send + Sync {
    fn subscribe(
        &self,
        session: &BrowserSession,
        after_cursor: Option<&str>,
    ) -> Result<ClientRealtimeSubscriptionV1, String>;
}

/// A source must subscribe before taking its replay snapshot, preventing a
/// silent gap between reconnect replay and live delivery.
pub struct ClientRealtimeSubscriptionV1 {
    replay: Vec<ClientRealtimeFrameV1>,
    live: broadcast::Receiver<ClientRealtimeFrameV1>,
}

impl ClientRealtimeSubscriptionV1 {
    pub fn new(
        replay: Vec<ClientRealtimeFrameV1>,
        live: broadcast::Receiver<ClientRealtimeFrameV1>,
    ) -> Result<Self, String> {
        replay.iter().try_for_each(validate_client_realtime_frame)?;
        Ok(Self { replay, live })
    }

    fn into_parts(
        self,
    ) -> (
        Vec<ClientRealtimeFrameV1>,
        broadcast::Receiver<ClientRealtimeFrameV1>,
    ) {
        (self.replay, self.live)
    }
}

/// Detached SSE router. It exposes no owner query or command endpoint.
pub struct BrowserRealtimeRouter<A, S> {
    service: SharedBrowserGatewaySessionService<A>,
    source: Arc<S>,
}

impl<A, S> Clone for BrowserRealtimeRouter<A, S> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            source: self.source.clone(),
        }
    }
}

impl<A, S> BrowserRealtimeRouter<A, S>
where
    A: BrowserAuthenticationAuthority,
    S: BrowserRealtimeSubscriptionSource,
{
    #[must_use]
    pub fn new(service: SharedBrowserGatewaySessionService<A>, source: S) -> Self {
        Self {
            service,
            source: Arc::new(source),
        }
    }

    #[must_use]
    pub fn route<B>(&self, request: Request<B>) -> GatewayHttpResponse {
        let (parts, _) = request.into_parts();
        if !is_realtime_request(&parts.method, &parts.uri) {
            return response(StatusCode::NOT_FOUND, "not found\n");
        }
        let cookie = parts.headers.get(COOKIE).and_then(header_text);
        let after_cursor = match replay_cursor(&parts.headers) {
            Ok(cursor) => cursor,
            Err(()) => return response(StatusCode::BAD_REQUEST, "replay cursor is invalid\n"),
        };
        let session = match self.service.authorize_request(cookie) {
            Ok(session) => session,
            Err(_) => {
                return response(StatusCode::UNAUTHORIZED, "browser session is unavailable\n");
            }
        };
        match self.source.subscribe(&session, after_cursor.as_deref()) {
            Ok(subscription) => stream_response(subscription, after_cursor.as_deref()),
            Err(_) => response(StatusCode::SERVICE_UNAVAILABLE, "realtime is unavailable\n"),
        }
    }
}

fn is_realtime_request(method: &Method, uri: &hyper::Uri) -> bool {
    *method == Method::GET && uri.path() == REALTIME_PATH && uri.query().is_none()
}

fn header_text(value: &hyper::header::HeaderValue) -> Option<&str> {
    value.to_str().ok()
}

fn replay_cursor(headers: &hyper::HeaderMap) -> Result<Option<String>, ()> {
    match headers.get(LAST_EVENT_ID).map(header_text) {
        None => Ok(None),
        Some(Some(cursor)) if valid_cursor(cursor) => Ok(Some(cursor.to_owned())),
        Some(None) | Some(Some(_)) => Err(()),
    }
}

fn stream_response(
    subscription: ClientRealtimeSubscriptionV1,
    requested_cursor: Option<&str>,
) -> GatewayHttpResponse {
    let (replay, receiver) = subscription.into_parts();
    let replay = stream::iter(replay.into_iter().map(Ok::<_, Infallible>));
    let requested_cursor = requested_cursor.unwrap_or_default().to_owned();
    let live = live_frames(receiver, requested_cursor);
    let body = BodyExt::boxed(StreamBody::new(
        replay
            .chain(live)
            .map(|frame| frame.map(|frame| HttpFrame::data(encode_sse(&frame)))),
    ));
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/event-stream")
        .header(CACHE_CONTROL, "no-store")
        .header("x-accel-buffering", "no")
        .body(body)
        .expect("Gateway SSE response is valid")
}

fn live_frames(
    receiver: broadcast::Receiver<ClientRealtimeFrameV1>,
    requested_cursor: String,
) -> impl futures_util::Stream<Item = Result<ClientRealtimeFrameV1, Infallible>> {
    stream::unfold(
        (receiver, requested_cursor, false),
        |(mut receiver, cursor, ended)| async move {
            if ended {
                return None;
            }
            match receiver.recv().await {
                Ok(frame) if validate_client_realtime_frame(&frame).is_ok() => {
                    Some((Ok(frame), (receiver, cursor, false)))
                }
                Ok(_) => Some((
                    Ok(replay_gap(&cursor, "invalid_live_frame")),
                    (receiver, cursor, true),
                )),
                Err(broadcast::error::RecvError::Lagged(_)) => Some((
                    Ok(replay_gap(&cursor, "live_buffer_overrun")),
                    (receiver, cursor, true),
                )),
                Err(broadcast::error::RecvError::Closed) => None,
            }
        },
    )
}

fn replay_gap(requested_cursor: &str, reason_code: &str) -> ClientRealtimeFrameV1 {
    ClientRealtimeFrameV1 {
        frame: Some(Frame::ReplayGap(ClientReplayGapV1 {
            requested_cursor: requested_cursor.to_owned(),
            earliest_available_cursor: String::new(),
            reason_code: reason_code.to_owned(),
        })),
    }
}

fn encode_sse(frame: &ClientRealtimeFrameV1) -> Bytes {
    let mut payload = String::new();
    if let Some(cursor) = frame_cursor(frame) {
        payload.push_str("id: ");
        payload.push_str(cursor);
        payload.push('\n');
    }
    payload.push_str("event: hermes.realtime.v1\n");
    payload.push_str("data: ");
    payload.push_str(&URL_SAFE_NO_PAD.encode(frame.encode_to_vec()));
    payload.push_str("\n\n");
    Bytes::from(payload)
}

fn frame_cursor(frame: &ClientRealtimeFrameV1) -> Option<&str> {
    match frame.frame.as_ref()? {
        Frame::Event(event) => Some(&event.cursor),
        Frame::StreamState(state) if !state.cursor.is_empty() => Some(&state.cursor),
        Frame::ReplayGap(_) | Frame::StreamState(_) => None,
    }
}

fn valid_cursor(cursor: &str) -> bool {
    !cursor.is_empty()
        && cursor.len() <= MAX_CURSOR_BYTES
        && cursor
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && byte != b'\\' && byte != b'\"')
}

fn response(status: StatusCode, body: &'static str) -> GatewayHttpResponse {
    Response::builder()
        .status(status)
        .header(CACHE_CONTROL, "no-store")
        .body(full_gateway_body(body))
        .expect("Gateway realtime response is valid")
}
