//! Public, revocable browser device identities for the future Core Gateway.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserDeviceStateV1 {
    Active,
    Revoked,
}

impl BrowserDeviceStateV1 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Revoked => "revoked",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "revoked" => Some(Self::Revoked),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserDeviceEnrollmentV1 {
    owner_id: String,
    device_id: String,
    credential_id: Vec<u8>,
    cose_public_key: Vec<u8>,
    browser_key_public_key: Vec<u8>,
    rp_id: String,
    sign_count: u32,
    backup_eligible: bool,
    backup_state: bool,
}

pub struct BrowserDeviceEnrollmentInputV1 {
    pub owner_id: String,
    pub device_id: String,
    pub credential_id: Vec<u8>,
    pub cose_public_key: Vec<u8>,
    pub browser_key_public_key: Vec<u8>,
    pub rp_id: String,
    pub sign_count: u32,
    pub backup_eligible: bool,
    pub backup_state: bool,
}

impl BrowserDeviceEnrollmentV1 {
    pub fn new(fields: BrowserDeviceEnrollmentInputV1) -> Result<Self, String> {
        if !valid_token(&fields.owner_id)
            || !valid_token(&fields.device_id)
            || !valid_rp_id(&fields.rp_id)
            || fields.credential_id.is_empty()
            || fields.credential_id.len() > 1024
            || !(16..=1024).contains(&fields.cose_public_key.len())
            || !valid_browser_key_public_key(&fields.browser_key_public_key)
            || (fields.backup_state && !fields.backup_eligible)
        {
            return Err("browser device enrollment is invalid".to_owned());
        }
        Ok(Self {
            owner_id: fields.owner_id,
            device_id: fields.device_id,
            credential_id: fields.credential_id,
            cose_public_key: fields.cose_public_key,
            browser_key_public_key: fields.browser_key_public_key,
            rp_id: fields.rp_id,
            sign_count: fields.sign_count,
            backup_eligible: fields.backup_eligible,
            backup_state: fields.backup_state,
        })
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
    pub fn credential_id(&self) -> &[u8] {
        &self.credential_id
    }

    #[must_use]
    pub fn cose_public_key(&self) -> &[u8] {
        &self.cose_public_key
    }

    #[must_use]
    pub fn browser_key_public_key(&self) -> &[u8] {
        &self.browser_key_public_key
    }

    #[must_use]
    pub fn rp_id(&self) -> &str {
        &self.rp_id
    }

    #[must_use]
    pub const fn sign_count(&self) -> u32 {
        self.sign_count
    }

    #[must_use]
    pub const fn backup_eligible(&self) -> bool {
        self.backup_eligible
    }

    #[must_use]
    pub const fn backup_state(&self) -> bool {
        self.backup_state
    }
}

fn valid_browser_key_public_key(value: &[u8]) -> bool {
    value.len() == 65 && value.first() == Some(&4)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserDeviceIdentityV1 {
    enrollment: BrowserDeviceEnrollmentV1,
    state: BrowserDeviceStateV1,
    identity_epoch: u64,
}

impl BrowserDeviceIdentityV1 {
    pub fn new(
        enrollment: BrowserDeviceEnrollmentV1,
        state: BrowserDeviceStateV1,
        identity_epoch: u64,
    ) -> Result<Self, String> {
        if identity_epoch == 0 {
            return Err("browser device identity epoch is invalid".to_owned());
        }
        Ok(Self {
            enrollment,
            state,
            identity_epoch,
        })
    }

    #[must_use]
    pub fn enrollment(&self) -> &BrowserDeviceEnrollmentV1 {
        &self.enrollment
    }

    #[must_use]
    pub const fn state(&self) -> BrowserDeviceStateV1 {
        self.state
    }

    #[must_use]
    pub const fn identity_epoch(&self) -> u64 {
        self.identity_epoch
    }
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

fn valid_rp_id(value: &str) -> bool {
    value == "localhost"
        || (value.len() <= 253
            && value.split('.').count() >= 2
            && value.split('.').all(|label| {
                !label.is_empty()
                    && label.len() <= 63
                    && label
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                    && !label.starts_with('-')
                    && !label.ends_with('-')
            }))
}
