use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificateStorageKind {
    OsKeychain,
    EncryptedVault,
    Pkcs12File,
    PfxFile,
    SmartCard,
    UsbToken,
    ExternalVault,
}

impl CertificateStorageKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OsKeychain => "os_keychain",
            Self::EncryptedVault => "encrypted_vault",
            Self::Pkcs12File => "pkcs12_file",
            Self::PfxFile => "pfx_file",
            Self::SmartCard => "smart_card",
            Self::UsbToken => "usb_token",
            Self::ExternalVault => "external_vault",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "os_keychain" => Some(Self::OsKeychain),
            "encrypted_vault" => Some(Self::EncryptedVault),
            "pkcs12_file" => Some(Self::Pkcs12File),
            "pfx_file" => Some(Self::PfxFile),
            "smart_card" => Some(Self::SmartCard),
            "usb_token" => Some(Self::UsbToken),
            "external_vault" => Some(Self::ExternalVault),
            _ => None,
        }
    }
}
