use crate::domains::communications::storage::models::StoredCommunicationAttachmentWithBlob;

pub(super) fn allowed_by_scan_status(attachment: &StoredCommunicationAttachmentWithBlob) -> bool {
    attachment.attachment.scan_status.as_str() == "clean"
}

pub(super) fn is_zip(attachment: &StoredCommunicationAttachmentWithBlob) -> bool {
    let content_type = attachment
        .attachment
        .content_type
        .trim()
        .to_ascii_lowercase();
    content_type == "application/zip"
        || attachment
            .attachment
            .filename
            .as_deref()
            .map(|filename| filename.trim().to_ascii_lowercase().ends_with(".zip"))
            .unwrap_or(false)
}

pub(super) fn is_previewable_text(attachment: &StoredCommunicationAttachmentWithBlob) -> bool {
    let content_type = attachment
        .attachment
        .content_type
        .trim()
        .to_ascii_lowercase();
    if content_type.starts_with("text/") {
        return true;
    }
    matches!(
        content_type.as_str(),
        "application/json" | "application/xml" | "application/yaml" | "application/x-yaml"
    ) || attachment
        .attachment
        .filename
        .as_deref()
        .map(|filename| {
            let filename = filename.trim().to_ascii_lowercase();
            [".txt", ".md", ".csv", ".json", ".xml", ".yaml", ".yml"]
                .iter()
                .any(|suffix| filename.ends_with(suffix))
        })
        .unwrap_or(false)
}

pub(super) fn is_derived_text(attachment: &StoredCommunicationAttachmentWithBlob) -> bool {
    matches!(
        crate::platform::communications::attachment_text::rich_attachment_extraction_kind(
            &attachment.attachment.content_type,
            attachment.attachment.filename.as_deref(),
        ),
        Some(
            crate::platform::communications::attachment_text::RichAttachmentExtractionKind::Pdf
                | crate::platform::communications::attachment_text::RichAttachmentExtractionKind::Docx
        )
    )
}

pub(super) fn image_content_type(
    attachment: &StoredCommunicationAttachmentWithBlob,
) -> Option<&'static str> {
    let content_type = attachment
        .attachment
        .content_type
        .trim()
        .to_ascii_lowercase();
    match content_type.as_str() {
        "image/png" => Some("image/png"),
        "image/jpeg" => Some("image/jpeg"),
        "image/gif" => Some("image/gif"),
        "image/webp" => Some("image/webp"),
        _ => attachment
            .attachment
            .filename
            .as_deref()
            .and_then(|filename| {
                let filename = filename.trim().to_ascii_lowercase();
                if filename.ends_with(".png") {
                    Some("image/png")
                } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
                    Some("image/jpeg")
                } else if filename.ends_with(".gif") {
                    Some("image/gif")
                } else if filename.ends_with(".webp") {
                    Some("image/webp")
                } else {
                    None
                }
            }),
    }
}

pub(super) fn audio_content_type(
    attachment: &StoredCommunicationAttachmentWithBlob,
) -> Option<&str> {
    let content_type = attachment.attachment.content_type.trim();
    if content_type.to_ascii_lowercase().starts_with("audio/") && !content_type.is_empty() {
        return Some(content_type);
    }
    attachment
        .attachment
        .filename
        .as_deref()
        .and_then(|filename| {
            let filename = filename.trim().to_ascii_lowercase();
            if filename.ends_with(".mp3") {
                Some("audio/mpeg")
            } else if filename.ends_with(".m4a") || filename.ends_with(".aac") {
                Some("audio/mp4")
            } else if filename.ends_with(".ogg") || filename.ends_with(".opus") {
                Some("audio/ogg")
            } else if filename.ends_with(".wav") {
                Some("audio/wav")
            } else if filename.ends_with(".webm") {
                Some("audio/webm")
            } else {
                None
            }
        })
}

pub(super) fn video_content_type(
    attachment: &StoredCommunicationAttachmentWithBlob,
) -> Option<&str> {
    let content_type = attachment.attachment.content_type.trim();
    if content_type.to_ascii_lowercase().starts_with("video/") && !content_type.is_empty() {
        return Some(content_type);
    }
    attachment
        .attachment
        .filename
        .as_deref()
        .and_then(|filename| {
            let filename = filename.trim().to_ascii_lowercase();
            if filename.ends_with(".mp4") {
                Some("video/mp4")
            } else if filename.ends_with(".webm") {
                Some("video/webm")
            } else if filename.ends_with(".mov") {
                Some("video/quicktime")
            } else {
                None
            }
        })
}
