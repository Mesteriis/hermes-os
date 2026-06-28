use axum::Json;
use axum::extract::{Path, Query, State};
use serde_json::json;

use crate::app::api_support::{
    CallApiRequest, CallListResponse, CallTranscriptFixtureApiRequest, CallTranscriptResponse,
    TelegramListQuery, call_intelligence_store,
};
use crate::app::{ApiError, AppState};
use crate::platform::calls::{
    CallTranscript, FixtureSpeechToTextProvider, NewCallTranscript, ProviderCall,
    SpeechToTextProvider, TranscriptStatus,
};

pub(crate) async fn post_call(
    State(state): State<AppState>,
    Json(request): Json<CallApiRequest>,
) -> Result<Json<ProviderCall>, ApiError> {
    Ok(Json(
        call_intelligence_store(&state)?
            .upsert_call(&request.into_call())
            .await?,
    ))
}

pub(crate) async fn get_calls(
    State(state): State<AppState>,
    Query(query): Query<TelegramListQuery>,
) -> Result<Json<CallListResponse>, ApiError> {
    let items = call_intelligence_store(&state)?
        .list_calls(
            query.account_id.as_deref(),
            query.provider_chat_id.as_deref(),
            query.provider.as_deref(),
            query.limit.unwrap_or(50),
        )
        .await?;

    Ok(Json(CallListResponse { items }))
}

pub(crate) async fn post_call_transcript_fixture(
    State(state): State<AppState>,
    Path(call_id): Path<String>,
    Json(request): Json<CallTranscriptFixtureApiRequest>,
) -> Result<Json<CallTranscript>, ApiError> {
    let stt = FixtureSpeechToTextProvider;
    let fixture = stt.transcribe_fixture(&request.source_audio_ref)?;
    let transcript = NewCallTranscript {
        transcript_id: request.transcript_id,
        call_id,
        account_id: request.account_id,
        provider_chat_id: request.provider_chat_id,
        transcript_status: TranscriptStatus::Succeeded,
        stt_provider: stt.provider_name().to_owned(),
        source_audio_ref: Some(request.source_audio_ref),
        language_code: request.language_code,
        transcript_text: fixture.text,
        segments: fixture.segments,
        provenance: json!({
            "runtime": "fixture",
            "source": "local_call_audio",
            "always_on_policy": request.always_on_policy,
        }),
    };

    Ok(Json(
        call_intelligence_store(&state)?
            .upsert_transcript(&transcript)
            .await?,
    ))
}

pub(crate) async fn get_call_transcript(
    State(state): State<AppState>,
    Path(call_id): Path<String>,
) -> Result<Json<CallTranscriptResponse>, ApiError> {
    let transcript = call_intelligence_store(&state)?
        .transcript_for_call(&call_id)
        .await?;

    Ok(Json(CallTranscriptResponse { transcript }))
}
