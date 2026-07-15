use crate::app::error::types::ApiError;
use crate::integrations::whatsapp::client::errors::WhatsappWebError;
use crate::platform::events::bus::whatsapp_event_types;
use chrono::{DateTime, Utc};
use std::sync::atomic::{AtomicU64, Ordering};

static EVENT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub(super) fn event_id(scope: &str, subject: &str, now: DateTime<Utc>) -> String {
    let seq = EVENT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!(
        "evt_whatsapp_{}_{}_{}_{}",
        scope,
        subject.replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
        now.timestamp_nanos_opt().unwrap_or_default(),
        seq
    )
}

pub(super) fn runtime_bridge_media(direction: &str, phase: &str) -> Result<&'static str, ApiError> {
    match (direction, phase) {
        ("upload", "requested") => Ok(whatsapp_event_types::MEDIA_UPLOAD_REQUESTED),
        ("upload", "started") => Ok("whatsapp.media.upload.started"),
        ("upload", "progress") => Ok("whatsapp.media.upload.progress"),
        ("upload", "completed") => Ok("whatsapp.media.upload.completed"),
        ("upload", "failed") => Ok(whatsapp_event_types::MEDIA_UPLOAD_FAILED),
        ("download", "requested") => Ok(whatsapp_event_types::MEDIA_DOWNLOAD_REQUESTED),
        ("download", "started") => Ok("whatsapp.media.download.started"),
        ("download", "progress") => Ok("whatsapp.media.download.progress"),
        ("download", "completed") => Ok("whatsapp.media.download.completed"),
        ("download", "failed") => Ok(whatsapp_event_types::MEDIA_DOWNLOAD_FAILED),
        _ => Err(WhatsappWebError::InvalidRequest(format!(
            "unsupported runtime bridge media lifecycle `{direction}.{phase}`"
        ))
        .into()),
    }
}
