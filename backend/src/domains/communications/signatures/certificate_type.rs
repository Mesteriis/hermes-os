use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificateType {
    Smime,
    Pgp,
    PdfSign,
    Cades,
    Xades,
    GostSign,
    Unknown,
}

impl CertificateType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Smime => "smime",
            Self::Pgp => "pgp",
            Self::PdfSign => "pdf_sign",
            Self::Cades => "cades",
            Self::Xades => "xades",
            Self::GostSign => "gost_sign",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "smime" => Some(Self::Smime),
            "pgp" => Some(Self::Pgp),
            "pdf_sign" => Some(Self::PdfSign),
            "cades" => Some(Self::Cades),
            "xades" => Some(Self::Xades),
            "gost_sign" => Some(Self::GostSign),
            _ => None,
        }
    }
}
