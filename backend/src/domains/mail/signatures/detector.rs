use chrono::Utc;
use serde::Serialize;

use super::{CertificateRecord, CertificateType};

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
            signature_detected(CertificateType::Smime)
        } else if has_pgp {
            signature_detected(CertificateType::Pgp)
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
        let until = cert.valid_until?;
        let days = (until - Utc::now()).num_days();
        if days <= 0 {
            Some("Certificate has expired".into())
        } else if days <= 90 {
            Some(format!("Certificate expires in {days} days"))
        } else {
            None
        }
    }
}

fn signature_detected(signature_type: CertificateType) -> SignatureDetection {
    SignatureDetection {
        has_signature: true,
        signature_type: Some(signature_type),
        signer_info: None,
        is_valid: None,
        cert_expiry_warning: None,
    }
}
