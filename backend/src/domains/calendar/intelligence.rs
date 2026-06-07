use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventAnalysis {
    pub event_type: String,
    pub importance_score: f64,
    pub readiness_score: f64,
    pub risks: Vec<String>,
}

pub struct CalendarIntelligenceService;

impl CalendarIntelligenceService {
    pub fn classify_event(title: &str, participants_count: usize, duration_minutes: i64) -> String {
        let t = title.to_lowercase();
        if t.contains("meeting")
            || t.contains("call")
            || t.contains("sync")
            || t.contains("созвон")
            || t.contains("встреча")
        {
            return "meeting".into();
        }
        if t.contains("deadline")
            || t.contains("due")
            || t.contains("срок")
            || t.contains("дедлайн")
        {
            return "deadline".into();
        }
        if t.contains("focus") || t.contains("deep work") || t.contains("фокус") {
            return "focus".into();
        }
        if t.contains("travel")
            || t.contains("flight")
            || t.contains("поездка")
            || t.contains("перелёт")
        {
            return "travel".into();
        }
        if t.contains("review")
            || t.contains("review")
            || t.contains("обзор")
            || t.contains("ревью")
        {
            return "review".into();
        }
        if t.contains("planning") || t.contains("план") {
            return "planning".into();
        }
        if t.contains("tax")
            || t.contains("налог")
            || t.contains("aeat")
            || t.contains("declaracion")
        {
            return "tax".into();
        }
        if t.contains("legal") || t.contains("abogado") || t.contains("lawyer") {
            return "legal".into();
        }
        if t.contains("finance")
            || t.contains("invoice")
            || t.contains("счёт")
            || t.contains("фактура")
        {
            return "finance".into();
        }
        if t.contains("birthday") || t.contains("день рождения") {
            return "birthday".into();
        }
        if t.contains("reminder") || t.contains("напомин") {
            return "reminder".into();
        }
        if participants_count > 2 {
            return "meeting".into();
        }
        if duration_minutes > 120 {
            return "meeting".into();
        }
        "personal".into()
    }

    pub fn calculate_importance(
        title: &str,
        participants_count: usize,
        has_project: bool,
        has_deadline: bool,
    ) -> f64 {
        let mut score: f64 = 0.3;
        let t = title.to_lowercase();
        if t.contains("urgent") || t.contains("asap") || t.contains("срочно") {
            score += 0.3;
        }
        if t.contains("client") || t.contains("клиент") || t.contains("customer") {
            score += 0.2;
        }
        if t.contains("tax") || t.contains("legal") || t.contains("aeat") {
            score += 0.2;
        }
        if participants_count > 2 {
            score += 0.1;
        }
        if has_project {
            score += 0.1;
        }
        if has_deadline {
            score += 0.1;
        }
        score.clamp(0.0, 1.0)
    }

    pub fn calculate_readiness(
        has_agenda: bool,
        has_docs: bool,
        has_context: bool,
        has_checklist: bool,
        has_participants: bool,
    ) -> f64 {
        let mut score: f64 = 0.0;
        if has_agenda {
            score += 0.25;
        }
        if has_docs {
            score += 0.2;
        }
        if has_context {
            score += 0.2;
        }
        if has_checklist {
            score += 0.15;
        }
        if has_participants {
            score += 0.2;
        }
        score.clamp(0.0, 1.0)
    }

    pub fn detect_risks(
        has_agenda: bool,
        has_docs: bool,
        has_participants: bool,
        has_project: bool,
        is_upcoming_soon: bool,
    ) -> Vec<String> {
        let mut risks = Vec::new();
        if !has_agenda {
            risks.push("No agenda prepared".into());
        }
        if !has_docs {
            risks.push("No documents attached".into());
        }
        if !has_participants {
            risks.push("No participants resolved".into());
        }
        if !has_project {
            risks.push("Not linked to a project".into());
        }
        if is_upcoming_soon && (!has_agenda || !has_docs) {
            risks.push("Event is soon but preparation incomplete".into());
        }
        risks
    }

    /// Heuristic event fingerprint (same pattern as PersonIntelligenceService)
    pub fn heuristic_fingerprint(
        title: &str,
        description: Option<&str>,
        event_type: &str,
    ) -> EventFingerprint {
        let combined = format!("{} {}", title, description.unwrap_or(""));
        let lower = combined.to_lowercase();
        let mut fp = EventFingerprint {
            event_type: if event_type.trim().is_empty() {
                CalendarIntelligenceService::classify_event(title, 1, 60)
            } else {
                event_type.to_owned()
            },
            ..Default::default()
        };

        if lower.contains("important") || lower.contains("critical") || lower.contains("важно")
        {
            fp.importance = 0.8;
        } else if lower.contains("client") || lower.contains("tax") || lower.contains("legal") {
            fp.importance = 0.7;
        } else {
            fp.importance = 0.4;
        }

        fp.language = if lower.contains("испанск") || lower.contains("espanol") {
            Some("es".into())
        } else {
            Some("en".into())
        };

        if lower.contains("weekly") || lower.contains("еженедел") {
            fp.recurrence_hint = Some("weekly".into());
        }
        if lower.contains("daily") || lower.contains("ежеднев") {
            fp.recurrence_hint = Some("daily".into());
        }

        fp
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EventFingerprint {
    pub event_type: String,
    pub importance: f64,
    pub language: Option<String>,
    pub recurrence_hint: Option<String>,
    pub topics: Vec<String>,
}

#[derive(Debug, Error)]
pub enum CalendarIntelligenceError {
    #[error("analysis failed: {0}")]
    AnalysisFailed(String),
}

// ── Conference Link Detection ──────────────────────────────────────────────

impl CalendarIntelligenceService {
    pub fn detect_conference_provider(url: &str) -> Option<String> {
        let u = url.to_lowercase();
        if u.contains("meet.google.com") {
            return Some("google_meet".into());
        }
        if u.contains("zoom.us") || u.contains("zoom.com") {
            return Some("zoom".into());
        }
        if u.contains("teams.microsoft.com") || u.contains("teams.live.com") {
            return Some("microsoft_teams".into());
        }
        if u.contains("meet.jit.si") {
            return Some("jitsi".into());
        }
        if u.contains("webex.com") {
            return Some("webex".into());
        }
        None
    }

    pub fn extract_conference_url(text: &str) -> Option<String> {
        // Simple URL extraction for common meeting link patterns
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
                    .find(|c: char| c.is_whitespace())
                    .unwrap_or(lower[pos..].len());
                return Some(text[pos..pos + end].to_string());
            }
        }
        None
    }
}

// ── Location Intelligence ─────────────────────────────────────────────────

impl CalendarIntelligenceService {
    pub fn parse_location(location: &str) -> LocationInfo {
        let lower = location.to_lowercase();

        // Detect online
        let is_online = lower.contains("online")
            || lower.contains("virtual")
            || lower.contains("zoom")
            || lower.contains("meet.google")
            || lower.contains("teams.microsoft")
            || lower.contains("video call")
            || lower.contains("видеозвонок");

        // Detect common locations
        let parsed_name = if lower.contains("office") || lower.contains("офис") {
            Some("Office".into())
        } else if lower.contains("home") || lower.contains("дома") {
            Some("Home".into())
        } else if lower.contains("cafe") || lower.contains("coffee") || lower.contains("кафе") {
            Some("Cafe".into())
        } else if lower.contains("airport") || lower.contains("аэропорт") {
            Some("Airport".into())
        } else if lower.contains("hotel") || lower.contains("отель") {
            Some("Hotel".into())
        } else if !is_online && !location.is_empty() {
            Some(location.to_string())
        } else {
            None
        };

        // Estimate travel time for offline events (rough heuristic)
        let travel_buffer = if is_online { None } else { Some(15i32) }; // 15 min default

        LocationInfo {
            is_online,
            parsed_name,
            travel_buffer_minutes: travel_buffer,
        }
    }

    /// Time distribution: categorize events for analytics
    pub fn categorize_time(event_type: &str, title: &str) -> String {
        let t = title.to_lowercase();
        match event_type {
            "meeting" => "meetings".into(),
            "focus" => "focus".into(),
            "deadline" => "deadlines".into(),
            "travel" => "travel".into(),
            "tax" | "legal" | "government" => "admin".into(),
            "finance" => "finance".into(),
            "personal" | "birthday" => "personal".into(),
            "review" | "planning" => "planning".into(),
            _ => {
                if t.contains("meeting") || t.contains("call") {
                    "meetings".into()
                } else if t.contains("focus") {
                    "focus".into()
                } else if t.contains("lunch") || t.contains("dinner") || t.contains("coffee") {
                    "personal".into()
                } else {
                    "other".into()
                }
            }
        }
    }

    /// Meeting load: detect back-to-back meetings
    pub fn detect_back_to_back(
        events: &[(DateTime<Utc>, DateTime<Utc>, String)],
    ) -> Vec<BackToBackGroup> {
        let mut sorted: Vec<_> = events.to_vec();
        sorted.sort_by_key(|(s, _, _)| *s);
        let mut groups = Vec::new();
        let mut current: Vec<String> = Vec::new();

        for window in sorted.windows(2) {
            let (_, e1, t1) = &window[0];
            let (s2, _, t2) = &window[1];
            let gap = (*s2 - *e1).num_minutes();

            if gap <= 5 && current.is_empty() {
                current.push(t1.clone());
                current.push(t2.clone());
            } else if gap <= 5 {
                current.push(t2.clone());
            } else if !current.is_empty() {
                groups.push(BackToBackGroup {
                    titles: current.clone(),
                    count: current.len(),
                });
                current.clear();
            }
        }
        if !current.is_empty() {
            groups.push(BackToBackGroup {
                titles: current.clone(),
                count: current.len(),
            });
        }
        groups
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocationInfo {
    pub is_online: bool,
    pub parsed_name: Option<String>,
    pub travel_buffer_minutes: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackToBackGroup {
    pub titles: Vec<String>,
    pub count: usize,
}
