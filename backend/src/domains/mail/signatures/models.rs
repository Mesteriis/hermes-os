use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    CertificateError, CertificateProvider, CertificateStorageKind, CertificateType, TrustStatus,
};

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
    pub(super) fn validate(&self) -> Result<(), CertificateError> {
        if self.cert_id.trim().is_empty() {
            Err(CertificateError::Invalid("cert_id empty"))
        } else {
            Ok(())
        }
    }
}
