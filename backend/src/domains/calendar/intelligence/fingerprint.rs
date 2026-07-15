use super::CalendarIntelligenceService;
use super::classification::contains_any;
use super::models::EventFingerprint;

impl CalendarIntelligenceService {
    pub fn heuristic_fingerprint(
        title: &str,
        description: Option<&str>,
        event_type: &str,
    ) -> EventFingerprint {
        let combined = format!("{} {}", title, description.unwrap_or(""));
        let lower = combined.to_lowercase();
        let mut fingerprint = EventFingerprint {
            event_type: if event_type.trim().is_empty() {
                CalendarIntelligenceService::classify_event(title, 1, 60)
            } else {
                event_type.to_owned()
            },
            ..Default::default()
        };

        fingerprint.importance = fingerprint_importance(&lower);
        fingerprint.language = if contains_any(&lower, &["испанск", "espanol"]) {
            Some("es".into())
        } else {
            Some("en".into())
        };
        fingerprint.recurrence_hint = recurrence_hint(&lower);
        fingerprint
    }
}

fn fingerprint_importance(value: &str) -> f64 {
    if contains_any(value, &["important", "critical", "важно"]) {
        0.8
    } else if contains_any(value, &["client", "tax", "legal"]) {
        0.7
    } else {
        0.4
    }
}

fn recurrence_hint(value: &str) -> Option<String> {
    if contains_any(value, &["weekly", "еженедел"]) {
        Some("weekly".into())
    } else if contains_any(value, &["daily", "ежеднев"]) {
        Some("daily".into())
    } else {
        None
    }
}
