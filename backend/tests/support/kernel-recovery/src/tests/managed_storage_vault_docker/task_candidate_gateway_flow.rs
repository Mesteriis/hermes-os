//! Gateway, SSE and disposable-storage probes for the reviewed candidate chain.

use super::*;

use std::time::Instant;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hermes_gateway_protocol::v1::{
    ClientRealtimeEventV1, ClientRealtimeFrameV1, client_realtime_frame_v1::Frame as RealtimeFrame,
};
use hermes_review_task_candidate_api::{
    REVIEW_TASK_CANDIDATE_COMMAND_CONNECT_PATH_V1, REVIEW_TASK_CANDIDATE_QUERY_CONNECT_PATH_V1,
    REVIEW_TASK_CANDIDATE_REALTIME_CONTRACT_NAME_V1, REVIEW_TASK_CANDIDATE_REALTIME_EVENT_KIND_V1,
    wire::{
        DecideReviewTaskCandidateRequestV1, DecideReviewTaskCandidateResponseV1,
        GetReviewTaskCandidateRequestV1, GetReviewTaskCandidateResponseV1,
        ReviewTaskCandidateDecisionV1, ReviewTaskCandidateErrorCodeV1,
        ReviewTaskCandidatePromotionStatusV1, ReviewTaskCandidateStateV1,
        ReviewTaskCandidateStatusChangedV1,
    },
};
use hermes_review_task_candidate_core::derive_review_task_candidate_id_v1;
use http_body_util::BodyExt as _;
use hyper::{Request, StatusCode, body::Bytes};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use zeroize::Zeroizing;

pub(super) type TaskCandidateGateway = hermes_gateway_runtime::GatewayApplicationRouter<
    crate::identity::browser_gateway::ControlStoreBrowserAuthority,
    hermes_gateway_runtime::InMemoryBrowserRealtimeSource,
>;

pub(super) struct SeededTaskCandidateReviewsV1 {
    pub approved_review_id: Vec<u8>,
    pub approved_candidate_id: Vec<u8>,
    pub rejected_review_id: Vec<u8>,
    pub rejected_candidate_id: Vec<u8>,
}

pub(super) struct TaskCandidateTerminalEventsV1 {
    pub approved: ClientRealtimeEventV1,
    pub rejected: ClientRealtimeEventV1,
}

pub(super) fn task_candidate_gateway_v1(
    store: &Arc<SqliteControlStore>,
    supervisor: &ManagedRuntimeSupervisor,
    root: &Path,
    data: &Path,
    realtime: hermes_gateway_runtime::InMemoryBrowserRealtimeSource,
) -> TaskCandidateGateway {
    let configuration = crate::platform::gateway::BrowserGatewayConfigurationV1::new(
        "127.0.0.1:9443".parse().expect("loopback Gateway address"),
        "https://hub.local".to_owned(),
        "hub.local".to_owned(),
        root.join("task-candidate-gateway-cert.der"),
        root.join("task-candidate-gateway-key.der"),
    )
    .expect("Task candidate Gateway configuration");
    crate::platform::gateway::gateway_service(
        Arc::clone(store),
        data,
        supervisor.clone(),
        realtime,
        &configuration,
        None,
    )
    .expect("compose Task candidate Gateway routes")
}

pub(super) fn seed_pending_task_candidate_reviews_v1(
    runtime: &tokio::runtime::Runtime,
) -> SeededTaskCandidateReviewsV1 {
    runtime.block_on(async {
        let pool = task_candidate_admin_pool_v1().await;
        let updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("wall clock")
            .as_secs()
            .saturating_sub(60) as i64;
        let approved_candidate_id = [0x31; 16];
        let approved_digest = [0x32; 32];
        let approved_source_id = [0x33_u8; 16];
        let rejected_candidate_id = [0x41; 16];
        let rejected_digest = [0x42; 32];
        let rejected_source_id = [0x43_u8; 16];
        let approved_review_id = derive_review_task_candidate_id_v1(
            TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1,
            &approved_candidate_id,
            &approved_digest,
        )
        .expect("approved review id");
        let rejected_review_id = derive_review_task_candidate_id_v1(
            TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1,
            &rejected_candidate_id,
            &rejected_digest,
        )
        .expect("rejected review id");
        for (review_id, candidate_id, digest, source_id, title) in [
            (
                approved_review_id,
                approved_candidate_id,
                approved_digest,
                approved_source_id,
                "Approved candidate title",
            ),
            (
                rejected_review_id,
                rejected_candidate_id,
                rejected_digest,
                rejected_source_id,
                "Rejected candidate title",
            ),
        ] {
            sqlx::query(
                "INSERT INTO hermes_data.review_task_candidate_state (
                   logical_owner_id, review_id, candidate_id, candidate_digest,
                   source_evidence_id, source_evidence_revision, title, due_text_hint,
                   assignee_label_hint, state, promotion_status, review_revision,
                   decided_by_owner_device_id, decided_at_unix_seconds, decided_at_nanos,
                   promoted_task_id, updated_at_unix_seconds, updated_at_nanos
                 ) VALUES ($1,$2,$3,$4,$5,1,$6,NULL,NULL,1,1,1,NULL,NULL,NULL,NULL,$7,0)",
            )
            .bind(TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1)
            .bind(review_id.as_slice())
            .bind(candidate_id.as_slice())
            .bind(digest.as_slice())
            .bind(source_id.as_slice())
            .bind(title)
            .bind(updated_at)
            .execute(&pool)
            .await
            .expect("seed pending Review candidate");
        }
        pool.close().await;
        SeededTaskCandidateReviewsV1 {
            approved_review_id: approved_review_id.to_vec(),
            approved_candidate_id: approved_candidate_id.to_vec(),
            rejected_review_id: rejected_review_id.to_vec(),
            rejected_candidate_id: rejected_candidate_id.to_vec(),
        }
    })
}

pub(super) fn decide_task_candidate_v1(
    router: &TaskCandidateGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    operation_id: u8,
    review_id: &[u8],
    expected_review_revision: u64,
    decision: ReviewTaskCandidateDecisionV1,
) -> DecideReviewTaskCandidateResponseV1 {
    post_proto(
        router,
        runtime,
        cookie,
        REVIEW_TASK_CANDIDATE_COMMAND_CONNECT_PATH_V1,
        DecideReviewTaskCandidateRequestV1 {
            protocol_major: 1,
            operation_id: vec![operation_id; 16],
            review_id: review_id.to_vec(),
            expected_review_revision,
            decision: decision as i32,
        },
    )
}

pub(super) fn query_task_candidate_v1(
    router: &TaskCandidateGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    review_id: &[u8],
) -> GetReviewTaskCandidateResponseV1 {
    post_proto(
        router,
        runtime,
        cookie,
        REVIEW_TASK_CANDIDATE_QUERY_CONNECT_PATH_V1,
        GetReviewTaskCandidateRequestV1 {
            protocol_major: 1,
            review_id: review_id.to_vec(),
        },
    )
}

pub(super) fn wait_for_task_candidate_terminal_states_v1(
    router: &TaskCandidateGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    reviews: &SeededTaskCandidateReviewsV1,
) -> (
    GetReviewTaskCandidateResponseV1,
    GetReviewTaskCandidateResponseV1,
) {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let approved =
            query_task_candidate_v1(router, runtime, cookie, &reviews.approved_review_id);
        let rejected =
            query_task_candidate_v1(router, runtime, cookie, &reviews.rejected_review_id);
        let approved_terminal = approved.review.as_ref().is_some_and(|review| {
            review.promotion_status
                == ReviewTaskCandidatePromotionStatusV1::ReviewTaskCandidatePromotionStatusSucceeded
                    as i32
                && review.review_revision == 3
        });
        let rejected_terminal = rejected.review.as_ref().is_some_and(|review| {
            review.state == ReviewTaskCandidateStateV1::ReviewTaskCandidateStateRejected as i32
                && review.review_revision == 2
        });
        if approved_terminal && rejected_terminal {
            return (approved, rejected);
        }
        assert!(
            Instant::now() < deadline,
            "Task candidate terminal state timeout: approved={approved:?}; rejected={rejected:?}; storage={}",
            task_candidate_storage_diagnostics_v1(runtime)
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn task_candidate_storage_diagnostics_v1(runtime: &tokio::runtime::Runtime) -> String {
    runtime.block_on(async {
        let pool = task_candidate_admin_pool_v1().await;
        let review_outbox: (i64, i64) = sqlx::query_as(
            "SELECT count(*), count(*) FILTER (WHERE published_at_unix_millis IS NOT NULL)
             FROM hermes_data.review_task_candidate_outbox",
        )
        .fetch_one(&pool)
        .await
        .expect("Review outbox diagnostics");
        let workflow: (i64, i64, i64) = sqlx::query_as(
            "SELECT
               (SELECT count(*) FROM hermes_data.reviewed_task_candidate_promotion_requests),
               (SELECT count(*) FROM hermes_data.reviewed_task_candidate_promotion_result_inbox),
               (SELECT count(*) FROM hermes_data.reviewed_task_candidate_promotion_outbox)",
        )
        .fetch_one(&pool)
        .await
        .expect("promotion workflow diagnostics");
        let tasks: (i64, i64, i64) = sqlx::query_as(
            "SELECT
               (SELECT count(*) FROM hermes_data.tasks_reviewed_candidate_inbox),
               (SELECT count(*) FROM hermes_data.tasks_state),
               (SELECT count(*) FROM hermes_data.tasks_outbox)",
        )
        .fetch_one(&pool)
        .await
        .expect("Tasks diagnostics");
        let review_results: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM hermes_data.review_task_candidate_promotion_inbox",
        )
        .fetch_one(&pool)
        .await
        .expect("Review result inbox diagnostics");
        pool.close().await;
        format!(
            "review_outbox={review_outbox:?}, workflow={workflow:?}, tasks={tasks:?}, review_results={review_results}"
        )
    })
}

pub(super) fn read_task_candidate_terminal_events_v1(
    router: &TaskCandidateGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    reviews: &SeededTaskCandidateReviewsV1,
) -> TaskCandidateTerminalEventsV1 {
    let observed = Arc::new(std::sync::Mutex::new(Vec::new()));
    let response = runtime.block_on(
        router.route(
            Request::builder()
                .method("GET")
                .uri("/api/realtime/v1/events")
                .header("cookie", cookie)
                .body(http_body_util::Full::new(Bytes::new()))
                .expect("Task candidate Gateway SSE request"),
        ),
    );
    assert_eq!(response.status(), StatusCode::OK);
    runtime.block_on(async {
        match tokio::time::timeout(
            Duration::from_secs(15),
            find_terminal_events(response.into_body(), reviews, Arc::clone(&observed)),
        )
        .await
        {
            Ok(events) => events,
            Err(_) => panic!(
                "Task candidate SSE timeout; observed={:?}",
                observed.lock().expect("observed Task candidate frames")
            ),
        }
    })
}

pub(super) fn assert_exact_task_materialization_v1(
    runtime: &tokio::runtime::Runtime,
    reviews: &SeededTaskCandidateReviewsV1,
) {
    runtime.block_on(async {
        let pool = task_candidate_admin_pool_v1().await;
        let approved: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM hermes_data.tasks_state
             WHERE logical_owner_id=$1 AND approved_candidate_id=$2",
        )
        .bind(TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1)
        .bind(&reviews.approved_candidate_id)
        .fetch_one(&pool)
        .await
        .expect("count approved candidate Tasks");
        let rejected: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM hermes_data.tasks_state
             WHERE logical_owner_id=$1 AND approved_candidate_id=$2",
        )
        .bind(TASK_CANDIDATE_LOGICAL_HUMAN_OWNER_ID_V1)
        .bind(&reviews.rejected_candidate_id)
        .fetch_one(&pool)
        .await
        .expect("count rejected candidate Tasks");
        assert_eq!(approved, 1, "approve must materialize exactly one Task");
        assert_eq!(rejected, 0, "reject must never materialize a Task");
        pool.close().await;
    });
}

fn post_proto<M, R>(
    router: &TaskCandidateGateway,
    runtime: &tokio::runtime::Runtime,
    cookie: &str,
    path: &str,
    message: M,
) -> R
where
    M: Message,
    R: Message + Default,
{
    let payload = message.encode_to_vec();
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        let response = runtime.block_on(
            router.route(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("content-type", "application/connect+proto")
                    .header("cookie", cookie)
                    .body(http_body_util::Full::new(Bytes::from(payload.clone())))
                    .expect("Task candidate Gateway request"),
            ),
        );
        let status = response.status();
        let bytes = runtime
            .block_on(response.into_body().collect())
            .expect("Task candidate Gateway response")
            .to_bytes();
        if status == StatusCode::SERVICE_UNAVAILABLE && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(25));
            continue;
        }
        assert_eq!(
            status,
            StatusCode::OK,
            "Task candidate Gateway response: {}",
            String::from_utf8_lossy(&bytes)
        );
        return R::decode(bytes.as_ref()).expect("decode Task candidate Gateway response");
    }
}

async fn find_terminal_events<B>(
    mut body: B,
    reviews: &SeededTaskCandidateReviewsV1,
    observed: Arc<std::sync::Mutex<Vec<(Vec<u8>, i32, i32, u64)>>>,
) -> TaskCandidateTerminalEventsV1
where
    B: hyper::body::Body<Data = Bytes> + Unpin,
    B::Error: std::fmt::Debug,
{
    let mut pending = Vec::new();
    let mut approved = None;
    let mut rejected = None;
    while let Some(frame) = body.frame().await {
        let frame = frame.expect("Task candidate SSE frame");
        let Ok(data) = frame.into_data() else {
            continue;
        };
        pending.extend_from_slice(&data);
        while let Some(boundary) = pending.windows(2).position(|window| window == b"\n\n") {
            let block = pending.drain(..boundary + 2).collect::<Vec<_>>();
            let text = std::str::from_utf8(&block).expect("Task candidate SSE UTF-8");
            let Some(encoded) = text.lines().find_map(|line| line.strip_prefix("data: ")) else {
                continue;
            };
            let bytes = URL_SAFE_NO_PAD
                .decode(encoded)
                .expect("decode Task candidate frame");
            let frame = ClientRealtimeFrameV1::decode(bytes.as_slice())
                .expect("Task candidate realtime frame");
            let Some(RealtimeFrame::Event(event)) = frame.frame else {
                continue;
            };
            if event.contract_name != REVIEW_TASK_CANDIDATE_REALTIME_CONTRACT_NAME_V1
                || event.event_kind != REVIEW_TASK_CANDIDATE_REALTIME_EVENT_KIND_V1
            {
                continue;
            }
            let payload = ReviewTaskCandidateStatusChangedV1::decode(event.payload.as_slice())
                .expect("Task candidate status payload");
            observed.lock().expect("record Task candidate frame").push((
                payload.review_id.clone(),
                payload.state,
                payload.promotion_status,
                payload.review_revision,
            ));
            if payload.review_id == reviews.approved_review_id
                && payload.state == ReviewTaskCandidateStateV1::ReviewTaskCandidateStateApproved as i32
                && payload.promotion_status
                    == ReviewTaskCandidatePromotionStatusV1::ReviewTaskCandidatePromotionStatusSucceeded as i32
                && payload.review_revision == 3
            {
                approved = Some(event);
            } else if payload.review_id == reviews.rejected_review_id
                && payload.state == ReviewTaskCandidateStateV1::ReviewTaskCandidateStateRejected as i32
                && payload.promotion_status
                    == ReviewTaskCandidatePromotionStatusV1::ReviewTaskCandidatePromotionStatusNotRequested as i32
                && payload.review_revision == 2
            {
                rejected = Some(event);
            }
            if approved.is_some() && rejected.is_some() {
                return TaskCandidateTerminalEventsV1 {
                    approved: approved.expect("approved terminal event"),
                    rejected: rejected.expect("rejected terminal event"),
                };
            }
        }
    }
    panic!("Gateway SSE closed before both Task candidate terminal events");
}

async fn task_candidate_admin_pool_v1() -> sqlx::PgPool {
    let password = Zeroizing::new(
        std::fs::read_to_string(required(
            "HERMES_STORAGE_AUTHENTICATED_POSTGRES_PASSWORD_FILE",
        ))
        .expect("read disposable PostgreSQL credential")
        .trim()
        .to_owned(),
    );
    let options = PgConnectOptions::new()
        .host(&required("HERMES_STORAGE_AUTHENTICATED_POSTGRES_HOST"))
        .port(
            required("HERMES_STORAGE_AUTHENTICATED_POSTGRES_PORT")
                .parse()
                .expect("valid PostgreSQL port"),
        )
        .username("hermes_postgres_admin")
        .password(password.as_str())
        .database("hermes_storage_authenticated")
        .ssl_mode(PgSslMode::Disable);
    PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("connect Task candidate conformance database")
}

pub(super) fn assert_task_candidate_response_states_v1(
    approved: &GetReviewTaskCandidateResponseV1,
    rejected: &GetReviewTaskCandidateResponseV1,
) {
    assert_eq!(
        approved.error,
        ReviewTaskCandidateErrorCodeV1::ReviewTaskCandidateErrorCodeUnspecified as i32
    );
    let approved = approved.review.as_ref().expect("approved Review state");
    assert_eq!(
        approved.promotion_status,
        ReviewTaskCandidatePromotionStatusV1::ReviewTaskCandidatePromotionStatusSucceeded as i32
    );
    assert_eq!(approved.review_revision, 3);
    assert_eq!(
        rejected.error,
        ReviewTaskCandidateErrorCodeV1::ReviewTaskCandidateErrorCodeUnspecified as i32
    );
    let rejected = rejected.review.as_ref().expect("rejected Review state");
    assert_eq!(
        rejected.state,
        ReviewTaskCandidateStateV1::ReviewTaskCandidateStateRejected as i32
    );
    assert_eq!(rejected.review_revision, 2);
}
