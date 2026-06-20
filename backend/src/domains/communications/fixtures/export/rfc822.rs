use super::body::body_text_from_part;
use super::errors::EmailFixtureExportError;
use super::headers::{header_value, parse_headers, split_address_list};
use super::models::ParsedRfc822Message;
use super::text::{non_empty_or_default, non_empty_recipients};

pub(super) fn parse_rfc822_message(
    raw: &[u8],
) -> Result<ParsedRfc822Message, EmailFixtureExportError> {
    let raw = String::from_utf8_lossy(raw);
    let (header_block, body) = split_headers_and_body(&raw)?;
    let headers = parse_headers(header_block);

    let subject = header_value(&headers, "subject").unwrap_or_else(|| "(no subject)".to_owned());
    let from =
        header_value(&headers, "from").unwrap_or_else(|| "unknown@example.invalid".to_owned());
    let to = split_address_list(&header_value(&headers, "to").unwrap_or_default());
    let body_text = body_text_from_part(&headers, body);

    Ok(ParsedRfc822Message {
        subject: non_empty_or_default(subject, "(no subject)"),
        from: non_empty_or_default(from, "unknown@example.invalid"),
        to: non_empty_recipients(to),
        body_text: non_empty_or_default(body_text, "(empty body)"),
    })
}

pub(super) fn split_headers_and_body(raw: &str) -> Result<(&str, &str), EmailFixtureExportError> {
    if let Some((headers, body)) = raw.split_once("\r\n\r\n") {
        return Ok((headers, body));
    }
    if let Some((headers, body)) = raw.split_once("\n\n") {
        return Ok((headers, body));
    }

    Err(EmailFixtureExportError::MalformedRfc822)
}
