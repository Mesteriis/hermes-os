use super::decoding::{decode_transfer_body, decode_transfer_bytes};
use super::headers::{header_media_type, header_parameter, header_value};
use super::models::{ParsedEmailAttachment, ParsedEmailAttachmentDisposition};
use super::multipart::multipart_parts;
use super::util::non_empty_or_default;

#[derive(Default)]
pub(crate) struct ParsedEmailBodyContent {
    pub(crate) body_text: Option<String>,
    pub(crate) body_html: Option<String>,
    pub(crate) attachments: Vec<ParsedEmailAttachment>,
    next_attachment_index: usize,
}

pub(crate) fn body_content_from_part(
    headers: &[(String, String)],
    body: &[u8],
) -> ParsedEmailBodyContent {
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
        .map(|value| super::decoding::decode_rfc2047_words(value.trim()))
        .and_then(|value| {
            let value = value.trim().to_owned();
            if value.is_empty() { None } else { Some(value) }
        })
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
