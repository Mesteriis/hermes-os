use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use encoding_rs::{Encoding, UTF_8};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedEmailMessage {
    pub subject: String,
    pub from: String,
    pub to: Vec<String>,
    pub headers: Vec<(String, String)>,
    pub body_text: String,
    pub body_html: Option<String>,
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

fn split_headers_and_body(raw: &[u8]) -> Result<(&[u8], &[u8]), EmailRfc822ParseError> {
    if let Some(separator_start) = find_subslice(raw, b"\r\n\r\n") {
        return Ok((
            &raw[..separator_start],
            &raw[separator_start + b"\r\n\r\n".len()..],
        ));
    }
    if let Some(separator_start) = find_subslice(raw, b"\n\n") {
        return Ok((
            &raw[..separator_start],
            &raw[separator_start + b"\n\n".len()..],
        ));
    }

    Err(EmailRfc822ParseError::MalformedRfc822)
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }

    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn next_line_start(bytes: &[u8], start: usize) -> Option<usize> {
    bytes[start..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map(|line_end| start + line_end + 1)
}

fn parse_headers(header_block: &[u8]) -> Vec<(String, String)> {
    let mut raw_headers: Vec<(String, Vec<u8>)> = Vec::new();

    for line in header_block.split(|byte| *byte == b'\n') {
        let line = strip_trailing_cr(line);
        if line.starts_with(b" ") || line.starts_with(b"\t") {
            if let Some((_, value)) = raw_headers.last_mut() {
                value.push(b' ');
                value.extend_from_slice(trim_ascii_whitespace(line));
            }
            continue;
        }

        if let Some(separator_index) = line.iter().position(|byte| *byte == b':') {
            let name = decode_ascii_header_name(trim_ascii_whitespace(&line[..separator_index]));
            let value = trim_ascii_whitespace(&line[separator_index + 1..]).to_vec();
            headers_push_if_valid(&mut raw_headers, name, value);
        }
    }

    raw_headers
        .into_iter()
        .map(|(name, value)| (name, decode_header_value_bytes(&value)))
        .collect()
}

fn headers_push_if_valid(headers: &mut Vec<(String, Vec<u8>)>, name: String, value: Vec<u8>) {
    if !name.is_empty() {
        headers.push((name, value));
    }
}

fn strip_trailing_cr(line: &[u8]) -> &[u8] {
    line.strip_suffix(b"\r").unwrap_or(line)
}

fn trim_ascii_whitespace(value: &[u8]) -> &[u8] {
    let start = value
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(value.len());
    let end = value
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map(|index| index + 1)
        .unwrap_or(start);
    &value[start..end]
}

fn decode_ascii_header_name(value: &[u8]) -> String {
    String::from_utf8_lossy(value).trim().to_owned()
}

fn decode_header_value_bytes(value: &[u8]) -> String {
    decode_rfc2047_words(decode_text_bytes(value, None).trim())
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
    body_html: Option<String>,
    attachments: Vec<ParsedEmailAttachment>,
    next_attachment_index: usize,
}

fn body_content_from_part(headers: &[(String, String)], body: &[u8]) -> ParsedEmailBodyContent {
    let mut content = ParsedEmailBodyContent::default();
    collect_part_content(headers, body, &mut content);
    content
}

fn collect_part_content(
    headers: &[(String, String)],
    body: &[u8],
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

    let charset = header_parameter(&content_type, "charset");
    let decoded = decode_transfer_body(
        body,
        header_value(headers, "content-transfer-encoding")
            .unwrap_or_default()
            .as_str(),
        charset.as_deref(),
    );
    if content_type_media_type == "text/html" {
        if content.body_html.is_none() {
            content.body_html = non_empty_html_body(&decoded);
        }
        if content.body_text.is_none() {
            content.body_text = Some(strip_html_tags(&decoded));
        }
        return;
    }
    if content.body_text.is_none()
        && (content_type_media_type == "text/plain" || content_type_media_type.is_empty())
    {
        content.body_text = Some(normalize_body_text(&decoded));
    }
}

type MimeHeaders = Vec<(String, String)>;
type MimePart<'a> = (MimeHeaders, &'a [u8]);

fn multipart_parts<'a>(boundary: &str, body: &'a [u8]) -> Vec<MimePart<'a>> {
    let mut parts = Vec::new();
    let delimiter = format!("--{boundary}").into_bytes();
    let mut cursor = 0;
    let mut current_part_start = None;

    while let Some(relative_start) = find_subslice(&body[cursor..], &delimiter) {
        let boundary_start = cursor + relative_start;
        if boundary_start > 0 && body[boundary_start - 1] != b'\n' {
            cursor = boundary_start + delimiter.len();
            continue;
        }

        if let Some(part_start) = current_part_start {
            let raw_part = trim_multipart_part_body(&body[part_start..boundary_start]);
            if let Ok((headers, nested_body)) = split_headers_and_body(raw_part) {
                let headers = parse_headers(headers);
                parts.push((headers, nested_body));
            }
        }

        let after_delimiter = boundary_start + delimiter.len();
        if body.get(after_delimiter..after_delimiter + 2) == Some(b"--") {
            break;
        }

        let Some(next_line_start) = next_line_start(body, after_delimiter) else {
            break;
        };
        current_part_start = Some(next_line_start);
        cursor = next_line_start;
    }

    parts
}

fn trim_multipart_part_body(body: &[u8]) -> &[u8] {
    body.strip_suffix(b"\r\n")
        .or_else(|| body.strip_suffix(b"\n"))
        .unwrap_or(body)
}

fn parsed_attachment_from_part(
    headers: &[(String, String)],
    body: &[u8],
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
        if let Some(base_name) = name.strip_suffix('*')
            && base_name.eq_ignore_ascii_case(parameter)
        {
            encoded_parameter = Some(decode_rfc2231_parameter_value(value));
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
    let mut charset = None;
    for (expected_index, segment) in segments.into_iter().enumerate() {
        if segment.index != expected_index {
            return None;
        }

        let value = if expected_index == 0 {
            let (segment_charset, payload) = rfc2231_charset_and_payload(&segment.value);
            charset = segment_charset.map(str::to_owned);
            payload
        } else {
            segment.value.as_str()
        };
        if segment.encoded {
            output.extend(percent_decode_bytes(value));
        } else {
            output.extend_from_slice(value.as_bytes());
        }
    }

    Some(decode_text_bytes(&output, charset.as_deref()))
}

fn rfc2231_charset_and_payload(value: &str) -> (Option<&str>, &str) {
    let Some((charset, rest)) = value.split_once('\'') else {
        return (None, value);
    };
    let Some((_, encoded)) = rest.split_once('\'') else {
        return (None, value);
    };
    let charset = charset.trim();
    let charset = if charset.is_empty() {
        None
    } else {
        Some(charset)
    };
    (charset, encoded)
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
    let (charset, encoded_value) = rfc2231_charset_and_payload(&value);

    decode_text_bytes(&percent_decode_bytes(encoded_value), charset)
}

fn percent_decode_bytes(value: &str) -> Vec<u8> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%'
            && let (Some(high), Some(low)) = (bytes.get(index + 1), bytes.get(index + 2))
            && let (Some(high), Some(low)) = (hex_value(*high), hex_value(*low))
        {
            output.push((high << 4) | low);
            index += 3;
            continue;
        }
        output.push(bytes[index]);
        index += 1;
    }

    output
}

fn decode_transfer_body(body: &[u8], transfer_encoding: &str, charset: Option<&str>) -> String {
    decode_text_bytes(&decode_transfer_bytes(body, transfer_encoding), charset)
}

fn decode_transfer_bytes(body: &[u8], transfer_encoding: &str) -> Vec<u8> {
    match transfer_encoding.trim().to_ascii_lowercase().as_str() {
        "base64" => {
            let compact = body
                .iter()
                .copied()
                .filter(|byte| !byte.is_ascii_whitespace())
                .collect::<Vec<_>>();
            BASE64_STANDARD
                .decode(compact)
                .unwrap_or_else(|_| body.to_vec())
        }
        "quoted-printable" => decode_quoted_printable_bytes(body),
        _ => body.to_vec(),
    }
}

fn decode_quoted_printable(input: &str, charset: Option<&str>) -> String {
    decode_text_bytes(&decode_quoted_printable_bytes(input.as_bytes()), charset)
}

fn decode_quoted_printable_bytes(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    let mut index = 0;

    while index < input.len() {
        if input[index] == b'=' {
            if input.get(index + 1) == Some(&b'\r') && input.get(index + 2) == Some(&b'\n') {
                index += 3;
                continue;
            }
            if input.get(index + 1) == Some(&b'\n') {
                index += 2;
                continue;
            }
            if let (Some(high), Some(low)) = (input.get(index + 1), input.get(index + 2))
                && let (Some(high), Some(low)) = (hex_value(*high), hex_value(*low))
            {
                output.push((high << 4) | low);
                index += 3;
                continue;
            }
        }
        output.push(input[index]);
        index += 1;
    }

    output
}

fn decode_text_bytes(bytes: &[u8], charset: Option<&str>) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    let primary_encoding = charset
        .and_then(|label| Encoding::for_label(label.trim().as_bytes()))
        .unwrap_or(UTF_8);
    let primary = decode_with_encoding(bytes, primary_encoding);
    if charset.is_some() && !primary.had_errors {
        return primary.text;
    }
    if charset.is_none() && !primary.had_errors {
        return primary.text;
    }

    legacy_text_candidates(bytes, primary)
        .into_iter()
        .max_by_key(score_decoded_text)
        .map(|candidate| candidate.text)
        .unwrap_or_else(|| String::from_utf8_lossy(bytes).into_owned())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DecodedTextCandidate {
    text: String,
    had_errors: bool,
    fallback_rank: i64,
}

fn decode_with_encoding(bytes: &[u8], encoding: &'static Encoding) -> DecodedTextCandidate {
    let (text, _, had_errors) = encoding.decode(bytes);
    DecodedTextCandidate {
        text: text.into_owned(),
        had_errors,
        fallback_rank: 0,
    }
}

fn legacy_text_candidates(
    bytes: &[u8],
    primary: DecodedTextCandidate,
) -> Vec<DecodedTextCandidate> {
    let mut candidates = vec![primary];
    for (fallback_rank, label) in [
        "windows-1251",
        "koi8-r",
        "iso-8859-5",
        "windows-1252",
        "iso-8859-1",
    ]
    .iter()
    .enumerate()
    {
        let Some(encoding) = Encoding::for_label(label.as_bytes()) else {
            continue;
        };
        let mut candidate = decode_with_encoding(bytes, encoding);
        candidate.fallback_rank = fallback_rank as i64 + 1;
        candidates.push(candidate);
    }
    candidates
}

fn score_decoded_text(candidate: &DecodedTextCandidate) -> i64 {
    let mut replacement_count = 0;
    let mut disallowed_control_count = 0;
    let mut cyrillic_count = 0;
    let mut printable_count = 0;

    for character in candidate.text.chars() {
        if character == '\u{fffd}' {
            replacement_count += 1;
        } else if character.is_control()
            && character != '\n'
            && character != '\r'
            && character != '\t'
        {
            disallowed_control_count += 1;
        } else if ('\u{0400}'..='\u{04ff}').contains(&character) {
            cyrillic_count += 1;
            printable_count += 1;
        } else if !character.is_control() {
            printable_count += 1;
        }
    }

    printable_count + (cyrillic_count * 8)
        - (replacement_count * 1_000)
        - (disallowed_control_count * 100)
        - candidate.fallback_rank
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
        let charset = &candidate[..charset_end];
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
                .map(|bytes| decode_text_bytes(&bytes, Some(charset)))
                .ok(),
            "q" => Some(decode_quoted_printable(
                &encoded.replace('_', " "),
                Some(charset),
            )),
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

fn non_empty_html_body(input: &str) -> Option<String> {
    let value = input.trim().to_owned();
    if value.is_empty() { None } else { Some(value) }
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
