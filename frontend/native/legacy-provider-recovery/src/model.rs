use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

pub const RECOVERY_SCHEMA_REVISION: u16 = 1;
pub const EXPECTED_GMAIL_ACTIVE: u16 = 1;
pub const EXPECTED_ICLOUD_ACTIVE: u16 = 1;
pub const EXPECTED_TELEGRAM_USER_ACTIVE: u16 = 1;
pub const EXPECTED_GMAIL_DELETED: u16 = 2;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyProviderCandidateKindV1 {
    Gmail,
    Icloud,
    TelegramUser,
}

impl LegacyProviderCandidateKindV1 {
    #[cfg(feature = "prepare")]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Gmail => "gmail",
            Self::Icloud => "icloud",
            Self::TelegramUser => "telegram_user",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyProviderRecoveryTerminalStateV1 {
    Completed,
    ReauthorizationRequired,
    QrAuthorizationRequired,
    BlockedSource,
    BlockedConfig,
    OutcomeUnknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyProviderRecoveryStepDispositionV1 {
    Execute,
    Completed,
    OutcomeUnknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegacyProviderRecoveryStepV1 {
    pub disposition: LegacyProviderRecoveryStepDispositionV1,
    pub operation_id: [u8; 16],
    pub target_configuration_instance_id: Option<String>,
    pub public_revision: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyProviderRecoverySecretPurposeV1 {
    IcloudImapPassword,
    TelegramApiHash,
    GeneratedTelegramSessionStoreKey,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyProviderRecoveryStateV1 {
    ReadyToApply,
    ReauthorizationRequired,
    QrAuthorizationRequired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyProviderRecoveryCountsV1 {
    pub gmail_active: u16,
    pub icloud_active: u16,
    pub telegram_user_active: u16,
    pub gmail_deleted: u16,
}

impl LegacyProviderRecoveryCountsV1 {
    pub(crate) fn exact() -> Self {
        Self {
            gmail_active: EXPECTED_GMAIL_ACTIVE,
            icloud_active: EXPECTED_ICLOUD_ACTIVE,
            telegram_user_active: EXPECTED_TELEGRAM_USER_ACTIVE,
            gmail_deleted: EXPECTED_GMAIL_DELETED,
        }
    }

    pub(crate) fn is_exact(&self) -> bool {
        self == &Self::exact()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyProviderRecoveryCandidateV1 {
    pub handle: String,
    pub kind: LegacyProviderCandidateKindV1,
    pub state: LegacyProviderRecoveryStateV1,
    pub terminal_state: Option<LegacyProviderRecoveryTerminalStateV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyProviderRecoveryPlanV1 {
    pub schema_revision: u16,
    pub session_id: String,
    pub bundle_fingerprint_sha256: String,
    pub counts: LegacyProviderRecoveryCountsV1,
    pub candidates: Vec<LegacyProviderRecoveryCandidateV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LegacyProviderRecoverySourceV1 {
    Gmail {
        handle: String,
        account_id: String,
        display_name: String,
        email: String,
        oauth_client_id: String,
        oauth_redirect_uri: String,
    },
    Icloud {
        handle: String,
        account_id: String,
        display_name: String,
        email: String,
        imap_host: String,
        imap_port: u16,
        username: String,
    },
    TelegramUser {
        handle: String,
        account_id: String,
        display_name: String,
        external_account_id: String,
        api_id: i64,
    },
}

impl LegacyProviderRecoverySourceV1 {
    pub fn handle(&self) -> &str {
        match self {
            Self::Gmail { handle, .. }
            | Self::Icloud { handle, .. }
            | Self::TelegramUser { handle, .. } => handle,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryCatalogV1 {
    pub schema_revision: u16,
    pub source_generation: String,
    pub counts: LegacyProviderRecoveryCountsV1,
    pub candidates: Vec<RecoveryCatalogCandidateV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryCatalogCandidateV1 {
    pub kind: LegacyProviderCandidateKindV1,
    pub source_account_digest_sha256: String,
    pub account_id: String,
    pub display_name: String,
    pub external_account_id: String,
    pub configuration: RecoveryCatalogConfigurationV1,
    pub legacy_secret: RecoveryCatalogSecretBindingV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum RecoveryCatalogConfigurationV1 {
    Gmail {
        oauth_client_id: String,
    },
    Icloud {
        imap_host: String,
        imap_port: u16,
        tls: bool,
        username: String,
    },
    TelegramUser,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryCatalogSecretBindingV1 {
    pub purpose: String,
    pub secret_ref: String,
    pub secret_kind: String,
    pub store_kind: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LegacyProviderConfigurationV1 {
    pub schema_revision: u16,
    pub telegram_api_id: i64,
    pub telegram_api_hash: Zeroizing<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GoogleOauthClientV1 {
    pub schema_revision: u16,
    pub client_id: String,
    pub redirect_uris: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryBundleManifestV1 {
    pub schema_revision: u16,
    pub created_at_unix_seconds: u64,
    pub source_generation: String,
    pub files: Vec<RecoveryBundleFileV1>,
    pub catalog_row_count: u16,
    pub counts: LegacyProviderRecoveryCountsV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryBundleFileV1 {
    pub relative_path: String,
    pub size_bytes: u64,
    pub sha256: String,
}
