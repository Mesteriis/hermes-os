use std::fmt::{Display, Formatter};

pub type LegacyProviderRecoveryResultV1<T> = Result<T, LegacyProviderRecoveryErrorV1>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegacyProviderRecoveryErrorV1 {
    InvalidArguments,
    InvalidSource,
    InvalidBundle,
    InvalidCatalog,
    InvalidConfiguration,
    InvalidSecret,
    SourceChanged,
    SessionUnavailable,
    CapacityExceeded,
    UnsupportedPlatform,
    IoUnavailable,
    DatabaseUnavailable,
    CryptographyUnavailable,
}

impl LegacyProviderRecoveryErrorV1 {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidArguments => "invalid_arguments",
            Self::InvalidSource => "invalid_source",
            Self::InvalidBundle => "invalid_bundle",
            Self::InvalidCatalog => "invalid_catalog",
            Self::InvalidConfiguration => "invalid_configuration",
            Self::InvalidSecret => "invalid_secret",
            Self::SourceChanged => "source_changed",
            Self::SessionUnavailable => "session_unavailable",
            Self::CapacityExceeded => "capacity_exceeded",
            Self::UnsupportedPlatform => "unsupported_platform",
            Self::IoUnavailable => "io_unavailable",
            Self::DatabaseUnavailable => "database_unavailable",
            Self::CryptographyUnavailable => "cryptography_unavailable",
        }
    }
}

impl Display for LegacyProviderRecoveryErrorV1 {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for LegacyProviderRecoveryErrorV1 {}
