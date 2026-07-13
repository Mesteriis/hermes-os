use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use hermes_events_api::NewEventEnvelope;
use hermes_provider_telemost::models::{
    TelemostCohost, YandexTelemostCohostPage, YandexTelemostConference,
    YandexTelemostConferencePatchRequest, YandexTelemostConferenceRequest,
    YandexTelemostCreateConferenceCommand,
};
use hermes_provider_telemost::protocol::{
    YANDEX_TELEMOST_API_BASE_URL, YANDEX_TELEMOST_PROVIDER_KIND_STR,
    sanitize_yandex_telemost_payload, validate_required, validate_telemost_join_url,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path as FsPath, PathBuf};
use uuid::Uuid;

use crate::app::api_support::{
    automation_calls::*,
    communications::*,
    ensure_fixture_routes_enabled,
    messaging_integrations::*,
    platform_dtos::*,
    query_parsing::{communication::*, documents::*, graph::*, personas::*, projects::*, tasks::*},
    review_commands::*,
    review_lists::*,
    stores::{ai_runtime::*, domain_stores::*, integration_stores::*, settings_vault::*},
    telegram_capabilities::*,
    whatsapp_capabilities::*,
};
use crate::app::{ApiError, AppState};
use crate::domains::calendar::events::CalendarEventQueryPort;
use crate::domains::review::{
    NewReviewItem, NewReviewItemEvidence, ReviewInboxStore, ReviewItemKind,
};
use crate::integrations::yandex_telemost::client::errors::YandexTelemostError;
use crate::integrations::yandex_telemost::client::models::{
    YANDEX_TELEMOST_WEB_ORIGIN, YandexTelemostAccountListResponse,
    YandexTelemostAccountSetupRequest, YandexTelemostAccountSetupResponse,
    YandexTelemostCapabilityState, YandexTelemostConferenceOpenRequest,
    YandexTelemostConferenceWebviewManifest, YandexTelemostLocalRecordingManifest,
    YandexTelemostLocalRecordingPolicy, YandexTelemostRecordingBridgeRequest,
    YandexTelemostRecordingBridgeResponse, YandexTelemostRetentionCleanupRequest,
    YandexTelemostRetentionCleanupResponse, YandexTelemostRuntimeStatus,
    YandexTelemostSpeakerTimelinePolicy, YandexTelemostTranscriptBridgeRequest,
    YandexTelemostTranscriptBridgeResponse, webview_manifest_for_request,
    yandex_telemost_capabilities,
};
use crate::integrations::yandex_telemost::runtime_bridge::complete_yandex_telemost_transcript_bridge;

use crate::platform::events::bus::yandex_telemost_event_types;
use crate::platform::realtime_conversation::bundle::build_call_bundle_manifest;
use crate::platform::realtime_conversation::events::{
    REALTIME_CONVERSATION_AUDIO_CAPTURE_COMPLETED, REALTIME_CONVERSATION_CALL_BUNDLE_CREATED,
    REALTIME_CONVERSATION_RADAR_SIGNALS_DETECTED, REALTIME_CONVERSATION_SPEAKER_HINT_OBSERVED,
    REALTIME_CONVERSATION_TRANSCRIPT_REQUESTED,
};
use crate::platform::realtime_conversation::models::{
    CallBundleManifest, RealtimeConversationProviderKind, SpeakerTimelineHint,
};
use crate::vault::VaultMode;
use crate::workflows::realtime_conversation_memory_pipeline::plan_memory_pipeline;
use crate::workflows::realtime_conversation_radar_projection::{
    RealtimeConversationRadarProjectionContext, call_bundle_radar_candidates,
};
use hermes_observations_api::models::{NewObservation, ObservationOriginKind};
use hermes_observations_postgres::store::ObservationStore;

const REALTIME_CONVERSATION_RADAR_SIGNAL_OBSERVATION_KIND: &str =
    "REALTIME_CONVERSATION_RADAR_SIGNAL";

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct YandexTelemostAccountsQuery {
    #[serde(default)]
    pub(crate) include_removed: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct YandexTelemostRuntimeStatusQuery {
    pub(crate) account_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct YandexTelemostCohostsQuery {
    #[serde(default)]
    pub(crate) offset: Option<u32>,
    #[serde(default)]
    pub(crate) limit: Option<u16>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct YandexTelemostCapabilitiesResponse {
    pub(crate) provider_kind: &'static str,
    pub(crate) api_base_url: &'static str,
    pub(crate) web_origin: &'static str,
    pub(crate) capabilities: Vec<YandexTelemostCapabilityState>,
    pub(crate) recording_policy: YandexTelemostLocalRecordingManifest,
    pub(crate) speaker_timeline_policy: YandexTelemostSpeakerTimelinePolicy,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct YandexTelemostConferenceOperationResponse {
    pub(crate) account_id: String,
    pub(crate) conference: YandexTelemostConference,
    pub(crate) status: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct YandexTelemostRecordingIntentResponse {
    pub(crate) account_id: String,
    pub(crate) conference_id: Option<String>,
    pub(crate) join_url: String,
    pub(crate) consent_required: bool,
    pub(crate) source_of_truth: bool,
    pub(crate) local_recording: YandexTelemostLocalRecordingManifest,
    pub(crate) speaker_timeline: YandexTelemostSpeakerTimelinePolicy,
    pub(crate) tauri_commands: serde_json::Value,
}

pub(crate) async fn get_yandex_telemost_capabilities(
    State(state): State<AppState>,
) -> Result<Json<YandexTelemostCapabilitiesResponse>, ApiError> {
    let authorized = matches!(state.vault.status()?.state, VaultMode::Unlocked);
    Ok(Json(YandexTelemostCapabilitiesResponse {
        provider_kind: YANDEX_TELEMOST_PROVIDER_KIND_STR,
        api_base_url: YANDEX_TELEMOST_API_BASE_URL,
        web_origin: YANDEX_TELEMOST_WEB_ORIGIN,
        capabilities: yandex_telemost_capabilities(authorized),
        recording_policy: recording_policy_manifest(),
        speaker_timeline_policy: speaker_timeline_policy(),
    }))
}

pub(crate) async fn get_yandex_telemost_accounts(
    State(state): State<AppState>,
    Query(query): Query<YandexTelemostAccountsQuery>,
) -> Result<Json<YandexTelemostAccountListResponse>, ApiError> {
    Ok(Json(
        yandex_telemost_provider_runtime_store(&state)?
            .list_accounts(query.include_removed)
            .await?,
    ))
}

pub(crate) async fn post_yandex_telemost_account(
    State(state): State<AppState>,
    Json(request): Json<YandexTelemostAccountSetupRequest>,
) -> Result<Json<YandexTelemostAccountSetupResponse>, ApiError> {
    require_yandex_telemost_unlocked_host_vault(&state)?;
    let store = yandex_telemost_provider_runtime_store(&state)?;
    let secret_store = yandex_telemost_secret_reference_store(&state)?;
    Ok(Json(
        store
            .setup_account(&secret_store, &state.vault, &request)
            .await?,
    ))
}

pub(crate) async fn get_yandex_telemost_runtime_status(
    State(state): State<AppState>,
    Query(query): Query<YandexTelemostRuntimeStatusQuery>,
) -> Result<Json<YandexTelemostRuntimeStatus>, ApiError> {
    Ok(Json(
        yandex_telemost_provider_runtime_store(&state)?
            .runtime_status(&query.account_id)
            .await?,
    ))
}

pub(crate) async fn post_yandex_telemost_retention_cleanup(
    State(state): State<AppState>,
    Path(account_id): Path<String>,
    Json(request): Json<YandexTelemostRetentionCleanupRequest>,
) -> Result<Json<YandexTelemostRetentionCleanupResponse>, ApiError> {
    Ok(Json(
        yandex_telemost_provider_runtime_service(&state)?
            .cleanup_retention(&account_id, &request)
            .await?,
    ))
}

pub(crate) async fn post_yandex_telemost_conference(
    State(state): State<AppState>,
    Json(command): Json<YandexTelemostCreateConferenceCommand>,
) -> Result<Json<YandexTelemostConferenceOperationResponse>, ApiError> {
    require_yandex_telemost_unlocked_host_vault(&state)?;
    let store = yandex_telemost_provider_runtime_store(&state)?;
    let secret_store = yandex_telemost_secret_reference_store(&state)?;
    let conference = store
        .create_conference(
            &secret_store,
            &state.vault,
            &command.account_id,
            &command.body,
        )
        .await?;
    Ok(Json(YandexTelemostConferenceOperationResponse {
        account_id: command.account_id,
        conference,
        status: "created",
    }))
}

pub(crate) async fn get_yandex_telemost_conference(
    State(state): State<AppState>,
    Path((account_id, conference_id)): Path<(String, String)>,
) -> Result<Json<YandexTelemostConferenceOperationResponse>, ApiError> {
    require_yandex_telemost_unlocked_host_vault(&state)?;
    let store = yandex_telemost_provider_runtime_store(&state)?;
    let secret_store = yandex_telemost_secret_reference_store(&state)?;
    let conference = store
        .get_conference(&secret_store, &state.vault, &account_id, &conference_id)
        .await?;
    Ok(Json(YandexTelemostConferenceOperationResponse {
        account_id,
        conference,
        status: "observed",
    }))
}

pub(crate) async fn patch_yandex_telemost_conference(
    State(state): State<AppState>,
    Path((account_id, conference_id)): Path<(String, String)>,
    Json(request): Json<YandexTelemostConferencePatchRequest>,
) -> Result<Json<YandexTelemostConferenceOperationResponse>, ApiError> {
    require_yandex_telemost_unlocked_host_vault(&state)?;
    let store = yandex_telemost_provider_runtime_store(&state)?;
    let secret_store = yandex_telemost_secret_reference_store(&state)?;
    let conference = store
        .update_conference(
            &secret_store,
            &state.vault,
            &account_id,
            &conference_id,
            &request,
        )
        .await?;
    Ok(Json(YandexTelemostConferenceOperationResponse {
        account_id,
        conference,
        status: "updated",
    }))
}

pub(crate) async fn get_yandex_telemost_cohosts(
    State(state): State<AppState>,
    Path((account_id, conference_id)): Path<(String, String)>,
    Query(query): Query<YandexTelemostCohostsQuery>,
) -> Result<Json<YandexTelemostCohostPage>, ApiError> {
    require_yandex_telemost_unlocked_host_vault(&state)?;
    let store = yandex_telemost_provider_runtime_store(&state)?;
    let secret_store = yandex_telemost_secret_reference_store(&state)?;
    Ok(Json(
        store
            .list_cohosts(
                &secret_store,
                &state.vault,
                &account_id,
                &conference_id,
                query.offset,
                query.limit,
            )
            .await?,
    ))
}

pub(crate) async fn post_yandex_telemost_webview_manifest(
    State(state): State<AppState>,
    Json(request): Json<YandexTelemostConferenceOpenRequest>,
) -> Result<Json<YandexTelemostConferenceWebviewManifest>, ApiError> {
    validate_telemost_join_url(&request.join_url).map_err(YandexTelemostError::from)?;
    let window_label = telemost_window_label(&request.account_id, request.conference_id.as_deref());
    publish_yandex_telemost_companion_event(
        &state,
        yandex_telemost_event_types::WEBVIEW_OPEN_REQUESTED,
        "webview_open_requested",
        &request,
        json!({ "window_label": window_label.clone(), "owner_visible": true, "hidden_headless_mode": "forbidden" }),
    )
    .await?;
    Ok(Json(webview_manifest_for_request(
        &request,
        window_label,
        false,
        false,
    )))
}

pub(crate) async fn post_yandex_telemost_recording_intent(
    State(state): State<AppState>,
    Json(request): Json<YandexTelemostConferenceOpenRequest>,
) -> Result<Json<YandexTelemostRecordingIntentResponse>, ApiError> {
    validate_telemost_join_url(&request.join_url).map_err(YandexTelemostError::from)?;
    publish_yandex_telemost_companion_event(
        &state,
        yandex_telemost_event_types::LOCAL_RECORDING_REQUESTED,
        "local_recording_requested",
        &request,
        json!({
            "consent_required": true,
            "source_of_truth": false,
            "audio_format": "mp3",
            "speaker_timeline": "hint_not_truth"
        }),
    )
    .await?;
    Ok(Json(YandexTelemostRecordingIntentResponse {
        account_id: request.account_id,
        conference_id: request.conference_id,
        join_url: request.join_url,
        consent_required: true,
        source_of_truth: false,
        local_recording: recording_policy_manifest(),
        speaker_timeline: speaker_timeline_policy(),
        tauri_commands: json!({
            "open_webview": "open_yandex_telemost_companion",
            "prepare_audio_device": "yandex_telemost_prepare_audio_device",
            "start_recording": "yandex_telemost_recording_start",
            "stop_recording": "yandex_telemost_recording_stop",
            "append_speaker_hint": "yandex_telemost_speaker_timeline_append"
        }),
    }))
}

pub(crate) async fn post_yandex_telemost_runtime_bridge_recording(
    State(state): State<AppState>,
    Json(request): Json<YandexTelemostRecordingBridgeRequest>,
) -> Result<Json<YandexTelemostRecordingBridgeResponse>, ApiError> {
    validate_yandex_telemost_recording_bridge_request(&request)?;
    let matched_calendar_event_id = matched_telemost_calendar_event_id(&state, &request).await?;
    let retention_policy = yandex_telemost_local_recording_retention_policy(
        &state,
        recording_bridge_observed_at(&request),
    )
    .await?;
    let materialized = materialize_yandex_telemost_call_bundle(
        &request,
        matched_calendar_event_id,
        retention_policy,
    )?;
    publish_local_recording_completed_event(&state, &request, &materialized).await?;
    publish_realtime_conversation_bootstrap_events(&state, &request, &materialized).await?;
    mirror_radar_candidates_into_review(&state, &request, &materialized).await?;
    Ok(Json(YandexTelemostRecordingBridgeResponse {
        account_id: request.account_id,
        conference_id: request.conference_id,
        recording_session_id: request.recording_session_id,
        bundle_id: materialized.manifest.bundle_id.clone(),
        bundle_root: materialized.bundle_root.to_string_lossy().into_owned(),
        manifest_path: materialized.manifest_path.to_string_lossy().into_owned(),
        follow_up_events: materialized.pipeline_plan.follow_up_events.clone(),
        radar_signal_kinds: materialized
            .radar_candidates
            .iter()
            .map(|candidate| candidate.signal_kind.clone())
            .collect(),
    }))
}

pub(crate) async fn post_yandex_telemost_runtime_bridge_transcript(
    State(state): State<AppState>,
    Json(request): Json<YandexTelemostTranscriptBridgeRequest>,
) -> Result<Json<YandexTelemostTranscriptBridgeResponse>, ApiError> {
    let response = complete_yandex_telemost_transcript_bridge(
        &event_store(&state).map_err(yandex_telemost_event_store_access_error)?,
        Some(&state.event_bus),
        &request,
    )
    .await?;
    Ok(Json(response))
}

async fn publish_yandex_telemost_companion_event(
    state: &AppState,
    event_type: &str,
    event_name: &'static str,
    request: &YandexTelemostConferenceOpenRequest,
    extra: Value,
) -> Result<(), ApiError> {
    let entity_id = request
        .conference_id
        .as_deref()
        .unwrap_or("manual_join_url");
    let event = NewEventEnvelope::builder(
        format!(
            "yandex-telemost-companion-{}-{}-{}",
            request.account_id,
            event_name,
            Uuid::new_v4()
        ),
        event_type,
        Utc::now(),
        json!({ "source_code": "integration.yandex_telemost", "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR }),
        json!({ "kind": "telemost_conference", "entity_id": entity_id }),
    )
    .payload(sanitize_yandex_telemost_payload(json!({
        "account_id": request.account_id,
        "conference_id": request.conference_id,
        "join_url": request.join_url,
        "display_name": request.display_name,
        "event_name": event_name,
        "extra": extra,
    })))
    .provenance(json!({ "origin": "hermes_desktop_companion_contract" }))
    .correlation_id(format!("yandex-telemost:{}:{}", request.account_id, entity_id))
    .build()?;
    if event_store(state)?
        .append_for_dispatch_idempotent(&event)
        .await?
        .is_some()
    {
        state.event_bus.broadcast(event);
    }
    Ok(())
}

struct MaterializedTelemostCallBundle {
    bundle_root: PathBuf,
    manifest_path: PathBuf,
    manifest: CallBundleManifest,
    pipeline_plan:
        crate::workflows::realtime_conversation_memory_pipeline::RealtimeConversationMemoryPipelinePlan,
    radar_candidates:
        Vec<crate::workflows::realtime_conversation_radar_projection::RealtimeConversationRadarSignalCandidate>,
    speaker_hints: Vec<SpeakerTimelineHint>,
}

fn validate_yandex_telemost_recording_bridge_request(
    request: &YandexTelemostRecordingBridgeRequest,
) -> Result<(), YandexTelemostError> {
    if !request.consent_attested {
        return Err(YandexTelemostError::InvalidRequest(
            "Telemost recording bridge requires consent_attested=true".to_owned(),
        ));
    }
    if request.stopped_at_epoch_ms < request.started_at_epoch_ms {
        return Err(YandexTelemostError::InvalidRequest(
            "Telemost recording stop time must be after start time".to_owned(),
        ));
    }
    validate_required("account_id", &request.account_id)?;
    validate_required("recording_session_id", &request.recording_session_id)?;
    validate_telemost_join_url(&request.join_url)?;
    Ok(())
}

async fn matched_telemost_calendar_event_id(
    state: &AppState,
    request: &YandexTelemostRecordingBridgeRequest,
) -> Result<Option<String>, ApiError> {
    let Some(pool) = state.database.pool() else {
        return Ok(None);
    };
    let matched = CalendarEventQueryPort::new(pool.clone())
        .find_yandex_telemost_conference_match(
            Some(&request.join_url),
            request
                .conference_id
                .as_deref()
                .unwrap_or(request.recording_session_id.as_str()),
        )
        .await?;
    Ok(matched.map(|event| event.event_id))
}

async fn yandex_telemost_local_recording_retention_policy(
    state: &AppState,
    observed_at: chrono::DateTime<Utc>,
) -> Result<Value, ApiError> {
    let store = settings_store(state)?;
    store.repair_declared_settings().await?;
    let recording_retention_days = store
        .setting("privacy.yandex_telemost_recording_retention_days")
        .await?
        .and_then(|setting| setting.value.as_i64())
        .unwrap_or(0)
        .max(0);
    let speaker_hint_retention_days = store
        .setting("privacy.yandex_telemost_speaker_timeline_retention_days")
        .await?
        .and_then(|setting| setting.value.as_i64())
        .unwrap_or(0)
        .max(0);
    Ok(json!({
        "local_recording": {
            "recording_retention_days": recording_retention_days,
            "speaker_hint_retention_days": speaker_hint_retention_days,
            "audio_expires_at": if recording_retention_days > 0 {
                Some(observed_at + chrono::TimeDelta::days(recording_retention_days))
            } else {
                None::<chrono::DateTime<Utc>>
            },
            "speaker_hints_expires_at": if speaker_hint_retention_days > 0 {
                Some(observed_at + chrono::TimeDelta::days(speaker_hint_retention_days))
            } else {
                None::<chrono::DateTime<Utc>>
            }
        }
    }))
}

fn materialize_yandex_telemost_call_bundle(
    request: &YandexTelemostRecordingBridgeRequest,
    calendar_event_id: Option<String>,
    retention_policy: Value,
) -> Result<MaterializedTelemostCallBundle, YandexTelemostError> {
    let bundle_root = canonical_existing_dir("output_dir", &request.output_dir)?;
    let audio_path = canonical_existing_file("audio_path", &request.audio_path, &bundle_root)?;
    let speaker_jsonl_path = canonical_existing_file(
        "speaker_jsonl_path",
        &request.speaker_jsonl_path,
        &bundle_root,
    )?;
    let speaker_txt_path =
        canonical_existing_file("speaker_txt_path", &request.speaker_txt_path, &bundle_root)?;
    let mut manifest = build_call_bundle_manifest(
        request.recording_session_id.clone(),
        RealtimeConversationProviderKind::YandexTelemost,
        "visible_webview_local_capture",
        request.account_id.clone(),
        request.conference_id.clone(),
        Some(request.join_url.clone()),
        bundle_root.to_string_lossy().into_owned(),
    );
    manifest.calendar_event_id = calendar_event_id;
    if let Some(provenance) = manifest.provenance.as_object_mut() {
        provenance.insert(
            "capture_mode".to_owned(),
            json!("visible_webview_local_loopback"),
        );
        provenance.insert("consent_attested".to_owned(), json!(true));
        provenance.insert("hidden_capture".to_owned(), json!(false));
        provenance.insert("provider_recording".to_owned(), json!(false));
        provenance.insert("local_only".to_owned(), json!(true));
        provenance.insert("retention_policy".to_owned(), retention_policy);
    }

    fs::create_dir_all(bundle_root.join(&manifest.layout.screenshots_dir))?;
    fs::create_dir_all(bundle_root.join(&manifest.layout.attachments_dir))?;
    fs::create_dir_all(bundle_root.join(&manifest.layout.ocr_dir))?;

    let speaker_hints_path = bundle_root.join(&manifest.layout.speaker_hints_jsonl);
    if speaker_hints_path != speaker_jsonl_path {
        fs::copy(&speaker_jsonl_path, &speaker_hints_path)?;
    }
    let speaker_hints = read_speaker_timeline_hints(&speaker_hints_path)?;

    let event_track_path = bundle_root.join(&manifest.layout.event_track_jsonl);
    write_jsonl_lines(
        &event_track_path,
        &[
            json!({
                "occurred_at_epoch_ms": request.started_at_epoch_ms,
                "event_kind": "audio_capture_started",
                "label": "Telemost local recording started",
                "confidence": 1.0,
                "source": "tauri_local_recorder",
                "evidence": {
                    "recording_session_id": request.recording_session_id,
                    "audio_path": audio_path.to_string_lossy(),
                }
            }),
            json!({
                "occurred_at_epoch_ms": request.stopped_at_epoch_ms,
                "event_kind": "audio_capture_completed",
                "label": "Telemost local recording completed",
                "confidence": 1.0,
                "source": "tauri_local_recorder",
                "evidence": {
                    "recording_session_id": request.recording_session_id,
                    "audio_path": audio_path.to_string_lossy(),
                    "speaker_txt_path": speaker_txt_path.to_string_lossy(),
                }
            }),
        ],
    )?;

    fs::write(
        bundle_root.join(&manifest.layout.meeting_json),
        serde_json::to_string_pretty(&json!({
            "bundle_id": manifest.bundle_id,
            "account_id": request.account_id,
            "conference_id": request.conference_id,
            "join_url": request.join_url,
            "started_at_epoch_ms": request.started_at_epoch_ms,
            "stopped_at_epoch_ms": request.stopped_at_epoch_ms,
            "consent_attested": request.consent_attested,
        }))?,
    )?;
    fs::write(
        bundle_root.join(&manifest.layout.provider_json),
        serde_json::to_string_pretty(&json!({
            "provider_kind": "yandex_telemost",
            "provider_shape": "visible_webview_local_capture",
            "source_of_truth": false,
            "speaker_hints_truth_status": "hint_not_truth",
            "audio_capture_mode": "local_loopback_mp3",
        }))?,
    )?;
    fs::write(
        bundle_root.join(&manifest.layout.participants_json),
        serde_json::to_string_pretty(&json!([]))?,
    )?;

    let pipeline_plan = plan_memory_pipeline(&manifest);
    let radar_candidates = call_bundle_radar_candidates(
        &manifest,
        &RealtimeConversationRadarProjectionContext {
            recording_session_id: Some(request.recording_session_id.clone()),
            ..RealtimeConversationRadarProjectionContext::default()
        },
    );
    fs::write(
        bundle_root.join(&manifest.layout.radar_signals_json),
        serde_json::to_string_pretty(&radar_candidates)?,
    )?;
    fs::write(
        bundle_root.join(&manifest.layout.metrics_json),
        serde_json::to_string_pretty(&json!({
            "speaker_hint_count": speaker_hints.len(),
            "radar_signal_count": radar_candidates.len(),
            "artifacts_ready": ["audio.mp3", "speaker-hints.jsonl", "speaker-timeline.txt", "event-track.jsonl", "radar-signals.json"],
        }))?,
    )?;
    manifest.artifacts.push(
        crate::platform::realtime_conversation::models::CallBundleArtifact {
            kind: "radar_signals".to_owned(),
            relative_path: manifest.layout.radar_signals_json.clone(),
            source: "telemost_runtime_bootstrap".to_owned(),
            truth_status: "candidate_output".to_owned(),
            media_type: Some("application/json".to_owned()),
            description: Some(
                "Bootstrap Radar candidates derived from local Telemost capture context".to_owned(),
            ),
        },
    );
    let manifest_path = bundle_root.join(&manifest.layout.manifest);
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

    Ok(MaterializedTelemostCallBundle {
        bundle_root,
        manifest_path,
        manifest,
        pipeline_plan,
        radar_candidates,
        speaker_hints,
    })
}

async fn publish_local_recording_completed_event(
    state: &AppState,
    request: &YandexTelemostRecordingBridgeRequest,
    materialized: &MaterializedTelemostCallBundle,
) -> Result<(), YandexTelemostError> {
    let entity_id = request
        .conference_id
        .as_deref()
        .unwrap_or(request.recording_session_id.as_str());
    let event = NewEventEnvelope::builder(
        format!(
            "yandex-telemost-recording-completed-{}-{}",
            request.account_id,
            Uuid::new_v4()
        ),
        yandex_telemost_event_types::LOCAL_RECORDING_COMPLETED,
        Utc::now(),
        json!({ "source_code": "integration.yandex_telemost", "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR }),
        json!({ "kind": "telemost_conference", "entity_id": entity_id }),
    )
    .payload(sanitize_yandex_telemost_payload(json!({
        "account_id": request.account_id,
        "conference_id": request.conference_id,
        "join_url": request.join_url,
        "recording_session_id": request.recording_session_id,
        "bundle_id": materialized.manifest.bundle_id,
        "bundle_root": materialized.bundle_root.to_string_lossy(),
        "manifest_path": materialized.manifest_path.to_string_lossy(),
        "audio_path": request.audio_path,
        "speaker_jsonl_path": request.speaker_jsonl_path,
        "speaker_txt_path": request.speaker_txt_path,
        "consent_attested": request.consent_attested,
        "stopped_at_epoch_ms": request.stopped_at_epoch_ms,
    })))
    .provenance(json!({ "origin": "hermes_desktop_companion_runtime_bridge" }))
    .correlation_id(format!("yandex-telemost:{}:{}", request.account_id, entity_id))
    .build()?;
    append_and_broadcast(state, &event).await
}

async fn publish_realtime_conversation_bootstrap_events(
    state: &AppState,
    request: &YandexTelemostRecordingBridgeRequest,
    materialized: &MaterializedTelemostCallBundle,
) -> Result<(), YandexTelemostError> {
    let entity_id = materialized.manifest.bundle_id.as_str();
    let audio_capture_completed = NewEventEnvelope::builder(
        format!("realtime-conversation-audio-capture-{}-{}", entity_id, Uuid::new_v4()),
        REALTIME_CONVERSATION_AUDIO_CAPTURE_COMPLETED,
        Utc::now(),
        json!({ "source_code": "platform.realtime_conversation", "provider": "yandex_telemost" }),
        json!({ "kind": "call_bundle", "entity_id": entity_id }),
    )
    .payload(json!({
        "bundle_id": materialized.manifest.bundle_id,
        "audio_path": request.audio_path,
        "speaker_hints_path": materialized.bundle_root.join(&materialized.manifest.layout.speaker_hints_jsonl).to_string_lossy(),
        "stopped_at_epoch_ms": request.stopped_at_epoch_ms,
    }))
    .provenance(json!({ "origin": "telemost_runtime_bridge" }))
    .correlation_id(format!("realtime-conversation:{}", entity_id))
    .build()?;
    append_and_broadcast(state, &audio_capture_completed).await?;
    publish_speaker_hint_events(state, request, materialized).await?;

    let bundle_created = NewEventEnvelope::builder(
        format!(
            "realtime-conversation-bundle-created-{}-{}",
            entity_id,
            Uuid::new_v4()
        ),
        REALTIME_CONVERSATION_CALL_BUNDLE_CREATED,
        Utc::now(),
        json!({ "source_code": "platform.realtime_conversation", "provider": "yandex_telemost" }),
        json!({ "kind": "call_bundle", "entity_id": entity_id }),
    )
    .payload(serde_json::to_value(&materialized.manifest)?)
    .provenance(json!({ "origin": "telemost_runtime_bridge" }))
    .correlation_id(format!("realtime-conversation:{}", entity_id))
    .build()?;
    append_and_broadcast(state, &bundle_created).await?;

    let transcript_requested = NewEventEnvelope::builder(
        format!(
            "realtime-conversation-transcript-requested-{}-{}",
            entity_id,
            Uuid::new_v4()
        ),
        REALTIME_CONVERSATION_TRANSCRIPT_REQUESTED,
        Utc::now(),
        json!({ "source_code": "workflow.realtime_conversation", "provider": "yandex_telemost" }),
        json!({ "kind": "call_bundle", "entity_id": entity_id }),
    )
    .payload(serde_json::to_value(&materialized.pipeline_plan)?)
    .provenance(json!({ "origin": "telemost_local_recording_pipeline_bootstrap" }))
    .correlation_id(format!("realtime-conversation:{}", entity_id))
    .build()?;
    append_and_broadcast(state, &transcript_requested).await?;

    let radar_event = NewEventEnvelope::builder(
        format!(
            "realtime-conversation-radar-signals-{}-{}",
            entity_id,
            Uuid::new_v4()
        ),
        REALTIME_CONVERSATION_RADAR_SIGNALS_DETECTED,
        Utc::now(),
        json!({ "source_code": "workflow.realtime_conversation", "provider": "yandex_telemost" }),
        json!({ "kind": "call_bundle", "entity_id": entity_id }),
    )
    .payload(serde_json::to_value(&materialized.radar_candidates)?)
    .provenance(json!({ "origin": "telemost_local_recording_pipeline_bootstrap" }))
    .correlation_id(format!("realtime-conversation:{}", entity_id))
    .build()?;
    append_and_broadcast(state, &radar_event).await
}

async fn publish_speaker_hint_events(
    state: &AppState,
    request: &YandexTelemostRecordingBridgeRequest,
    materialized: &MaterializedTelemostCallBundle,
) -> Result<(), YandexTelemostError> {
    for (index, hint) in materialized.speaker_hints.iter().enumerate() {
        let subject_entity_id = request
            .conference_id
            .as_deref()
            .unwrap_or(request.recording_session_id.as_str());
        let integration_event = NewEventEnvelope::builder(
            format!(
                "yandex-telemost-speaker-hint-{}-{}-{}",
                request.account_id, materialized.manifest.bundle_id, index
            ),
            yandex_telemost_event_types::SPEAKER_HINT_OBSERVED,
            Utc::now(),
            json!({ "source_code": "integration.yandex_telemost", "provider": YANDEX_TELEMOST_PROVIDER_KIND_STR }),
            json!({ "kind": "telemost_conference", "entity_id": subject_entity_id }),
        )
        .payload(sanitize_yandex_telemost_payload(serde_json::to_value(hint)?))
        .provenance(json!({
            "origin": "telemost_local_speaker_hint_file",
            "bundle_id": materialized.manifest.bundle_id,
        }))
        .correlation_id(format!(
            "yandex-telemost:{}:{}",
            request.account_id, materialized.manifest.bundle_id
        ))
        .build()?;
        append_and_broadcast(state, &integration_event).await?;

        let realtime_event = NewEventEnvelope::builder(
            format!(
                "realtime-conversation-speaker-hint-{}-{}-{}",
                request.account_id, materialized.manifest.bundle_id, index
            ),
            REALTIME_CONVERSATION_SPEAKER_HINT_OBSERVED,
            Utc::now(),
            json!({ "source_code": "platform.realtime_conversation", "provider": "yandex_telemost" }),
            json!({ "kind": "call_bundle", "entity_id": materialized.manifest.bundle_id }),
        )
        .payload(serde_json::to_value(hint)?)
        .provenance(json!({ "origin": "telemost_local_speaker_hint_file" }))
        .correlation_id(format!(
            "realtime-conversation:{}",
            materialized.manifest.bundle_id
        ))
        .build()?;
        append_and_broadcast(state, &realtime_event).await?;
    }
    Ok(())
}

async fn mirror_radar_candidates_into_review(
    state: &AppState,
    request: &YandexTelemostRecordingBridgeRequest,
    materialized: &MaterializedTelemostCallBundle,
) -> Result<(), YandexTelemostError> {
    let Some(pool) = state.database.pool() else {
        return Err(yandex_telemost_event_store_access_error(
            ApiError::DatabaseNotConfigured,
        ));
    };
    let observation_store = app_store::<ObservationStore>(pool.clone());
    let review_store = app_store::<ReviewInboxStore>(pool.clone());
    let observed_at = recording_bridge_observed_at(request);

    for candidate in &materialized.radar_candidates {
        let source_ref = format!(
            "call-bundle://{}/radar/{}",
            materialized.manifest.bundle_id, candidate.signal_kind
        );
        let observation = observation_store
            .capture(
                &NewObservation::new(
                    REALTIME_CONVERSATION_RADAR_SIGNAL_OBSERVATION_KIND,
                    ObservationOriginKind::LocalRuntime,
                    observed_at,
                    radar_signal_observation_payload(request, materialized, candidate),
                    source_ref,
                )
                .confidence(candidate.confidence as f64)
                .provenance(json!({
                    "captured_by": "integration.yandex_telemost.runtime_bridge_recording",
                    "provider_kind": YANDEX_TELEMOST_PROVIDER_KIND_STR,
                    "bundle_id": materialized.manifest.bundle_id,
                    "recording_session_id": request.recording_session_id,
                })),
            )
            .await?;
        let review_item =
            radar_signal_review_item(request, &materialized.manifest.bundle_id, candidate);
        let evidence = NewReviewItemEvidence::new(observation.observation_id)
            .role("primary")
            .metadata(json!({
                "mirrored_from": "realtime_conversation_radar_signal",
                "signal_kind": candidate.signal_kind,
                "bundle_id": materialized.manifest.bundle_id,
            }));
        let _ = review_store
            .create_with_evidence(&review_item, &[evidence])
            .await?;
    }

    Ok(())
}

async fn append_and_broadcast(
    state: &AppState,
    event: &NewEventEnvelope,
) -> Result<(), YandexTelemostError> {
    let store = event_store(state).map_err(yandex_telemost_event_store_access_error)?;
    if store.append_for_dispatch_idempotent(event).await?.is_some() {
        state.event_bus.broadcast(event.clone());
    }
    Ok(())
}

fn recording_bridge_observed_at(
    request: &YandexTelemostRecordingBridgeRequest,
) -> chrono::DateTime<Utc> {
    chrono::DateTime::<Utc>::from_timestamp_millis(request.stopped_at_epoch_ms as i64)
        .unwrap_or_else(Utc::now)
}

fn radar_signal_observation_payload(
    request: &YandexTelemostRecordingBridgeRequest,
    materialized: &MaterializedTelemostCallBundle,
    candidate: &crate::workflows::realtime_conversation_radar_projection::RealtimeConversationRadarSignalCandidate,
) -> Value {
    json!({
        "provider_kind": YANDEX_TELEMOST_PROVIDER_KIND_STR,
        "bundle_id": materialized.manifest.bundle_id,
        "conference_id": request.conference_id,
        "recording_session_id": request.recording_session_id,
        "join_url": request.join_url,
        "signal_kind": candidate.signal_kind,
        "title": candidate.title,
        "promotion_policy": candidate.promotion_policy,
        "evidence": candidate.evidence,
    })
}

fn radar_signal_review_item(
    request: &YandexTelemostRecordingBridgeRequest,
    bundle_id: &str,
    candidate: &crate::workflows::realtime_conversation_radar_projection::RealtimeConversationRadarSignalCandidate,
) -> NewReviewItem {
    let review_kind = match candidate.signal_kind.as_str() {
        "unknown_cohosts" => ReviewItemKind::PotentialRelationship,
        "unmatched_meeting_link" => ReviewItemKind::PotentialProject,
        _ => ReviewItemKind::KnowledgeCandidate,
    };
    let review_title = match candidate.signal_kind.as_str() {
        "unmatched_meeting_link" => telemost_project_review_title(request, bundle_id),
        _ => candidate.title.clone(),
    };
    let mut metadata = json!({
        "mirrored_from": "realtime_conversation_radar_signal",
        "provider_kind": YANDEX_TELEMOST_PROVIDER_KIND_STR,
        "bundle_id": bundle_id,
        "recording_session_id": request.recording_session_id,
        "conference_id": request.conference_id,
        "signal_kind": candidate.signal_kind,
        "promotion_policy": candidate.promotion_policy,
        "join_url": request.join_url,
    });
    if review_kind == ReviewItemKind::PotentialRelationship
        && let Some(object) = metadata.as_object_mut()
    {
        object.insert(
            "relationship_type".to_owned(),
            Value::String("conference_cohost".to_owned()),
        );
    }
    if review_kind == ReviewItemKind::PotentialProject
        && let Some(object) = metadata.as_object_mut()
    {
        object.insert(
            "candidate_group".to_owned(),
            Value::String("meeting_context".to_owned()),
        );
        object.insert(
            "project_title".to_owned(),
            Value::String(review_title.clone()),
        );
    }

    NewReviewItem::new(
        review_kind,
        review_title,
        radar_signal_review_summary(candidate),
        candidate.confidence as f64,
    )
    .metadata(metadata)
}

fn telemost_project_review_title(
    request: &YandexTelemostRecordingBridgeRequest,
    bundle_id: &str,
) -> String {
    if let Some(conference_id) = request
        .conference_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return format!("Telemost conference {conference_id}");
    }
    format!("Telemost meeting context {bundle_id}")
}

fn radar_signal_review_summary(
    candidate: &crate::workflows::realtime_conversation_radar_projection::RealtimeConversationRadarSignalCandidate,
) -> String {
    match candidate.signal_kind.as_str() {
        "unmatched_meeting_link" => {
            "Telemost runtime captured a meeting link that is not yet bound to calendar, project, or organization context.".to_owned()
        }
        "live_stream_reference" => {
            "Telemost runtime detected a live stream reference that requires owner review before promotion.".to_owned()
        }
        "unknown_cohosts" => {
            "Telemost runtime detected cohosts without confirmed persona mapping; review is required before relationship promotion.".to_owned()
        }
        "local_recording_artifact" => {
            "Telemost runtime produced a local recording artifact bundle that is available for reviewed promotion into memory or documents.".to_owned()
        }
        _ => format!(
            "Realtime conversation radar candidate `{}` requires owner review before promotion.",
            candidate.signal_kind
        ),
    }
}

fn canonical_existing_dir(name: &str, value: &str) -> Result<PathBuf, YandexTelemostError> {
    let required = require_non_empty_path(name, value)?;
    let path = FsPath::new(&required);
    if !path.exists() {
        return Err(YandexTelemostError::InvalidRequest(format!(
            "{name} `{}` does not exist",
            path.display()
        )));
    }
    if !path.is_dir() {
        return Err(YandexTelemostError::InvalidRequest(format!(
            "{name} `{}` must be a directory",
            path.display()
        )));
    }
    Ok(path.canonicalize()?)
}

fn canonical_existing_file(
    name: &str,
    value: &str,
    expected_root: &FsPath,
) -> Result<PathBuf, YandexTelemostError> {
    let required = require_non_empty_path(name, value)?;
    let path = FsPath::new(&required);
    if !path.exists() {
        return Err(YandexTelemostError::InvalidRequest(format!(
            "{name} `{}` does not exist",
            path.display()
        )));
    }
    if !path.is_file() {
        return Err(YandexTelemostError::InvalidRequest(format!(
            "{name} `{}` must be a file",
            path.display()
        )));
    }
    let canonical = path.canonicalize()?;
    if !canonical.starts_with(expected_root) {
        return Err(YandexTelemostError::InvalidRequest(format!(
            "{name} `{}` must stay under output_dir `{}`",
            canonical.display(),
            expected_root.display()
        )));
    }
    Ok(canonical)
}

fn yandex_telemost_event_store_access_error(error: ApiError) -> YandexTelemostError {
    match error {
        ApiError::Store(error) => YandexTelemostError::EventStore(error),
        ApiError::DatabaseNotConfigured => YandexTelemostError::InvalidRequest(
            "database not configured for Yandex Telemost runtime bridge".to_owned(),
        ),
        other => YandexTelemostError::InvalidRequest(format!(
            "failed to access Yandex Telemost event store: {}",
            api_error_code(&other)
        )),
    }
}

fn api_error_code(error: &ApiError) -> &'static str {
    match error {
        ApiError::DatabaseNotConfigured => "database_not_configured",
        ApiError::InvalidEnvelope(_) => "invalid_envelope",
        ApiError::Audit(_) => "audit_error",
        ApiError::Store(_) => "event_store_error",
        _ => "unexpected_api_error",
    }
}

fn require_non_empty_path(name: &str, value: &str) -> Result<String, YandexTelemostError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(YandexTelemostError::InvalidRequest(format!(
            "{name} is required"
        )));
    }
    Ok(trimmed.to_owned())
}

fn write_jsonl_lines(path: &FsPath, lines: &[Value]) -> Result<(), YandexTelemostError> {
    let payload = lines
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    fs::write(path, format!("{payload}\n"))?;
    Ok(())
}

fn read_speaker_timeline_hints(
    path: &FsPath,
) -> Result<Vec<SpeakerTimelineHint>, YandexTelemostError> {
    let content = fs::read_to_string(path)?;
    let mut hints = Vec::new();
    for line in content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let value: Value = serde_json::from_str(line)?;
        let observed_at_ms = value
            .get("observed_at_epoch_ms")
            .and_then(Value::as_i64)
            .ok_or_else(|| {
                YandexTelemostError::InvalidRequest(
                    "speaker timeline hint is missing observed_at_epoch_ms".to_owned(),
                )
            })?;
        let speaker_label = value
            .get("speaker_label")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|label| !label.is_empty())
            .ok_or_else(|| {
                YandexTelemostError::InvalidRequest(
                    "speaker timeline hint is missing speaker_label".to_owned(),
                )
            })?
            .to_owned();
        hints.push(SpeakerTimelineHint {
            observed_at_ms,
            speaker_label,
            source: value
                .get("source")
                .and_then(Value::as_str)
                .unwrap_or("webview_dom_heuristic")
                .to_owned(),
            confidence: value
                .get("confidence")
                .and_then(Value::as_f64)
                .unwrap_or(0.35) as f32,
            truth_status: value
                .get("truth_status")
                .and_then(Value::as_str)
                .unwrap_or("hint_not_truth")
                .to_owned(),
            provider_participant_id: value
                .get("provider_participant_id")
                .and_then(Value::as_str)
                .map(str::to_owned),
        });
    }
    Ok(hints)
}

fn require_yandex_telemost_unlocked_host_vault(state: &AppState) -> Result<(), ApiError> {
    let vault_status = state.vault.status()?;
    if vault_status.state != VaultMode::Unlocked {
        return Err(ApiError::SecretVaultNotConfigured);
    }
    Ok(())
}

fn telemost_window_label(account_id: &str, conference_id: Option<&str>) -> String {
    let stable = format!(
        "{}-{}",
        account_id.trim(),
        conference_id.unwrap_or("meeting")
    )
    .chars()
    .map(|ch| {
        if ch.is_ascii_alphanumeric() {
            ch.to_ascii_lowercase()
        } else {
            '-'
        }
    })
    .collect::<String>()
    .split('-')
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join("-");
    format!("yandex-telemost-{stable}")
}

fn recording_policy_manifest() -> YandexTelemostLocalRecordingManifest {
    YandexTelemostLocalRecordingManifest {
        state: "implemented_as_tauri_local_ffmpeg_controller",
        audio_format: "mp3",
        recorder_boundary: "local_desktop_only_no_backend_secret_access",
        consent_required: true,
        default_output_policy: "app_data_dir/telemost-recordings/{account_id}/{recording_session_id}",
        audio_device_policy: YandexTelemostLocalRecordingPolicy {
            macos: "use explicit loopback input such as BlackHole 2ch; Hermes does not install kernel audio drivers silently",
            linux: "prepare command can create a PulseAudio/PipeWire null sink named hermes_telemost and record hermes_telemost.monitor",
            windows: "use WASAPI loopback or an explicitly configured virtual input",
            ffmpeg_path_env: "HERMES_TELEMOST_FFMPEG_PATH",
            ffmpeg_input_env: "HERMES_TELEMOST_FFMPEG_INPUT",
        },
    }
}

fn speaker_timeline_policy() -> YandexTelemostSpeakerTimelinePolicy {
    YandexTelemostSpeakerTimelinePolicy {
        state: "implemented_as_webview_dom_heuristic_jsonl_and_txt",
        source: "visible_webview_active_speaker_dom_observation",
        reliability: "hint_not_source_of_truth",
        output_files: vec!["speaker-timeline.jsonl", "speaker-timeline.txt"],
        role_in_transcription: "warm-start diarization/person-count hints for Whisper-side processing",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn materialize_yandex_telemost_call_bundle_writes_manifest_and_bridge_files() {
        let temp_root =
            std::env::temp_dir().join(format!("telemost-bundle-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_root).expect("create temp root");
        let audio_path = temp_root.join("audio.mp3");
        let speaker_jsonl_path = temp_root.join("speaker-timeline.jsonl");
        let speaker_txt_path = temp_root.join("speaker-timeline.txt");
        fs::write(&audio_path, b"mp3").expect("write audio");
        fs::write(
            &speaker_jsonl_path,
            "{\"observed_at_epoch_ms\":1000,\"speaker_label\":\"Owner\",\"confidence\":0.35,\"source\":\"webview_dom_heuristic\",\"truth_status\":\"hint_not_truth\"}\n",
        )
        .expect("write jsonl");
        fs::write(&speaker_txt_path, "0\tOwner\tspeaker_hint\n").expect("write txt");

        let request = YandexTelemostRecordingBridgeRequest {
            account_id: "telemost-main".to_owned(),
            conference_id: Some("conf-1".to_owned()),
            join_url: "https://telemost.yandex.ru/j/conf-1".to_owned(),
            recording_session_id: "session-1".to_owned(),
            output_dir: temp_root.to_string_lossy().into_owned(),
            audio_path: audio_path.to_string_lossy().into_owned(),
            speaker_jsonl_path: speaker_jsonl_path.to_string_lossy().into_owned(),
            speaker_txt_path: speaker_txt_path.to_string_lossy().into_owned(),
            started_at_epoch_ms: 1000,
            stopped_at_epoch_ms: 2000,
            consent_attested: true,
        };

        let materialized = materialize_yandex_telemost_call_bundle(
            &request,
            None,
            json!({ "local_recording": {} }),
        )
        .expect("materialize bundle");

        assert!(materialized.manifest_path.exists());
        assert!(temp_root.join("meeting.json").exists());
        assert!(temp_root.join("provider.json").exists());
        assert!(temp_root.join("participants.json").exists());
        assert!(temp_root.join("speaker-hints.jsonl").exists());
        assert!(temp_root.join("event-track.jsonl").exists());
        assert_eq!(materialized.manifest.bundle_id, "session-1");
        assert!(
            materialized
                .pipeline_plan
                .follow_up_events
                .contains(&REALTIME_CONVERSATION_TRANSCRIPT_REQUESTED.to_owned())
        );
        assert!(!materialized.radar_candidates.is_empty());

        fs::remove_dir_all(&temp_root).expect("cleanup temp root");
    }

    #[test]
    fn materialize_yandex_telemost_call_bundle_skips_unmatched_link_when_calendar_bound() {
        let temp_root =
            std::env::temp_dir().join(format!("telemost-bundle-calendar-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&temp_root).expect("create temp root");
        let audio_path = temp_root.join("audio.mp3");
        let speaker_jsonl_path = temp_root.join("speaker-timeline.jsonl");
        let speaker_txt_path = temp_root.join("speaker-timeline.txt");
        fs::write(&audio_path, b"mp3").expect("write audio");
        fs::write(
            &speaker_jsonl_path,
            "{\"observed_at_epoch_ms\":1000,\"speaker_label\":\"Owner\",\"confidence\":0.35,\"source\":\"webview_dom_heuristic\",\"truth_status\":\"hint_not_truth\"}\n",
        )
        .expect("write jsonl");
        fs::write(&speaker_txt_path, "0\tOwner\tspeaker_hint\n").expect("write txt");

        let request = YandexTelemostRecordingBridgeRequest {
            account_id: "telemost-main".to_owned(),
            conference_id: Some("conf-1".to_owned()),
            join_url: "https://telemost.yandex.ru/j/conf-1".to_owned(),
            recording_session_id: "session-1".to_owned(),
            output_dir: temp_root.to_string_lossy().into_owned(),
            audio_path: audio_path.to_string_lossy().into_owned(),
            speaker_jsonl_path: speaker_jsonl_path.to_string_lossy().into_owned(),
            speaker_txt_path: speaker_txt_path.to_string_lossy().into_owned(),
            started_at_epoch_ms: 1000,
            stopped_at_epoch_ms: 2000,
            consent_attested: true,
        };

        let materialized = materialize_yandex_telemost_call_bundle(
            &request,
            Some("evt:v1:calendar-bound".to_owned()),
            json!({ "local_recording": {} }),
        )
        .expect("materialize bundle");

        assert_eq!(
            materialized.manifest.calendar_event_id.as_deref(),
            Some("evt:v1:calendar-bound")
        );
        assert!(
            !materialized
                .radar_candidates
                .iter()
                .any(|candidate| candidate.signal_kind == "unmatched_meeting_link")
        );

        fs::remove_dir_all(&temp_root).expect("cleanup temp root");
    }

    #[test]
    fn telemost_window_label_is_stable() {
        assert_eq!(
            telemost_window_label("Main Account", Some("Room/42")),
            "yandex-telemost-main-account-room-42"
        );
    }

    #[test]
    fn cohost_import_keeps_public_shape_reachable() {
        let cohost = TelemostCohost {
            email: "cohost@yandex.ru".to_owned(),
        };
        assert_eq!(cohost.email, "cohost@yandex.ru");
    }

    #[test]
    fn conference_request_shape_stays_available_for_routes() {
        let request = YandexTelemostConferenceRequest {
            waiting_room_level: None,
            live_stream: None,
            cohosts: vec![],
            is_auto_summarization_enabled: Some(true),
            metadata: json!({}),
        };
        assert_eq!(request.is_auto_summarization_enabled, Some(true));
    }

    #[test]
    fn radar_signal_review_summary_matches_current_signal_contracts() {
        let candidate =
            crate::workflows::realtime_conversation_radar_projection::RealtimeConversationRadarSignalCandidate {
                signal_kind: "unknown_cohosts".to_owned(),
                title: "Review Telemost cohosts without confirmed persona mapping".to_owned(),
                confidence: 0.68,
                evidence: json!({}),
                promotion_policy:
                    "radar_review_required_before_relationship_or_persona_link".to_owned(),
            };

        let summary = radar_signal_review_summary(&candidate);

        assert!(summary.contains("cohosts"));
        assert!(summary.contains("review"));
    }

    #[test]
    fn unknown_cohosts_review_item_uses_relationship_flow() {
        let request = YandexTelemostRecordingBridgeRequest {
            account_id: "telemost-main".to_owned(),
            conference_id: Some("conf-1".to_owned()),
            join_url: "https://telemost.yandex.ru/j/conf-1".to_owned(),
            recording_session_id: "session-1".to_owned(),
            output_dir: "/tmp/telemost-output".to_owned(),
            audio_path: "/tmp/telemost-output/audio.mp3".to_owned(),
            speaker_jsonl_path: "/tmp/telemost-output/speaker-timeline.jsonl".to_owned(),
            speaker_txt_path: "/tmp/telemost-output/speaker-timeline.txt".to_owned(),
            started_at_epoch_ms: 1000,
            stopped_at_epoch_ms: 2000,
            consent_attested: true,
        };
        let candidate =
            crate::workflows::realtime_conversation_radar_projection::RealtimeConversationRadarSignalCandidate {
                signal_kind: "unknown_cohosts".to_owned(),
                title: "Review Telemost cohosts without confirmed persona mapping".to_owned(),
                confidence: 0.68,
                evidence: json!({}),
                promotion_policy:
                    "radar_review_required_before_relationship_or_persona_link".to_owned(),
            };

        let review_item = radar_signal_review_item(&request, "bundle-1", &candidate);

        assert_eq!(review_item.item_kind, ReviewItemKind::PotentialRelationship);
        assert_eq!(
            review_item
                .metadata
                .get("relationship_type")
                .and_then(Value::as_str),
            Some("conference_cohost")
        );
    }

    #[test]
    fn unmatched_meeting_link_review_item_uses_project_flow() {
        let request = YandexTelemostRecordingBridgeRequest {
            account_id: "telemost-main".to_owned(),
            conference_id: Some("conf-42".to_owned()),
            join_url: "https://telemost.yandex.ru/j/conf-42".to_owned(),
            recording_session_id: "session-1".to_owned(),
            output_dir: "/tmp/telemost-output".to_owned(),
            audio_path: "/tmp/telemost-output/audio.mp3".to_owned(),
            speaker_jsonl_path: "/tmp/telemost-output/speaker-timeline.jsonl".to_owned(),
            speaker_txt_path: "/tmp/telemost-output/speaker-timeline.txt".to_owned(),
            started_at_epoch_ms: 1000,
            stopped_at_epoch_ms: 2000,
            consent_attested: true,
        };
        let candidate =
            crate::workflows::realtime_conversation_radar_projection::RealtimeConversationRadarSignalCandidate {
                signal_kind: "unmatched_meeting_link".to_owned(),
                title: "Review unmatched Telemost meeting link".to_owned(),
                confidence: 0.72,
                evidence: json!({}),
                promotion_policy:
                    "radar_review_required_before_calendar_or_project_link".to_owned(),
            };

        let review_item = radar_signal_review_item(&request, "bundle-1", &candidate);

        assert_eq!(review_item.item_kind, ReviewItemKind::PotentialProject);
        assert_eq!(review_item.title, "Telemost conference conf-42");
        assert_eq!(
            review_item
                .metadata
                .get("project_title")
                .and_then(Value::as_str),
            Some("Telemost conference conf-42")
        );
    }
}
