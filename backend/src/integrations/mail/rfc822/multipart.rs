use super::headers::parse_headers;
use super::wire::{find_subslice, next_line_start, split_headers_and_body};

type MimeHeaders = Vec<(String, String)>;
type MimePart<'a> = (MimeHeaders, &'a [u8]);

pub(crate) fn multipart_parts<'a>(boundary: &str, body: &'a [u8]) -> Vec<MimePart<'a>> {
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
