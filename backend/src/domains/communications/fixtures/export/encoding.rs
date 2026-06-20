use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

pub(super) fn decode_transfer_body(body: &str, transfer_encoding: &str) -> String {
    match transfer_encoding.trim().to_ascii_lowercase().as_str() {
        "base64" => decode_base64_body(body),
        "quoted-printable" => decode_quoted_printable(body),
        _ => body.to_owned(),
    }
}

fn decode_base64_body(body: &str) -> String {
    let compact = body
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    BASE64_STANDARD
        .decode(compact)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_else(|_| body.to_owned())
}

pub(super) fn decode_quoted_printable(input: &str) -> String {
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
            if let (Some(high), Some(low)) = (bytes.get(index + 1), bytes.get(index + 2))
                && let (Some(high), Some(low)) = (hex_value(*high), hex_value(*low))
            {
                output.push((high << 4) | low);
                index += 3;
                continue;
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
