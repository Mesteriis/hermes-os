use crate::application::provider_runtime_contracts::{
    YandexTelemostError, YandexTelemostTranscriptBridgeRequest,
    YandexTelemostTranscriptBridgeResponse,
};
use crate::platform::events::{EventBus, EventStore};

pub(crate) use crate::workflows::realtime_conversation_transcript_execution::*;

pub(crate) async fn complete_realtime_conversation_transcript_bridge(
    event_store: &EventStore,
    event_bus: Option<&EventBus>,
    request: &YandexTelemostTranscriptBridgeRequest,
) -> Result<YandexTelemostTranscriptBridgeResponse, YandexTelemostError> {
    crate::integrations::yandex_telemost::runtime_bridge::complete_yandex_telemost_transcript_bridge(
        event_store,
        event_bus,
        request,
    )
    .await
}
