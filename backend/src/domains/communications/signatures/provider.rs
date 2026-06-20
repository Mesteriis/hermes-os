use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificateProvider {
    Fnmt,
    Dnie,
    CryptoPro,
    Gost,
    AppleKeychain,
    Pkcs12,
    Yubikey,
    UsbToken,
    Other,
}

impl CertificateProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fnmt => "fnmt",
            Self::Dnie => "dnie",
            Self::CryptoPro => "cryptopro",
            Self::Gost => "gost",
            Self::AppleKeychain => "apple_keychain",
            Self::Pkcs12 => "pkcs12",
            Self::Yubikey => "yubikey",
            Self::UsbToken => "usb_token",
            Self::Other => "other",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "fnmt" => Some(Self::Fnmt),
            "dnie" => Some(Self::Dnie),
            "cryptopro" => Some(Self::CryptoPro),
            "gost" => Some(Self::Gost),
            "apple_keychain" => Some(Self::AppleKeychain),
            "pkcs12" => Some(Self::Pkcs12),
            "yubikey" => Some(Self::Yubikey),
            "usb_token" => Some(Self::UsbToken),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}
