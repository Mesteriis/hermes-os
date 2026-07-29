//! Authenticated, replayable SSE transport for client-safe Gateway frames.

use std::collections::{BTreeMap, VecDeque};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use bytes::Bytes;
use futures_util::{StreamExt, stream};
use hermes_gateway_protocol::{
    v1::{
        ClientRealtimeFrameV1, ClientRealtimeStreamStateKindV1, ClientReplayGapV1,
        client_realtime_frame_v1::Frame,
    },
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

#[derive(Clone)]
pub struct InMemoryBrowserRealtimeSource {
    owners: Arc<Mutex<BTreeMap<String, OwnerRealtimeState>>>,
    history_limit: usize,
}

struct OwnerRealtimeState {
    history: VecDeque<ClientRealtimeFrameV1>,
    live: broadcast::Sender<ClientRealtimeFrameV1>,
}

#[derive(Clone)]
pub struct BrowserRealtimePublisherV1 {
    owner_id: String,
    source: InMemoryBrowserRealtimeSource,
}

impl InMemoryBrowserRealtimeSource {
    pub fn new(history_limit: usize) -> Result<Self, String> {
        if !(1..=16_384).contains(&history_limit) {
            return Err("Gateway realtime history limit is invalid".to_owned());
        }
        Ok(Self {
            owners: Arc::new(Mutex::new(BTreeMap::new())),
            history_limit,
        })
    }

    pub fn admit_owner(
        &self,
        owner_id: impl Into<String>,
    ) -> Result<BrowserRealtimePublisherV1, String> {
        let owner_id = owner_id.into();
        if !valid_owner_id(&owner_id) {
            return Err("Gateway realtime owner is invalid".to_owned());
        }
        let mut owners = self
            .owners
            .lock()
            .map_err(|_| "Gateway realtime state is unavailable".to_owned())?;
        ensure_owner_state(&mut owners, &owner_id, self.history_limit);
        Ok(BrowserRealtimePublisherV1 {
            owner_id,
            source: self.clone(),
        })
    }

    pub fn revoke_owner(&self, owner_id: &str) -> Result<bool, String> {
        let state = self
            .owners
            .lock()
            .map_err(|_| "Gateway realtime state is unavailable".to_owned())?
            .remove(owner_id);
        let Some(state) = state else {
            return Ok(false);
        };
        let cursor = state
            .history
            .back()
            .and_then(frame_cursor)
            .unwrap_or_default();
        let _ = state.live.send(stream_state(
            ClientRealtimeStreamStateKindV1::ClientRealtimeStreamStateKindClosed,
            cursor,
        ));
        Ok(true)
    }

    fn publish(&self, owner_id: &str, frame: ClientRealtimeFrameV1) -> Result<(), String> {
        validate_client_realtime_frame(&frame)?;
        let Some(Frame::Event(event)) = frame.frame.as_ref() else {
            return Err("Gateway realtime publisher accepts only owner events".to_owned());
        };
        let mut owners = self
            .owners
            .lock()
            .map_err(|_| "Gateway realtime state is unavailable".to_owned())?;
        let state = owners
            .get_mut(owner_id)
            .ok_or_else(|| "Gateway realtime owner is not admitted".to_owned())?;
        if let Some(existing) = state
            .history
            .iter()
            .find(|candidate| frame_cursor(candidate) == Some(event.cursor.as_str()))
        {
            return (existing == &frame)
                .then_some(())
                .ok_or_else(|| "Gateway realtime cursor conflicts".to_owned());
        }
        if state.history.len() == self.history_limit {
            state.history.pop_front();
        }
        state.history.push_back(frame.clone());
        let _ = state.live.send(frame);
        Ok(())
    }

    fn subscribe_owner(
        &self,
        owner_id: &str,
        after_cursor: Option<&str>,
    ) -> Result<ClientRealtimeSubscriptionV1, String> {
        let mut owners = self
            .owners
            .lock()
            .map_err(|_| "Gateway realtime state is unavailable".to_owned())?;
        ensure_owner_state(&mut owners, owner_id, self.history_limit);
        let state = owners
            .get(owner_id)
            .ok_or_else(|| "client realtime owner is unavailable".to_owned())?;
        let live = state.live.subscribe();
        let latest_cursor = state
            .history
            .back()
            .and_then(frame_cursor)
            .unwrap_or_default();
        let replay = match after_cursor {
            None => {
                let mut replay = Vec::with_capacity(state.history.len() + 1);
                replay.extend(state.history.iter().cloned());
                replay.push(stream_state(
                    ClientRealtimeStreamStateKindV1::ClientRealtimeStreamStateKindOpen,
                    latest_cursor,
                ));
                replay
            }
            Some(cursor) => {
                let Some(position) = state
                    .history
                    .iter()
                    .position(|frame| frame_cursor(frame) == Some(cursor))
                else {
                    let earliest = state
                        .history
                        .front()
                        .and_then(frame_cursor)
                        .unwrap_or_default()
                        .to_owned();
                    drop(owners);
                    let (closed, live) = broadcast::channel(1);
                    drop(closed);
                    return ClientRealtimeSubscriptionV1::new(
                        vec![replay_gap_with_earliest(
                            cursor,
                            &earliest,
                            "cursor_not_available",
                        )],
                        live,
                    );
                };
                let mut replay = Vec::with_capacity(state.history.len() - position + 2);
                replay.push(stream_state(
                    ClientRealtimeStreamStateKindV1::ClientRealtimeStreamStateKindReplaying,
                    cursor,
                ));
                replay.extend(state.history.iter().skip(position + 1).cloned());
                replay.push(stream_state(
                    ClientRealtimeStreamStateKindV1::ClientRealtimeStreamStateKindOpen,
                    latest_cursor,
                ));
                replay
            }
        };
        ClientRealtimeSubscriptionV1::new(replay, live)
    }
}

fn ensure_owner_state(
    owners: &mut BTreeMap<String, OwnerRealtimeState>,
    owner_id: &str,
    history_limit: usize,
) {
    owners.entry(owner_id.to_owned()).or_insert_with(|| {
        let (live, _) = broadcast::channel(history_limit);
        OwnerRealtimeState {
            history: VecDeque::with_capacity(history_limit),
            live,
        }
    });
}

impl BrowserRealtimePublisherV1 {
    pub fn publish(&self, frame: ClientRealtimeFrameV1) -> Result<(), String> {
        self.source.publish(&self.owner_id, frame)
    }
}

impl BrowserRealtimeSubscriptionSource for InMemoryBrowserRealtimeSource {
    fn subscribe(
        &self,
        session: &BrowserSession,
        after_cursor: Option<&str>,
    ) -> Result<ClientRealtimeSubscriptionV1, String> {
        self.subscribe_owner(session.owner_id(), after_cursor)
    }
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
    replay_gap_with_earliest(requested_cursor, "", reason_code)
}

fn replay_gap_with_earliest(
    requested_cursor: &str,
    earliest_available_cursor: &str,
    reason_code: &str,
) -> ClientRealtimeFrameV1 {
    ClientRealtimeFrameV1 {
        frame: Some(Frame::ReplayGap(ClientReplayGapV1 {
            requested_cursor: requested_cursor.to_owned(),
            earliest_available_cursor: earliest_available_cursor.to_owned(),
            reason_code: reason_code.to_owned(),
        })),
    }
}

fn stream_state(state: ClientRealtimeStreamStateKindV1, cursor: &str) -> ClientRealtimeFrameV1 {
    ClientRealtimeFrameV1 {
        frame: Some(Frame::StreamState(
            hermes_gateway_protocol::v1::ClientRealtimeStreamStateV1 {
                state: state as i32,
                cursor: cursor.to_owned(),
            },
        )),
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

fn valid_owner_id(owner_id: &str) -> bool {
    !owner_id.is_empty()
        && owner_id.len() <= 96
        && owner_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn response(status: StatusCode, body: &'static str) -> GatewayHttpResponse {
    Response::builder()
        .status(status)
        .header(CACHE_CONTROL, "no-store")
        .body(full_gateway_body(body))
        .expect("Gateway realtime response is valid")
}

#[cfg(test)]
mod tests {
    use hermes_gateway_protocol::v1::ClientRealtimeEventV1;
    use hermes_gateway_session::BrowserGatewaySessionService;
    use hermes_gateway_session_contract::{
        BrowserAssertionAuthority, BrowserAuthenticationAuthority, BrowserDeviceAuthority,
        BrowserDeviceCredentialV1, BrowserDevicePrincipalV1, GatewayIdentityFenceV1,
    };

    use super::*;

    struct TestBrowserAuthority;

    impl BrowserDeviceAuthority for TestBrowserAuthority {
        fn current_identity_fence(&self) -> Result<GatewayIdentityFenceV1, String> {
            Err("unused test authority".to_owned())
        }

        fn active_browser_device(
            &self,
            _device_id: &str,
        ) -> Result<BrowserDevicePrincipalV1, String> {
            Err("unused test authority".to_owned())
        }

        fn active_browser_device_by_credential(
            &self,
            _credential_id: &[u8],
        ) -> Result<BrowserDevicePrincipalV1, String> {
            Err("unused test authority".to_owned())
        }
    }

    impl BrowserAssertionAuthority for TestBrowserAuthority {
        fn accept_verified_browser_assertion(
            &self,
            _credential_id: &[u8],
            _sign_count: u32,
            _backup_eligible: bool,
            _backup_state: bool,
        ) -> Result<BrowserDevicePrincipalV1, String> {
            Err("unused test authority".to_owned())
        }
    }

    impl BrowserAuthenticationAuthority for TestBrowserAuthority {
        fn active_browser_credential(
            &self,
            _credential_id: &[u8],
        ) -> Result<BrowserDeviceCredentialV1, String> {
            Err("unused test authority".to_owned())
        }
    }

    #[test]
    fn authenticated_owner_can_subscribe_before_the_first_publication() {
        let source = InMemoryBrowserRealtimeSource::new(2).expect("source");
        let session = test_session();
        let (replay, mut live) = source
            .subscribe(&session, None)
            .expect("empty owner stream")
            .into_parts();
        assert!(matches!(
            replay.as_slice(),
            [ClientRealtimeFrameV1 {
                frame: Some(Frame::StreamState(state)),
            }] if state.state
                == ClientRealtimeStreamStateKindV1::ClientRealtimeStreamStateKindOpen as i32
        ));

        source
            .admit_owner("owner-1")
            .expect("publisher")
            .publish(event("cursor/1", 1))
            .expect("first publication");
        assert!(matches!(
            live.try_recv().expect("live event").frame,
            Some(Frame::Event(event)) if event.cursor == "cursor/1"
        ));
    }

    #[test]
    fn exact_duplicates_replay_live_delivery_and_bounded_gap_are_deterministic() {
        let source = InMemoryBrowserRealtimeSource::new(2).expect("source");
        let publisher = source.admit_owner("owner-1").expect("publisher");
        let session = test_session();
        let first = event("cursor/1", 1);
        publisher.publish(first.clone()).expect("first");
        publisher.publish(first).expect("exact duplicate");
        assert!(
            publisher.publish(event("cursor/1", 2)).is_err(),
            "same cursor with different bytes must fail closed"
        );
        publisher.publish(event("cursor/2", 2)).expect("second");

        let (initial_replay, _) = source
            .subscribe(&session, None)
            .expect("initial bounded replay")
            .into_parts();
        assert_eq!(initial_replay.len(), 3);
        assert!(matches!(
            initial_replay[0].frame.as_ref(),
            Some(Frame::Event(event)) if event.cursor == "cursor/1"
        ));
        assert!(matches!(
            initial_replay[1].frame.as_ref(),
            Some(Frame::Event(event)) if event.cursor == "cursor/2"
        ));

        let (replay, _) = source
            .subscribe(&session, Some("cursor/1"))
            .expect("replay")
            .into_parts();
        assert_eq!(replay.len(), 3);
        assert!(matches!(
            replay[1].frame.as_ref(),
            Some(Frame::Event(event)) if event.cursor == "cursor/2"
        ));

        let (_, mut live) = source
            .subscribe(&session, Some("cursor/2"))
            .expect("live")
            .into_parts();
        publisher.publish(event("cursor/3", 3)).expect("third");
        assert!(matches!(
            live.try_recv().expect("live event").frame,
            Some(Frame::Event(event)) if event.cursor == "cursor/3"
        ));

        let (gap, _) = source
            .subscribe(&session, Some("cursor/1"))
            .expect("gap")
            .into_parts();
        assert!(matches!(
            gap.as_slice(),
            [ClientRealtimeFrameV1 {
                frame: Some(Frame::ReplayGap(gap)),
            }] if gap.requested_cursor == "cursor/1"
                && gap.earliest_available_cursor == "cursor/2"
        ));
    }

    fn test_session() -> BrowserSession {
        BrowserGatewaySessionService::new_loopback_development(
            TestBrowserAuthority,
            "http://127.0.0.1:5173",
            "owner-1",
            "device-1",
        )
        .expect("service")
        .authorize_request(None)
        .expect("session")
    }

    fn event(cursor: &str, revision: u8) -> ClientRealtimeFrameV1 {
        ClientRealtimeFrameV1 {
            frame: Some(Frame::Event(ClientRealtimeEventV1 {
                event_id: vec![revision; 16],
                cursor: cursor.to_owned(),
                contract_name: "communication.delivery_intent.status_changed".to_owned(),
                contract_version: 1,
                event_kind: "delivery_intent_status_changed".to_owned(),
                occurred_at_unix_millis: u64::from(revision) * 1_000,
                causation_id: String::new(),
                correlation_id: String::new(),
                trace_id: String::new(),
                payload: vec![revision],
            })),
        }
    }
}
