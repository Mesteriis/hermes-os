use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedEmailMessage {
    pub subject: String,
    pub from: String,
    pub to: Vec<String>,
    pub body_text: String,
    pub attachments: Vec<ParsedEmailAttachment>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedEmailAttachment {
    pub provider_attachment_id: String,
    pub filename: Option<String>,
    pub content_type: String,
    pub disposition: ParsedEmailAttachmentDisposition,
    pub body_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParsedEmailAttachmentDisposition {
    Attachment,
    Inline,
    Unknown,
}

pub fn parse_rfc822_message(raw: &[u8]) -> Result<ParsedEmailMessage, EmailRfc822ParseError> {
    let raw = String::from_utf8_lossy(raw);
    let (header_block, body) = split_headers_and_body(&raw)?;
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
        body_text: non_empty_or_default(body_text, "(empty body)"),
        attachments: body_content.attachments,
    })
}

fn split_headers_and_body(raw: &str) -> Result<(&str, &str), EmailRfc822ParseError> {
    if let Some((headers, body)) = raw.split_once("\r\n\r\n") {
        return Ok((headers, body));
    }
    if let Some((headers, body)) = raw.split_once("\n\n") {
        return Ok((headers, body));
    }

    Err(EmailRfc822ParseError::MalformedRfc822)
}

fn parse_headers(header_block: &str) -> Vec<(String, String)> {
    let mut headers: Vec<(String, String)> = Vec::new();

    for line in header_block.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            if let Some((_, value)) = headers.last_mut() {
                value.push(' ');
                value.push_str(line.trim());
            }
            continue;
        }

        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_ascii_lowercase(), value.trim().to_owned()));
        }
    }

    headers
}

fn header_value(headers: &[(String, String)], name: &str) -> Option<String> {
    headers
        .iter()
        .find(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
        .map(|(_, value)| decode_rfc2047_words(value.trim()))
}

#[derive(Default)]
struct ParsedEmailBodyContent {
    body_text: Option<String>,
    attachments: Vec<ParsedEmailAttachment>,
    next_attachment_index: usize,
}

fn body_content_from_part(headers: &[(String, String)], body: &str) -> ParsedEmailBodyContent {
    let mut content = ParsedEmailBodyContent::default();
    collect_part_content(headers, body, &mut content);
    content
}

fn collect_part_content(
    headers: &[(String, String)],
    body: &str,
    content: &mut ParsedEmailBodyContent,
) {
    let content_type = header_value(headers, "content-type").unwrap_or_default();
    let content_type_media_type = header_media_type(&content_type);

    if content_type_media_type.starts_with("multipart/") {
        if let Some(boundary) = header_parameter(&content_type, "boundary") {
            for (part_headers, part_body) in multipart_parts(&boundary, body) {
                collect_part_content(&part_headers, part_body, content);
            }
        }
        return;
    }

    if is_attachment_like_part(headers, &content_type) {
        content.next_attachment_index += 1;
        let provider_attachment_id = format!("part-{}", content.next_attachment_index);
        content.attachments.push(parsed_attachment_from_part(
            headers,
            body,
            &content_type,
            provider_attachment_id,
        ));
        return;
    }

    if content.body_text.is_some() {
        return;
    }

    let decoded = decode_transfer_body(
        body,
        header_value(headers, "content-transfer-encoding")
            .unwrap_or_default()
            .as_str(),
    );
    if content_type_media_type == "text/html" {
        content.body_text = Some(strip_html_tags(&decoded));
        return;
    }
    if content_type_media_type == "text/plain" || content_type_media_type.is_empty() {
        content.body_text = Some(normalize_body_text(&decoded));
    }
}

fn multipart_parts<'a>(boundary: &str, body: &'a str) -> Vec<(Vec<(String, String)>, &'a str)> {
    let mut parts = Vec::new();
    let delimiter = format!("--{boundary}");
    for raw_part in body.split(&delimiter).skip(1) {
        let part = raw_part.trim_start_matches("\r\n").trim_start_matches('\n');
        if part.starts_with("--") {
            break;
        }
        let Ok((headers, nested_body)) = split_headers_and_body(part) else {
            continue;
        };
        let headers = parse_headers(headers);
        parts.push((headers, trim_multipart_part_body(nested_body)));
    }

    parts
}

fn trim_multipart_part_body(body: &str) -> &str {
    body.strip_suffix("\r\n")
        .or_else(|| body.strip_suffix('\n'))
        .unwrap_or(body)
}

fn parsed_attachment_from_part(
    headers: &[(String, String)],
    body: &str,
    content_type: &str,
    provider_attachment_id: String,
) -> ParsedEmailAttachment {
    let transfer_encoding = header_value(headers, "content-transfer-encoding").unwrap_or_default();
    ParsedEmailAttachment {
        provider_attachment_id,
        filename: attachment_filename(headers, content_type),
        content_type: non_empty_or_default(
            header_media_type(content_type),
            "application/octet-stream",
        ),
        disposition: parsed_attachment_disposition(headers),
        body_bytes: decode_transfer_bytes(body, &transfer_encoding),
    }
}

fn is_attachment_like_part(headers: &[(String, String)], content_type: &str) -> bool {
    match parsed_attachment_disposition(headers) {
        ParsedEmailAttachmentDisposition::Attachment => true,
        ParsedEmailAttachmentDisposition::Inline => {
            attachment_filename(headers, content_type).is_some()
        }
        ParsedEmailAttachmentDisposition::Unknown => {
            attachment_filename(headers, content_type).is_some()
        }
    }
}

fn parsed_attachment_disposition(headers: &[(String, String)]) -> ParsedEmailAttachmentDisposition {
    let content_disposition = header_value(headers, "content-disposition").unwrap_or_default();
    match content_disposition
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "attachment" => ParsedEmailAttachmentDisposition::Attachment,
        "inline" => ParsedEmailAttachmentDisposition::Inline,
        _ => ParsedEmailAttachmentDisposition::Unknown,
    }
}

fn attachment_filename(headers: &[(String, String)], content_type: &str) -> Option<String> {
    header_value(headers, "content-disposition")
        .and_then(|value| header_parameter(&value, "filename"))
        .or_else(|| header_parameter(content_type, "name"))
        .map(|value| decode_rfc2047_words(value.trim()))
        .and_then(|value| {
            let value = value.trim().to_owned();
            if value.is_empty() { None } else { Some(value) }
        })
}

fn header_media_type(value: &str) -> String {
    value
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

fn header_parameter(value: &str, parameter: &str) -> Option<String> {
    let mut continuation_segments = Vec::new();
    let mut plain_parameter = None;
    let mut encoded_parameter = None;

    for part in value.split(';').skip(1) {
        let Some((name, value)) = part.split_once('=') else {
            continue;
        };
        let name = name.trim();
        if name.eq_ignore_ascii_case(parameter) {
            plain_parameter = Some(unquote_header_parameter_value(value));
            continue;
        }
        if let Some(segment) = rfc2231_continuation_segment(name, parameter, value) {
            continuation_segments.push(segment);
            continue;
        }
        if let Some(base_name) = name.strip_suffix('*') {
            if base_name.eq_ignore_ascii_case(parameter) {
                encoded_parameter = Some(decode_rfc2231_parameter_value(value));
            }
        }
    }

    encoded_parameter
        .or_else(|| decode_rfc2231_continuation_segments(continuation_segments))
        .or(plain_parameter)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Rfc2231ContinuationSegment {
    index: usize,
    encoded: bool,
    value: String,
}

fn rfc2231_continuation_segment(
    name: &str,
    parameter: &str,
    value: &str,
) -> Option<Rfc2231ContinuationSegment> {
    if name.len() <= parameter.len() || !name[..parameter.len()].eq_ignore_ascii_case(parameter) {
        return None;
    }
    let rest = &name[parameter.len()..];
    let rest = rest.strip_prefix('*')?;
    let encoded = rest.ends_with('*');
    let index = rest.trim_end_matches('*').parse::<usize>().ok()?;

    Some(Rfc2231ContinuationSegment {
        index,
        encoded,
        value: unquote_header_parameter_value(value),
    })
}

fn decode_rfc2231_continuation_segments(
    mut segments: Vec<Rfc2231ContinuationSegment>,
) -> Option<String> {
    if segments.is_empty() {
        return None;
    }

    segments.sort_by_key(|segment| segment.index);
    if segments.first().map(|segment| segment.index) != Some(0) {
        return None;
    }

    let mut output = Vec::new();
    for (expected_index, segment) in segments.into_iter().enumerate() {
        if segment.index != expected_index {
            return None;
        }

        let value = if expected_index == 0 {
            rfc2231_continuation_payload(&segment.value)
        } else {
            segment.value.as_str()
        };
        if segment.encoded {
            output.extend(percent_decode_bytes(value));
        } else {
            output.extend_from_slice(value.as_bytes());
        }
    }

    Some(String::from_utf8_lossy(&output).into_owned())
}

fn rfc2231_continuation_payload(value: &str) -> &str {
    value
        .split_once('\'')
        .and_then(|(_, rest)| rest.split_once('\'').map(|(_, encoded)| encoded))
        .unwrap_or(value)
}

fn unquote_header_parameter_value(value: &str) -> String {
    let value = value.trim();
    let value = value
        .strip_prefix('"')
        .and_then(|stripped| stripped.strip_suffix('"'))
        .unwrap_or(value);
    let mut output = String::with_capacity(value.len());
    let mut escaped = false;

    for character in value.chars() {
        if escaped {
            output.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        output.push(character);
    }

    if escaped {
        output.push('\\');
    }

    output
}

fn decode_rfc2231_parameter_value(value: &str) -> String {
    let value = unquote_header_parameter_value(value);
    let encoded_value = value
        .split_once('\'')
        .and_then(|(_, rest)| rest.split_once('\'').map(|(_, encoded)| encoded))
        .unwrap_or(&value);

    String::from_utf8_lossy(&percent_decode_bytes(encoded_value)).into_owned()
}

fn percent_decode_bytes(value: &str) -> Vec<u8> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            if let (Some(high), Some(low)) = (bytes.get(index + 1), bytes.get(index + 2)) {
                if let (Some(high), Some(low)) = (hex_value(*high), hex_value(*low)) {
                    output.push((high << 4) | low);
                    index += 3;
                    continue;
                }
            }
        }
        output.push(bytes[index]);
        index += 1;
    }

    output
}

fn decode_transfer_body(body: &str, transfer_encoding: &str) -> String {
    String::from_utf8_lossy(&decode_transfer_bytes(body, transfer_encoding)).into_owned()
}

fn decode_transfer_bytes(body: &str, transfer_encoding: &str) -> Vec<u8> {
    match transfer_encoding.trim().to_ascii_lowercase().as_str() {
        "base64" => {
            let compact = body
                .chars()
                .filter(|character| !character.is_whitespace())
                .collect::<String>();
            BASE64_STANDARD
                .decode(compact)
                .unwrap_or_else(|_| body.as_bytes().to_vec())
        }
        "quoted-printable" => decode_quoted_printable_bytes(body),
        _ => body.as_bytes().to_vec(),
    }
}

fn decode_quoted_printable(input: &str) -> String {
    String::from_utf8_lossy(&decode_quoted_printable_bytes(input)).into_owned()
}

fn decode_quoted_printable_bytes(input: &str) -> Vec<u8> {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'=' {
            if bytes.get(index + 1) == Some(&b'\r') && bytes.get(index + 2) == Some(&b'\n') {
                index += 3;
                continue;
            }
            if bytes.get(index + 1) == Some(&b'\n') {
                index += 2;
                continue;
            }
            if let (Some(high), Some(low)) = (bytes.get(index + 1), bytes.get(index + 2)) {
                if let (Some(high), Some(low)) = (hex_value(*high), hex_value(*low)) {
                    output.push((high << 4) | low);
                    index += 3;
                    continue;
                }
            }
        }
        output.push(bytes[index]);
        index += 1;
    }

    output
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn decode_rfc2047_words(input: &str) -> String {
    let mut output = String::new();
    let mut rest = input;

    while let Some(start) = rest.find("=?") {
        output.push_str(&rest[..start]);
        let candidate = &rest[start + 2..];
        let Some(charset_end) = candidate.find('?') else {
            output.push_str(&rest[start..]);
            return output;
        };
        let candidate = &candidate[charset_end + 1..];
        let Some(encoding_end) = candidate.find('?') else {
            output.push_str(&rest[start..]);
            return output;
        };
        let encoding = &candidate[..encoding_end];
        let candidate = &candidate[encoding_end + 1..];
        let Some(encoded_end) = candidate.find("?=") else {
            output.push_str(&rest[start..]);
            return output;
        };
        let encoded = &candidate[..encoded_end];
        let decoded = match encoding.to_ascii_lowercase().as_str() {
            "b" => BASE64_STANDARD
                .decode(encoded)
                .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                .ok(),
            "q" => Some(decode_quoted_printable(&encoded.replace('_', " "))),
            _ => None,
        };

        if let Some(decoded) = decoded {
            output.push_str(&decoded);
        } else {
            output.push_str(
                &rest[start..start + 2 + charset_end + 1 + encoding_end + 1 + encoded_end + 2],
            );
        }
        rest = &candidate[encoded_end + 2..];
    }

    output.push_str(rest);
    output
}

fn split_address_list(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect()
}

fn strip_html_tags(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut inside_tag = false;
    for character in input.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => {
                inside_tag = false;
                output.push(' ');
            }
            _ if !inside_tag => output.push(character),
            _ => {}
        }
    }

    normalize_body_text(&output)
}

fn normalize_body_text(input: &str) -> String {
    input
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
}

fn non_empty_or_default(value: String, default: &str) -> String {
    let value = value.trim().to_owned();
    if value.is_empty() {
        default.to_owned()
    } else {
        value
    }
}

fn non_empty_recipients(recipients: Vec<String>) -> Vec<String> {
    if recipients.is_empty() {
        vec!["unknown@example.invalid".to_owned()]
    } else {
        recipients
    }
}

#[derive(Debug, Error)]
pub enum EmailRfc822ParseError {
    #[error("RFC822 message must contain headers and body")]
    MalformedRfc822,
}
