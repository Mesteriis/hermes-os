//! Authenticated first-party owner principal shared by public control ceremonies.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnerBrowserPrincipalV1 {
    owner_id: String,
    device_id: String,
    session_id: String,
}

impl OwnerBrowserPrincipalV1 {
    pub fn new(
        owner_id: impl Into<String>,
        device_id: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Result<Self, &'static str> {
        let principal = Self {
            owner_id: owner_id.into(),
            device_id: device_id.into(),
            session_id: session_id.into(),
        };
        if [
            &principal.owner_id,
            &principal.device_id,
            &principal.session_id,
        ]
        .into_iter()
        .any(|value| {
            value.is_empty()
                || value.len() > 128
                || !value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        }) {
            return Err("owner browser principal is invalid");
        }
        Ok(principal)
    }

    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.owner_id
    }

    #[must_use]
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}
