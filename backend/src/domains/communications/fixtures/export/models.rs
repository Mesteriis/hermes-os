#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmailFixturePrivacyMode {
    Redacted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmailFixtureExportOptions {
    pub privacy_mode: EmailFixturePrivacyMode,
}

impl Default for EmailFixtureExportOptions {
    fn default() -> Self {
        Self {
            privacy_mode: EmailFixturePrivacyMode::Redacted,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ParsedRfc822Message {
    pub(super) subject: String,
    pub(super) from: String,
    pub(super) to: Vec<String>,
    pub(super) body_text: String,
}
