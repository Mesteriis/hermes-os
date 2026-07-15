use thiserror::Error;

#[derive(Debug, Error)]
pub enum TdlibProtocolError {
    #[error("invalid Telegram TDLib command: {0}")]
    InvalidCommand(&'static str),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TdlibMediaKind {
    Photo,
    Video,
    Document,
    Audio,
    Voice,
    Sticker,
    Animation,
}

impl TdlibMediaKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Photo => "photo",
            Self::Video => "video",
            Self::Document => "document",
            Self::Audio => "audio",
            Self::Voice => "voice",
            Self::Sticker => "sticker",
            Self::Animation => "animation",
        }
    }
}

impl TryFrom<&str> for TdlibMediaKind {
    type Error = TdlibProtocolError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "photo" => Ok(Self::Photo),
            "video" => Ok(Self::Video),
            "document" => Ok(Self::Document),
            "audio" => Ok(Self::Audio),
            "voice" | "voice_note" => Ok(Self::Voice),
            "sticker" => Ok(Self::Sticker),
            "animation" | "gif" => Ok(Self::Animation),
            _ => Err(TdlibProtocolError::InvalidCommand(
                "unsupported media upload type",
            )),
        }
    }
}
