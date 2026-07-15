use qrcode::QrCode;
use qrcode::render::svg;

use crate::integrations::telegram::client::errors::TelegramError;

pub(in crate::integrations::telegram::tdjson) fn render_qr_svg(
    link: &str,
) -> Result<String, TelegramError> {
    let code = QrCode::new(link.as_bytes())
        .map_err(|error| TelegramError::QrGeneration(format!("failed to encode QR: {error}")))?;
    Ok(code
        .render::<svg::Color<'_>>()
        .min_dimensions(240, 240)
        .build())
}
