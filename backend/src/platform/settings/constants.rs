pub(crate) const SECRET_LIKE_MARKERS: [&str; 5] =
    ["secret", "password", "token", "credential", "private_key"];
pub(crate) const UI_STATE_FORBIDDEN_KEYS: [&str; 7] =
    ["body", "html", "raw", "text", "password", "token", "secret"];
pub(crate) const UI_STATE_MAX_BYTES: u64 = 65_536;
