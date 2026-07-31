use std::time::Duration;

use async_std::{
    future::timeout,
    io::{ReadExt, WriteExt},
    net::TcpStream,
};
use zeroize::Zeroizing;

use crate::model::OllamaAiHttpErrorV1;

const LOOPBACK_HOST: &str = "127.0.0.1";
const MAX_RESPONSE_BYTES: usize = 131_072;
const MAX_HEADER_BYTES: usize = 16_384;

pub(crate) async fn execute_json_v1(
    port: u16,
    method: &str,
    path: &str,
    body: &[u8],
    request_timeout: Duration,
) -> Result<Zeroizing<Vec<u8>>, OllamaAiHttpErrorV1> {
    if !matches!((method, path), ("GET", "/api/tags") | ("POST", "/api/chat"))
        || body.len() > MAX_RESPONSE_BYTES
    {
        return Err(OllamaAiHttpErrorV1::InvalidRequest);
    }
    timeout(request_timeout, execute_once_v1(port, method, path, body))
        .await
        .map_err(|_| OllamaAiHttpErrorV1::Unavailable)?
}

async fn execute_once_v1(
    port: u16,
    method: &str,
    path: &str,
    body: &[u8],
) -> Result<Zeroizing<Vec<u8>>, OllamaAiHttpErrorV1> {
    let mut stream = TcpStream::connect((LOOPBACK_HOST, port))
        .await
        .map_err(|_| OllamaAiHttpErrorV1::Unavailable)?;
    let mut request = Zeroizing::new(
        format!(
            "{method} {path} HTTP/1.1\r\nHost: {LOOPBACK_HOST}:{port}\r\nAccept: application/json\r\nAccept-Encoding: identity\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len(),
        )
        .into_bytes(),
    );
    request.extend_from_slice(body);
    stream
        .write_all(&request)
        .await
        .map_err(|_| OllamaAiHttpErrorV1::Unavailable)?;
    stream
        .flush()
        .await
        .map_err(|_| OllamaAiHttpErrorV1::Unavailable)?;
    let mut response = Zeroizing::new(Vec::new());
    stream
        .take((MAX_RESPONSE_BYTES + MAX_HEADER_BYTES + 1) as u64)
        .read_to_end(&mut response)
        .await
        .map_err(|_| OllamaAiHttpErrorV1::Unavailable)?;
    if response.len() > MAX_RESPONSE_BYTES + MAX_HEADER_BYTES {
        return Err(OllamaAiHttpErrorV1::Protocol);
    }
    decode_response_v1(&response)
}

fn decode_response_v1(bytes: &[u8]) -> Result<Zeroizing<Vec<u8>>, OllamaAiHttpErrorV1> {
    let split = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or(OllamaAiHttpErrorV1::Protocol)?;
    if split > MAX_HEADER_BYTES {
        return Err(OllamaAiHttpErrorV1::Protocol);
    }
    let header = std::str::from_utf8(&bytes[..split]).map_err(|_| OllamaAiHttpErrorV1::Protocol)?;
    let status = header
        .lines()
        .next()
        .and_then(|line| {
            let mut fields = line.split_whitespace();
            (fields.next() == Some("HTTP/1.1"))
                .then(|| fields.next())
                .flatten()
        })
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or(OllamaAiHttpErrorV1::Protocol)?;
    if (300..400).contains(&status) {
        return Err(OllamaAiHttpErrorV1::Rejected);
    }
    if !(200..300).contains(&status) {
        return Err(OllamaAiHttpErrorV1::Rejected);
    }
    let body = &bytes[split + 4..];
    let decoded = if header
        .lines()
        .any(|line| line.eq_ignore_ascii_case("transfer-encoding: chunked"))
    {
        decode_chunked_v1(body)?
    } else {
        let length = content_length_v1(header)?;
        if body.len() != length {
            return Err(OllamaAiHttpErrorV1::Protocol);
        }
        Zeroizing::new(body.to_vec())
    };
    if decoded.is_empty() || decoded.len() > MAX_RESPONSE_BYTES {
        return Err(OllamaAiHttpErrorV1::Protocol);
    }
    Ok(decoded)
}

fn content_length_v1(header: &str) -> Result<usize, OllamaAiHttpErrorV1> {
    let mut values = header.lines().filter_map(|line| {
        line.split_once(':').and_then(|(name, value)| {
            name.eq_ignore_ascii_case("content-length")
                .then_some(value.trim())
        })
    });
    let length = values
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .ok_or(OllamaAiHttpErrorV1::Protocol)?;
    if values.next().is_some() || length > MAX_RESPONSE_BYTES {
        return Err(OllamaAiHttpErrorV1::Protocol);
    }
    Ok(length)
}

fn decode_chunked_v1(mut bytes: &[u8]) -> Result<Zeroizing<Vec<u8>>, OllamaAiHttpErrorV1> {
    let mut decoded = Zeroizing::new(Vec::new());
    loop {
        let line_end = bytes
            .windows(2)
            .position(|window| window == b"\r\n")
            .ok_or(OllamaAiHttpErrorV1::Protocol)?;
        let line =
            std::str::from_utf8(&bytes[..line_end]).map_err(|_| OllamaAiHttpErrorV1::Protocol)?;
        if line.contains(';') {
            return Err(OllamaAiHttpErrorV1::Protocol);
        }
        let size = usize::from_str_radix(line, 16).map_err(|_| OllamaAiHttpErrorV1::Protocol)?;
        bytes = &bytes[line_end + 2..];
        if size == 0 {
            return (bytes == b"\r\n")
                .then_some(decoded)
                .ok_or(OllamaAiHttpErrorV1::Protocol);
        }
        if size > MAX_RESPONSE_BYTES.saturating_sub(decoded.len())
            || bytes.len() < size + 2
            || &bytes[size..size + 2] != b"\r\n"
        {
            return Err(OllamaAiHttpErrorV1::Protocol);
        }
        decoded.extend_from_slice(&bytes[..size]);
        bytes = &bytes[size + 2..];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_parser_requires_exact_framing() {
        assert_eq!(
            decode_response_v1(
                b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}"
            )
            .as_ref()
            .map(|body| body.as_slice()),
            Ok(b"{}".as_slice())
        );
        assert!(
            decode_response_v1(
                b"HTTP/1.1 302 Found\r\nContent-Length: 2\r\nLocation: http://example.test\r\n\r\n{}"
            )
            .is_err()
        );
    }
}
