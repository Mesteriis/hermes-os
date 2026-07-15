use super::*;

#[test]
fn detect_smime() {
    let result =
        SignatureDetector::detect_in_message("body", "Content-Type: application/pkcs7-mime\n");

    assert!(result.has_signature);
    assert_eq!(result.signature_type, Some(CertificateType::Smime));
}

#[test]
fn detect_pgp() {
    let result = SignatureDetector::detect_in_message(
        "-----BEGIN PGP SIGNATURE-----\nxyz\n-----END PGP SIGNATURE-----",
        "",
    );

    assert!(result.has_signature);
}

#[test]
fn detect_none() {
    let result = SignatureDetector::detect_in_message("plain text", "");

    assert!(!result.has_signature);
}

#[test]
fn cert_types_roundtrip() {
    for cert_type in [
        CertificateType::Smime,
        CertificateType::Cades,
        CertificateType::GostSign,
        CertificateType::Pgp,
    ] {
        assert_eq!(CertificateType::parse(cert_type.as_str()), Some(cert_type));
    }
}
use super::certificate_type::CertificateType;
use super::detector::SignatureDetector;
