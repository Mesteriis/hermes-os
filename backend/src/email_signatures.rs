use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use sqlx::postgres::{PgPool, PgRow};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificateRecord {
    pub cert_id: String,
    pub owner_name: String,
    pub issuer: String,
    pub serial_number: Option<String>,
    pub fingerprint_sha256: Option<String>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub cert_type: CertificateType,
    pub provider: CertificateProvider,
    pub storage_kind: CertificateStorageKind,
    pub storage_ref: Option<String>,
    pub trust_status: TrustStatus,
    pub is_revoked: bool,
    pub usage: Vec<String>,
    pub linked_message_id: Option<String>,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

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
    pub fn parse(v: &str) -> Option<Self> {
        match v {
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
    pub fn parse(v: &str) -> Option<Self> {
        match v {
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
    pub fn parse(v: &str) -> Option<Self> {
        match v {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustStatus {
    Trusted,
    Untrusted,
    Expired,
    Revoked,
    PendingVerification,
    SelfSigned,
}

impl TrustStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::Untrusted => "untrusted",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
            Self::PendingVerification => "pending_verification",
            Self::SelfSigned => "self_signed",
        }
    }
    pub fn parse(v: &str) -> Option<Self> {
        match v {
            "trusted" => Some(Self::Trusted),
            "untrusted" => Some(Self::Untrusted),
            "expired" => Some(Self::Expired),
            "revoked" => Some(Self::Revoked),
            "pending_verification" => Some(Self::PendingVerification),
            "self_signed" => Some(Self::SelfSigned),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SignatureDetection {
    pub has_signature: bool,
    pub signature_type: Option<CertificateType>,
    pub signer_info: Option<String>,
    pub is_valid: Option<bool>,
    pub cert_expiry_warning: Option<String>,
}

pub struct SignatureDetector;

impl SignatureDetector {
    pub fn detect_in_message(body_text: &str, headers: &str) -> SignatureDetection {
        let has_smime = headers.contains("Content-Type: application/pkcs7-mime")
            || headers.contains("Content-Type: application/x-pkcs7-signature");
        let has_pgp = body_text.contains("-----BEGIN PGP SIGNATURE-----")
            || body_text.contains("-----BEGIN PGP MESSAGE-----");

        if has_smime {
            SignatureDetection {
                has_signature: true,
                signature_type: Some(CertificateType::Smime),
                signer_info: None,
                is_valid: None,
                cert_expiry_warning: None,
            }
        } else if has_pgp {
            SignatureDetection {
                has_signature: true,
                signature_type: Some(CertificateType::Pgp),
                signer_info: None,
                is_valid: None,
                cert_expiry_warning: None,
            }
        } else {
            SignatureDetection {
                has_signature: false,
                signature_type: None,
                signer_info: None,
                is_valid: None,
                cert_expiry_warning: None,
            }
        }
    }

    pub fn check_expiry_warning(cert: &CertificateRecord) -> Option<String> {
        if let Some(until) = cert.valid_until {
            let remaining = until - Utc::now();
            let days = remaining.num_days();
            if days <= 0 {
                Some("Certificate has expired".into())
            } else if days <= 90 {
                Some(format!("Certificate expires in {days} days"))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct CertificateStore {
    pool: PgPool,
}

impl CertificateStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(
        &self,
        cert: &NewCertificate,
    ) -> Result<CertificateRecord, CertificateError> {
        cert.validate()?;
        let row = sqlx::query(
            r#"INSERT INTO email_certificates (cert_id, owner_name, issuer, serial_number, fingerprint_sha256, valid_from, valid_until, cert_type, provider, storage_kind, storage_ref, trust_status, is_revoked, usage, linked_message_id, metadata)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
            ON CONFLICT (cert_id) DO UPDATE SET owner_name=EXCLUDED.owner_name, issuer=EXCLUDED.issuer, valid_from=EXCLUDED.valid_from, valid_until=EXCLUDED.valid_until, trust_status=EXCLUDED.trust_status, is_revoked=EXCLUDED.is_revoked, usage=EXCLUDED.usage, metadata=EXCLUDED.metadata, updated_at=now()
            RETURNING cert_id,owner_name,issuer,serial_number,fingerprint_sha256,valid_from,valid_until,cert_type,provider,storage_kind,storage_ref,trust_status,is_revoked,usage,linked_message_id,metadata,created_at,updated_at"#,
        )
        .bind(&cert.cert_id).bind(&cert.owner_name).bind(&cert.issuer).bind(cert.serial_number.as_deref())
        .bind(cert.fingerprint_sha256.as_deref()).bind(cert.valid_from).bind(cert.valid_until)
        .bind(cert.cert_type.as_str()).bind(cert.provider.as_str()).bind(cert.storage_kind.as_str())
        .bind(cert.storage_ref.as_deref()).bind(cert.trust_status.as_str()).bind(cert.is_revoked)
        .bind(serde_json::to_value(&cert.usage).unwrap_or_default()).bind(cert.linked_message_id.as_deref())
        .bind(&cert.metadata).fetch_one(&self.pool).await?;
        row_to_cert(row)
    }

    pub async fn list(&self) -> Result<Vec<CertificateRecord>, CertificateError> {
        let rows = sqlx::query("SELECT cert_id,owner_name,issuer,serial_number,fingerprint_sha256,valid_from,valid_until,cert_type,provider,storage_kind,storage_ref,trust_status,is_revoked,usage,linked_message_id,metadata,created_at,updated_at FROM email_certificates ORDER BY COALESCE(valid_until, created_at) DESC").fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_cert).collect()
    }

    pub async fn expiring_soon(
        &self,
        days: i64,
    ) -> Result<Vec<CertificateRecord>, CertificateError> {
        let rows = sqlx::query("SELECT cert_id,owner_name,issuer,serial_number,fingerprint_sha256,valid_from,valid_until,cert_type,provider,storage_kind,storage_ref,trust_status,is_revoked,usage,linked_message_id,metadata,created_at,updated_at FROM email_certificates WHERE valid_until IS NOT NULL AND valid_until BETWEEN now() AND now() + ($1 || ' days')::interval AND is_revoked = false ORDER BY valid_until ASC").bind(days).fetch_all(&self.pool).await?;
        rows.into_iter().map(row_to_cert).collect()
    }
}

fn row_to_cert(row: PgRow) -> Result<CertificateRecord, CertificateError> {
    Ok(CertificateRecord {
        cert_id: row.try_get("cert_id")?,
        owner_name: row.try_get("owner_name")?,
        issuer: row.try_get("issuer")?,
        serial_number: row.try_get("serial_number")?,
        fingerprint_sha256: row.try_get("fingerprint_sha256")?,
        valid_from: row.try_get("valid_from")?,
        valid_until: row.try_get("valid_until")?,
        cert_type: CertificateType::parse(&row.try_get::<String, _>("cert_type")?)
            .unwrap_or(CertificateType::Unknown),
        provider: CertificateProvider::parse(&row.try_get::<String, _>("provider")?)
            .unwrap_or(CertificateProvider::Other),
        storage_kind: CertificateStorageKind::parse(&row.try_get::<String, _>("storage_kind")?)
            .unwrap_or(CertificateStorageKind::EncryptedVault),
        storage_ref: row.try_get("storage_ref")?,
        trust_status: TrustStatus::parse(&row.try_get::<String, _>("trust_status")?)
            .unwrap_or(TrustStatus::Untrusted),
        is_revoked: row.try_get("is_revoked")?,
        usage: serde_json::from_value(row.try_get("usage")?).unwrap_or_default(),
        linked_message_id: row.try_get("linked_message_id")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[derive(Clone, Debug)]
pub struct NewCertificate {
    pub cert_id: String,
    pub owner_name: String,
    pub issuer: String,
    pub serial_number: Option<String>,
    pub fingerprint_sha256: Option<String>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub cert_type: CertificateType,
    pub provider: CertificateProvider,
    pub storage_kind: CertificateStorageKind,
    pub storage_ref: Option<String>,
    pub trust_status: TrustStatus,
    pub is_revoked: bool,
    pub usage: Vec<String>,
    pub linked_message_id: Option<String>,
    pub metadata: Value,
}
impl NewCertificate {
    fn validate(&self) -> Result<(), CertificateError> {
        if self.cert_id.trim().is_empty() {
            Err(CertificateError::Invalid("cert_id empty"))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Error)]
pub enum CertificateError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("invalid cert: {0}")]
    Invalid(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detect_smime() {
        let r =
            SignatureDetector::detect_in_message("body", "Content-Type: application/pkcs7-mime\n");
        assert!(r.has_signature);
        assert_eq!(r.signature_type, Some(CertificateType::Smime));
    }
    #[test]
    fn detect_pgp() {
        let r = SignatureDetector::detect_in_message(
            "-----BEGIN PGP SIGNATURE-----\nxyz\n-----END PGP SIGNATURE-----",
            "",
        );
        assert!(r.has_signature);
    }
    #[test]
    fn detect_none() {
        let r = SignatureDetector::detect_in_message("plain text", "");
        assert!(!r.has_signature);
    }
    #[test]
    fn cert_types_roundtrip() {
        for t in [
            CertificateType::Smime,
            CertificateType::Cades,
            CertificateType::GostSign,
            CertificateType::Pgp,
        ] {
            assert_eq!(CertificateType::parse(t.as_str()), Some(t));
        }
    }
}
