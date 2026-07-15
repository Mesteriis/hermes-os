use super::CalendarIntelligenceService;
use super::models::LocationInfo;

impl CalendarIntelligenceService {
    pub fn parse_location(location: &str) -> LocationInfo {
        let lower = location.to_lowercase();
        let is_online = is_online_location(&lower);
        let parsed_name = parsed_location_name(location, &lower, is_online);
        let travel_buffer_minutes = if is_online { None } else { Some(15i32) };

        LocationInfo {
            is_online,
            parsed_name,
            travel_buffer_minutes,
        }
    }
}

fn is_online_location(location: &str) -> bool {
    location.contains("online")
        || location.contains("virtual")
        || location.contains("zoom")
        || location.contains("meet.google")
        || location.contains("teams.microsoft")
        || location.contains("video call")
        || location.contains("видеозвонок")
}

fn parsed_location_name(original: &str, lower: &str, is_online: bool) -> Option<String> {
    if lower.contains("office") || lower.contains("офис") {
        Some("Office".into())
    } else if lower.contains("home") || lower.contains("дома") {
        Some("Home".into())
    } else if lower.contains("cafe") || lower.contains("coffee") || lower.contains("кафе") {
        Some("Cafe".into())
    } else if lower.contains("airport") || lower.contains("аэропорт") {
        Some("Airport".into())
    } else if lower.contains("hotel") || lower.contains("отель") {
        Some("Hotel".into())
    } else if !is_online && !original.is_empty() {
        Some(original.to_string())
    } else {
        None
    }
}
