mod analytics;
mod classification;
mod conference;
mod errors;
mod fingerprint;
mod location;
mod models;
mod scoring;

pub struct CalendarIntelligenceService;

pub use errors::CalendarIntelligenceError;
pub use models::{BackToBackGroup, EventAnalysis, EventFingerprint, LocationInfo};
