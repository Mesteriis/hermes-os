use super::encoding::decode_transfer_body;
use super::headers::{content_type_parameter, header_value, parse_headers};
use super::rfc822::split_headers_and_body;
use super::text::normalize_body_text;

pub(super) fn body_text_from_part(headers: &[(String, String)], body: &str) -> String {
    let content_type = header_value(headers, "content-type").unwrap_or_default();
    if content_type.to_ascii_lowercase().starts_with("multipart/")
        && let Some(boundary) = content_type_parameter(&content_type, "boundary")
        && let Some(text) = first_text_plain_multipart_part(&boundary, body)
    {
        return text;
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
        if is_text_plain_non_attachment(&headers) {
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

fn is_text_plain_non_attachment(headers: &[(String, String)]) -> bool {
    let content_type = header_value(headers, "content-type").unwrap_or_default();
    let content_disposition = header_value(headers, "content-disposition").unwrap_or_default();
    let normalized_content_type = content_type.to_ascii_lowercase();
    let normalized_disposition = content_disposition.to_ascii_lowercase();
    normalized_content_type.starts_with("text/plain")
        && !normalized_disposition.contains("attachment")
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
