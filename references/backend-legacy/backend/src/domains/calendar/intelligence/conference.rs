use super::CalendarIntelligenceService;

impl CalendarIntelligenceService {
    pub fn detect_conference_provider(url: &str) -> Option<String> {
        let url = url.to_lowercase();
        if url.contains("meet.google.com") {
            return Some("google_meet".into());
        }
        if url.contains("zoom.us") || url.contains("zoom.com") {
            return Some("zoom".into());
        }
        if url.contains("teams.microsoft.com") || url.contains("teams.live.com") {
            return Some("microsoft_teams".into());
        }
        if url.contains("meet.jit.si") {
            return Some("jitsi".into());
        }
        if url.contains("webex.com") {
            return Some("webex".into());
        }
        None
    }

    pub fn extract_conference_url(text: &str) -> Option<String> {
        let patterns = [
            "https://meet.google.com/",
            "https://zoom.us/j/",
            "https://teams.microsoft.com/l/meetup-join/",
            "https://meet.jit.si/",
        ];
        let lower = text.to_lowercase();
        for prefix in &patterns {
            if let Some(pos) = lower.find(prefix) {
                let end = lower[pos..]
                    .find(|character: char| character.is_whitespace())
                    .unwrap_or(lower[pos..].len());
                return Some(text[pos..pos + end].to_string());
            }
        }
        None
    }
}
