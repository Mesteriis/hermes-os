use super::body::body_content_from_part;
use super::errors::EmailRfc822ParseError;
use super::headers::{header_value, parse_headers};
use super::models::ParsedEmailMessage;
use super::util::{non_empty_or_default, non_empty_recipients, split_address_list};
use super::wire::split_headers_and_body;

pub fn parse_rfc822_message(raw: &[u8]) -> Result<ParsedEmailMessage, EmailRfc822ParseError> {
    let (header_block, body) = split_headers_and_body(raw)?;
    let headers = parse_headers(header_block);

    let subject = header_value(&headers, "subject").unwrap_or_else(|| "(no subject)".to_owned());
    let from =
        header_value(&headers, "from").unwrap_or_else(|| "unknown@example.invalid".to_owned());
    let to = split_address_list(&header_value(&headers, "to").unwrap_or_default());
    let body_content = body_content_from_part(&headers, body);
    let body_text = body_content
        .body_text
        .unwrap_or_else(|| "(empty body)".to_owned());

    Ok(ParsedEmailMessage {
        subject: non_empty_or_default(subject, "(no subject)"),
        from: non_empty_or_default(from, "unknown@example.invalid"),
        to: non_empty_recipients(to),
        headers,
        body_text: non_empty_or_default(body_text, "(empty body)"),
        body_html: body_content.body_html,
        attachments: body_content.attachments,
    })
}
