use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedEmailMessage {
    pub subject: String,
    pub from: String,
    pub to: Vec<String>,
    pub body_text: String,
}

pub fn parse_rfc822_message(raw: &[u8]) -> Result<ParsedEmailMessage, EmailRfc822ParseError> {
    let raw = String::from_utf8_lossy(raw);
    let (header_block, body) = split_headers_and_body(&raw)?;
    let headers = parse_headers(header_block);

    let subject = header_value(&headers, "subject").unwrap_or_else(|| "(no subject)".to_owned());
    let from =
        header_value(&headers, "from").unwrap_or_else(|| "unknown@example.invalid".to_owned());
    let to = split_address_list(&header_value(&headers, "to").unwrap_or_default());
    let body_text = body_text_from_part(&headers, body);

    Ok(ParsedEmailMessage {
        subject: non_empty_or_default(subject, "(no subject)"),
        from: non_empty_or_default(from, "unknown@example.invalid"),
        to: non_empty_recipients(to),
        body_text: non_empty_or_default(body_text, "(empty body)"),
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

fn body_text_from_part(headers: &[(String, String)], body: &str) -> String {
    let content_type = header_value(headers, "content-type").unwrap_or_default();
    if content_type.to_ascii_lowercase().starts_with("multipart/") {
        if let Some(boundary) = content_type_parameter(&content_type, "boundary") {
            if let Some(text) = first_text_plain_multipart_part(&boundary, body) {
                return text;
            }
        }
    }

    let decoded = decode_transfer_body(
        body,
        header_value(headers, "content-transfer-encoding")
            .unwrap_or_default()
            .as_str(),
    );
    if content_type.to_ascii_lowercase().starts_with("text/html") {
        return strip_html_tags(&decoded);
    }

    normalize_body_text(&decoded)
}

fn first_text_plain_multipart_part(boundary: &str, body: &str) -> Option<String> {
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
        let content_type = header_value(&headers, "content-type").unwrap_or_default();
        let content_disposition = header_value(&headers, "content-disposition").unwrap_or_default();
        let normalized_content_type = content_type.to_ascii_lowercase();
        let normalized_disposition = content_disposition.to_ascii_lowercase();
        if normalized_content_type.starts_with("text/plain")
            && !normalized_disposition.contains("attachment")
        {
            return Some(normalize_body_text(&decode_transfer_body(
                nested_body,
                header_value(&headers, "content-transfer-encoding")
                    .unwrap_or_default()
                    .as_str(),
            )));
        }
    }

    None
}

fn content_type_parameter(content_type: &str, parameter: &str) -> Option<String> {
    for part in content_type.split(';').skip(1) {
        let Some((name, value)) = part.split_once('=') else {
            continue;
        };
        if name.trim().eq_ignore_ascii_case(parameter) {
            return Some(value.trim().trim_matches('"').to_owned());
        }
    }

    None
}

fn decode_transfer_body(body: &str, transfer_encoding: &str) -> String {
    match transfer_encoding.trim().to_ascii_lowercase().as_str() {
        "base64" => {
            let compact = body
                .chars()
                .filter(|character| !character.is_whitespace())
                .collect::<String>();
            BASE64_STANDARD
                .decode(compact)
                .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                .unwrap_or_else(|_| body.to_owned())
        }
        "quoted-printable" => decode_quoted_printable(body),
        _ => body.to_owned(),
    }
}

fn decode_quoted_printable(input: &str) -> String {
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

    String::from_utf8_lossy(&output).into_owned()
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
