use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EmailCategory {
    Critical,
    Important,
    Personal,
    Work,
    Finance,
    Legal,
    Notification,
    Newsletter,
    Marketing,
    Spam,
    Scam,
    Phishing,
    Suspicious,
}

impl EmailCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmailCategory::Critical => "critical",
            EmailCategory::Important => "important",
            EmailCategory::Personal => "personal",
            EmailCategory::Work => "work",
            EmailCategory::Finance => "finance",
            EmailCategory::Legal => "legal",
            EmailCategory::Notification => "notification",
            EmailCategory::Newsletter => "newsletter",
            EmailCategory::Marketing => "marketing",
            EmailCategory::Spam => "spam",
            EmailCategory::Scam => "scam",
            EmailCategory::Phishing => "phishing",
            EmailCategory::Suspicious => "suspicious",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "critical" => Some(EmailCategory::Critical),
            "important" => Some(EmailCategory::Important),
            "personal" => Some(EmailCategory::Personal),
            "work" => Some(EmailCategory::Work),
            "finance" => Some(EmailCategory::Finance),
            "legal" => Some(EmailCategory::Legal),
            "notification" => Some(EmailCategory::Notification),
            "newsletter" => Some(EmailCategory::Newsletter),
            "marketing" => Some(EmailCategory::Marketing),
            "spam" => Some(EmailCategory::Spam),
            "scam" => Some(EmailCategory::Scam),
            "phishing" => Some(EmailCategory::Phishing),
            "suspicious" => Some(EmailCategory::Suspicious),
            _ => None,
        }
    }
}
