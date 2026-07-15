pub(crate) fn media_runtime_event_kind(event_type: &str) -> Option<&'static str> {
    match event_type {
        "whatsapp.media.upload.requested" => Some("media.upload.requested"),
        "whatsapp.media.upload.started" => Some("media.upload.started"),
        "whatsapp.media.upload.progress" => Some("media.upload.progress"),
        "whatsapp.media.upload.completed" => Some("media.upload.completed"),
        "whatsapp.media.upload.failed" => Some("media.upload.failed"),
        "whatsapp.media.download.requested" => Some("media.download.requested"),
        "whatsapp.media.download.started" => Some("media.download.started"),
        "whatsapp.media.download.progress" => Some("media.download.progress"),
        "whatsapp.media.download.completed" => Some("media.download.completed"),
        "whatsapp.media.download.failed" => Some("media.download.failed"),
        _ => None,
    }
}

pub(crate) fn media_lifecycle_state(event_type: &str) -> &'static str {
    match event_type {
        "whatsapp.media.upload.requested" | "whatsapp.media.download.requested" => "requested",
        "whatsapp.media.upload.started" | "whatsapp.media.download.started" => "started",
        "whatsapp.media.upload.progress" | "whatsapp.media.download.progress" => "in_progress",
        "whatsapp.media.upload.completed" | "whatsapp.media.download.completed" => "completed",
        "whatsapp.media.upload.failed" | "whatsapp.media.download.failed" => "failed",
        _ => "observed",
    }
}
