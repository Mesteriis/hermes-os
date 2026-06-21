use super::errors::EmailRfc822ParseError;

pub(crate) fn split_headers_and_body(raw: &[u8]) -> Result<(&[u8], &[u8]), EmailRfc822ParseError> {
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

pub(crate) fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }

    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

pub(crate) fn next_line_start(bytes: &[u8], start: usize) -> Option<usize> {
    bytes[start..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map(|line_end| start + line_end + 1)
}

pub(crate) fn strip_trailing_cr(line: &[u8]) -> &[u8] {
    line.strip_suffix(b"\r").unwrap_or(line)
}

pub(crate) fn trim_ascii_whitespace(value: &[u8]) -> &[u8] {
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
