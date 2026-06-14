use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustStatus {
    Trusted,
    Untrusted,
    Expired,
    Revoked,
    PendingVerification,
    SelfSigned,
}

impl TrustStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::Untrusted => "untrusted",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
            Self::PendingVerification => "pending_verification",
            Self::SelfSigned => "self_signed",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "trusted" => Some(Self::Trusted),
            "untrusted" => Some(Self::Untrusted),
            "expired" => Some(Self::Expired),
            "revoked" => Some(Self::Revoked),
            "pending_verification" => Some(Self::PendingVerification),
            "self_signed" => Some(Self::SelfSigned),
            _ => None,
        }
    }
}
