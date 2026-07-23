//! Bounded RFC822/MIME extraction shared by Mail adapters.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use hermes_mail_api::MAX_PLAIN_TEXT_BYTES;

const MAX_RFC822_BYTES: usize = 16 * 1024 * 1024;
const MAX_MIME_DEPTH: u8 = 8;
const MAX_MIME_PARTS: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttachmentDispositionV1 {
    Attachment,
    Inline,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentMetadataV1 {
    pub part_id: u16,
    pub filename: Option<String>,
    pub media_type: String,
    pub declared_bytes: u64,
    pub disposition: AttachmentDispositionV1,
}

/// Extracts the first bounded `text/plain` MIME leaf that is not an attachment.
/// Malformed, oversized and unsupported encodings are rejected rather than letting
/// raw RFC822 or attachment bytes enter the Communications body pipeline.
pub fn direct_plain_text_body(raw_message: &[u8]) -> Option<Vec<u8>> {
    if raw_message.is_empty() || raw_message.len() > MAX_RFC822_BYTES {
        return None;
    }
    let (headers, body) = split_headers_and_body(raw_message)?;
    extract_plain_text_leaf(headers, body, 0, &mut 0)
}

fn extract_plain_text_leaf(
    headers: &[u8],
    body: &[u8],
    depth: u8,
    parts: &mut usize,
) -> Option<Vec<u8>> {
    if depth > MAX_MIME_DEPTH || *parts >= MAX_MIME_PARTS || is_attachment(headers) {
        return None;
    }
    let content_type =
        header_value(headers, "content-type").unwrap_or_else(|| "text/plain".to_owned());
    let media_type = content_type.split(';').next()?.trim().to_ascii_lowercase();
    if media_type.starts_with("multipart/") {
        let boundary = header_parameter(headers, "content-type", "boundary")?;
        for part in multipart_parts(body, &boundary)? {
            *parts += 1;
            let Some((part_headers, part_body)) = split_headers_and_body(&part) else {
                continue;
            };
            if let Some(plaintext) =
                extract_plain_text_leaf(part_headers, part_body, depth + 1, parts)
            {
                return Some(plaintext);
            }
        }
        return None;
    }
    if media_type != "text/plain" {
        return None;
    }
    let decoded = decode_transfer_encoding(
        body,
        header_value(headers, "content-transfer-encoding").unwrap_or_default(),
    )?;
    valid_plaintext(decoded)
}

fn is_attachment(headers: &[u8]) -> bool {
    let disposition = header_value(headers, "content-disposition").unwrap_or_default();
    disposition
        .split(';')
        .next()
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("attachment"))
        || header_parameter(headers, "content-type", "name").is_some()
        || header_parameter(headers, "content-disposition", "filename").is_some()
}

fn multipart_parts(body: &[u8], boundary: &str) -> Option<Vec<Vec<u8>>> {
    if boundary.is_empty() || boundary.len() > 200 || !boundary.is_ascii() {
        return None;
    }
    let marker = format!("--{boundary}");
    let closing = format!("{marker}--");
    let mut parts = Vec::new();
    let mut current = Vec::new();
    let mut inside_part = false;
    for line in body.split_inclusive(|byte| *byte == b'\n') {
        let normalized = line
            .strip_suffix(b"\n")
            .unwrap_or(line)
            .strip_suffix(b"\r")
            .unwrap_or(line);
        if normalized == marker.as_bytes() || normalized == closing.as_bytes() {
            if inside_part && !current.is_empty() {
                if parts.len() >= MAX_MIME_PARTS {
                    return None;
                }
                parts.push(std::mem::take(&mut current));
            }
            if normalized == closing.as_bytes() {
                return Some(parts);
            }
            inside_part = true;
        } else if inside_part {
            if current.len().saturating_add(line.len()) > MAX_RFC822_BYTES {
                return None;
            }
            current.extend_from_slice(line);
        }
    }
    None
}

fn decode_transfer_encoding(body: &[u8], encoding: String) -> Option<Vec<u8>> {
    match encoding.trim().to_ascii_lowercase().as_str() {
        "" | "7bit" | "8bit" | "binary" => {
            (body.len() <= MAX_PLAIN_TEXT_BYTES).then(|| body.to_vec())
        }
        "base64" => decode_base64(body),
        "quoted-printable" => decode_quoted_printable(body),
        _ => None,
    }
}

fn decode_base64(body: &[u8]) -> Option<Vec<u8>> {
    let compact = body
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    if compact.is_empty() || compact.len() > MAX_PLAIN_TEXT_BYTES.saturating_mul(2) {
        return None;
    }
    let decoded = STANDARD.decode(compact).ok()?;
    (decoded.len() <= MAX_PLAIN_TEXT_BYTES).then_some(decoded)
}

fn decode_quoted_printable(body: &[u8]) -> Option<Vec<u8>> {
    let mut decoded = Vec::with_capacity(body.len().min(MAX_PLAIN_TEXT_BYTES));
    let mut index = 0;
    while index < body.len() {
        if body[index] != b'=' {
            decoded.push(body[index]);
            index += 1;
        } else if body.get(index + 1) == Some(&b'\r') && body.get(index + 2) == Some(&b'\n') {
            index += 3;
        } else if body.get(index + 1) == Some(&b'\n') {
            index += 2;
        } else {
            let value =
                (hex_value(*body.get(index + 1)?)? << 4) | hex_value(*body.get(index + 2)?)?;
            decoded.push(value);
            index += 3;
        }
        if decoded.len() > MAX_PLAIN_TEXT_BYTES {
            return None;
        }
    }
    Some(decoded)
}

const fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn valid_plaintext(mut body: Vec<u8>) -> Option<Vec<u8>> {
    while matches!(body.last(), Some(b'\r' | b'\n')) {
        body.pop();
    }
    (!body.is_empty() && body.len() <= MAX_PLAIN_TEXT_BYTES && std::str::from_utf8(&body).is_ok())
        .then_some(body)
}

pub fn attachment_metadata(raw_message: &[u8]) -> Vec<AttachmentMetadataV1> {
    let Some((headers, body)) = split_headers_and_body(raw_message) else {
        return Vec::new();
    };
    let Some(boundary) = header_parameter(headers, "content-type", "boundary") else {
        return Vec::new();
    };
    let content_type = header_value(headers, "content-type").unwrap_or_default();
    if !content_type
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("multipart/")
    {
        return Vec::new();
    }
    let mut attachments = Vec::new();
    let mut next_part_id = 1_u16;
    collect_multipart_attachments(body, &boundary, 0, &mut next_part_id, &mut attachments);
    attachments
}

fn collect_multipart_attachments(
    body: &[u8],
    boundary: &str,
    depth: u8,
    next_part_id: &mut u16,
    attachments: &mut Vec<AttachmentMetadataV1>,
) {
    if depth >= MAX_MIME_DEPTH
        || boundary.is_empty()
        || boundary.len() > 200
        || !boundary.is_ascii()
    {
        return;
    }
    let marker = format!("--{boundary}");
    let closing_marker = format!("{marker}--");
    let mut current = Vec::new();
    let mut inside_part = false;
    for line in body.split(|byte| *byte == b'\n') {
        let normalized = line.strip_suffix(b"\r").unwrap_or(line);
        if normalized == marker.as_bytes() || normalized == closing_marker.as_bytes() {
            if inside_part {
                collect_attachment_from_part(&current, depth, next_part_id, attachments);
                current.clear();
            }
            if normalized == closing_marker.as_bytes() {
                break;
            }
            inside_part = true;
        } else if inside_part {
            current.extend_from_slice(line);
            current.push(b'\n');
        }
    }
}

fn collect_attachment_from_part(
    part: &[u8],
    depth: u8,
    next_part_id: &mut u16,
    attachments: &mut Vec<AttachmentMetadataV1>,
) {
    let Some((headers, body)) = split_headers_and_body(part) else {
        return;
    };
    let content_type = header_value(headers, "content-type").unwrap_or_default();
    if content_type
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("multipart/")
    {
        if let Some(boundary) = header_parameter(headers, "content-type", "boundary") {
            collect_multipart_attachments(body, &boundary, depth + 1, next_part_id, attachments);
        }
        return;
    }
    let Some(disposition) = header_value(headers, "content-disposition") else {
        return;
    };
    let disposition = match disposition
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "attachment" => AttachmentDispositionV1::Attachment,
        "inline" => AttachmentDispositionV1::Inline,
        _ => return,
    };
    let Some(media_type) = content_type
        .split(';')
        .next()
        .map(str::trim)
        .filter(valid_media_type)
    else {
        return;
    };
    let Some(declared_bytes) =
        decoded_part_size(body, header_value(headers, "content-transfer-encoding"))
    else {
        return;
    };
    let filename = header_parameter(headers, "content-disposition", "filename")
        .or_else(|| header_parameter(headers, "content-type", "name"))
        .filter(|value| !value.is_empty() && value.len() <= 512 && value.is_ascii());
    let part_id = *next_part_id;
    let Some(next) = next_part_id.checked_add(1) else {
        return;
    };
    *next_part_id = next;
    attachments.push(AttachmentMetadataV1 {
        part_id,
        filename,
        media_type: media_type.to_owned(),
        declared_bytes,
        disposition,
    });
}

fn split_headers_and_body(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| (&bytes[..index], &bytes[index + 4..]))
        .or_else(|| {
            bytes
                .windows(2)
                .position(|window| window == b"\n\n")
                .map(|index| (&bytes[..index], &bytes[index + 2..]))
        })
}

fn header_value(headers: &[u8], name: &str) -> Option<String> {
    let text = std::str::from_utf8(headers).ok()?;
    text.lines()
        .filter_map(|line| line.split_once(':'))
        .filter(|(key, _)| key.trim().eq_ignore_ascii_case(name))
        .map(|(_, value)| value.trim().to_owned())
        .next_back()
}

fn header_parameter(headers: &[u8], header: &str, parameter: &str) -> Option<String> {
    header_value(headers, header)?
        .split(';')
        .skip(1)
        .find_map(|part| {
            let (key, value) = part.trim().split_once('=')?;
            key.trim()
                .eq_ignore_ascii_case(parameter)
                .then(|| value.trim().trim_matches('"').to_owned())
        })
}

fn valid_media_type(value: &&str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.is_ascii()
        && value.contains('/')
        && !value.contains(char::is_whitespace)
}

fn decoded_part_size(body: &[u8], transfer_encoding: Option<String>) -> Option<u64> {
    let body = body
        .strip_suffix(b"\n")
        .unwrap_or(body)
        .strip_suffix(b"\r")
        .unwrap_or(body);
    match transfer_encoding
        .as_deref()
        .map(str::trim)
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        None | Some("") | Some("7bit") | Some("8bit") | Some("binary") => {
            u64::try_from(body.len()).ok()
        }
        Some("base64") => base64_decoded_size(body),
        _ => None,
    }
}

fn base64_decoded_size(body: &[u8]) -> Option<u64> {
    let bytes = body
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    if bytes.is_empty() || bytes.len() % 4 != 0 {
        return None;
    }
    let padding = bytes.iter().rev().take_while(|byte| **byte == b'=').count();
    if padding > 2
        || bytes[..bytes.len() - padding]
            .iter()
            .any(|byte| !byte.is_ascii_alphanumeric() && !matches!(*byte, b'+' | b'/'))
        || bytes[..bytes.len() - padding].contains(&b'=')
    {
        return None;
    }
    u64::try_from((bytes.len() / 4) * 3 - padding).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_bounded_attachment_metadata() {
        let raw = b"Content-Type: multipart/mixed; boundary=x\r\n\r\n--x\r\nContent-Type: text/plain\r\n\r\nbody\r\n--x\r\nContent-Type: application/pdf; name=a.pdf\r\nContent-Disposition: attachment; filename=a.pdf\r\nContent-Transfer-Encoding: base64\r\n\r\naGVsbG8=\r\n--x--\r\n";
        assert_eq!(attachment_metadata(raw)[0].declared_bytes, 5);
    }

    #[test]
    fn extracts_base64_plain_text_from_multipart() {
        let message = b"Content-Type: multipart/mixed; boundary=x\r\n\r\n--x\r\nContent-Type: text/plain\r\nContent-Transfer-Encoding: base64\r\n\r\naGVsbG8=\r\n--x\r\nContent-Type: application/pdf\r\nContent-Disposition: attachment\r\n\r\nnot-content\r\n--x--\r\n";
        assert_eq!(direct_plain_text_body(message), Some(b"hello".to_vec()));
    }

    #[test]
    fn decodes_quoted_printable_and_skips_attachment_text() {
        let message = b"Content-Type: multipart/alternative; boundary=x\r\n\r\n--x\r\nContent-Type: text/plain\r\nContent-Disposition: attachment\r\n\r\nignore\r\n--x\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: quoted-printable\r\n\r\nhello=20world=21\r\n--x--\r\n";
        assert_eq!(
            direct_plain_text_body(message),
            Some(b"hello world!".to_vec())
        );
    }
}
