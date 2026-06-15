pub(super) const DEFAULT_GOOGLE_AUTHORIZATION_ENDPOINT: &str =
    "https://accounts.google.com/o/oauth2/v2/auth";
pub(super) const DEFAULT_GOOGLE_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";

const GOOGLE_GMAIL_READONLY_SCOPE: &str = "https://www.googleapis.com/auth/gmail.readonly";
pub(crate) const GOOGLE_GMAIL_SEND_SCOPE: &str = "https://www.googleapis.com/auth/gmail.send";
const GOOGLE_CALENDAR_READONLY_SCOPE: &str = "https://www.googleapis.com/auth/calendar.readonly";
const GOOGLE_CONTACTS_READONLY_SCOPE: &str = "https://www.googleapis.com/auth/contacts.readonly";

pub(super) const DEFAULT_GOOGLE_WORKSPACE_SCOPES: [&str; 4] = [
    GOOGLE_GMAIL_READONLY_SCOPE,
    GOOGLE_GMAIL_SEND_SCOPE,
    GOOGLE_CALENDAR_READONLY_SCOPE,
    GOOGLE_CONTACTS_READONLY_SCOPE,
];
