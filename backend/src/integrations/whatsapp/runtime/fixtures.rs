use qrcode::QrCode;
use qrcode::render::svg;

use super::{WhatsappWebError, short_hash};

pub(super) fn setup_id(prefix: &str, account_id: &str) -> String {
    format!(
        "{prefix}-{}-{}",
        account_id.trim(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    )
}

pub(super) fn fixture_whatsapp_qr_payload(account_id: &str, setup_id: &str) -> String {
    format!(
        "hermes-whatsapp-fixture://link?account_id={}&setup_id={}",
        account_id.trim(),
        setup_id.trim()
    )
}

pub(super) fn render_fixture_whatsapp_qr_svg(payload: &str) -> Result<String, WhatsappWebError> {
    let code = QrCode::new(payload.as_bytes()).map_err(|error| {
        WhatsappWebError::InvalidRequest(format!("failed to encode QR: {error}"))
    })?;
    Ok(code
        .render::<svg::Color<'_>>()
        .min_dimensions(240, 240)
        .build())
}

pub(super) fn fixture_whatsapp_pair_code(
    account_id: &str,
    phone_number: &str,
    setup_id: &str,
) -> String {
    let seed = short_hash(&format!(
        "{}:{}:{}",
        account_id.trim(),
        phone_number.trim(),
        setup_id.trim()
    ))
    .to_uppercase();
    format!("{}-{}", &seed[..4], &seed[4..8])
}
