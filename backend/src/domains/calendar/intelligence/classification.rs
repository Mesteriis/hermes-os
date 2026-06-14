use super::CalendarIntelligenceService;

impl CalendarIntelligenceService {
    pub fn classify_event(title: &str, participants_count: usize, duration_minutes: i64) -> String {
        let title = title.to_lowercase();
        if contains_any(&title, &["meeting", "call", "sync", "созвон", "встреча"]) {
            return "meeting".into();
        }
        if contains_any(&title, &["deadline", "due", "срок", "дедлайн"]) {
            return "deadline".into();
        }
        if contains_any(&title, &["focus", "deep work", "фокус"]) {
            return "focus".into();
        }
        if contains_any(&title, &["travel", "flight", "поездка", "перелёт"]) {
            return "travel".into();
        }
        if contains_any(&title, &["review", "обзор", "ревью"]) {
            return "review".into();
        }
        if contains_any(&title, &["planning", "план"]) {
            return "planning".into();
        }
        if contains_any(&title, &["tax", "налог", "aeat", "declaracion"]) {
            return "tax".into();
        }
        if contains_any(&title, &["legal", "abogado", "lawyer"]) {
            return "legal".into();
        }
        if contains_any(&title, &["finance", "invoice", "счёт", "фактура"]) {
            return "finance".into();
        }
        if contains_any(&title, &["birthday", "день рождения"]) {
            return "birthday".into();
        }
        if contains_any(&title, &["reminder", "напомин"]) {
            return "reminder".into();
        }
        if participants_count > 2 || duration_minutes > 120 {
            return "meeting".into();
        }
        "personal".into()
    }
}

pub(super) fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}
