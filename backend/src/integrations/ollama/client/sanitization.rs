pub(in crate::integrations::ollama::client) fn strip_thinking_content(content: &str) -> String {
    let mut sanitized = content.trim().to_owned();
    while let Some(start) = sanitized.find("<think>") {
        let Some(end_offset) = sanitized[start..].find("</think>") else {
            sanitized.replace_range(start.., "");
            break;
        };
        let end = start + end_offset + "</think>".len();
        sanitized.replace_range(start..end, "");
    }

    if let Some(end) = sanitized.rfind("</think>") {
        sanitized = sanitized[end + "</think>".len()..].to_owned();
    }

    sanitized.trim().to_owned()
}
