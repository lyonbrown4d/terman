use std::{io, path::PathBuf};

use encoding_rs::Encoding;

pub(super) struct BufferIoSpec {
    pub(super) path: Option<PathBuf>,
    pub(super) encoding: Option<&'static Encoding>,
}

pub(super) fn parse_buffer_io_spec(payload: &str) -> io::Result<BufferIoSpec> {
    let payload = payload.trim();
    let Some(rest) = payload.strip_prefix("-e") else {
        return Ok(BufferIoSpec {
            path: non_empty_path(payload),
            encoding: None,
        });
    };

    let (label, remainder) = split_first_token(rest)?;
    Ok(BufferIoSpec {
        path: non_empty_path(remainder),
        encoding: Some(parse_encoding(label)?),
    })
}

pub(super) fn decode_buffer_bytes(
    bytes: Vec<u8>,
    encoding: Option<&'static Encoding>,
) -> Vec<u8> {
    let Some(encoding) = encoding else {
        return bytes;
    };
    let (text, _, _) = encoding.decode(&bytes);
    text.into_owned().into_bytes()
}

pub(super) fn encode_buffer_bytes(bytes: &[u8], encoding: Option<&'static Encoding>) -> Vec<u8> {
    let Some(encoding) = encoding else {
        return bytes.to_vec();
    };
    let text = String::from_utf8_lossy(bytes);
    let (encoded, _, _) = encoding.encode(&text);
    encoded.into_owned()
}

fn split_first_token(input: &str) -> io::Result<(&str, &str)> {
    let input = input.trim_start();
    if input.is_empty() {
        return Err(invalid_encoding());
    }
    let token_end = input.find(char::is_whitespace).unwrap_or(input.len());
    Ok((&input[..token_end], input[token_end..].trim_start()))
}

fn parse_encoding(label: &str) -> io::Result<&'static Encoding> {
    Encoding::for_label(label.as_bytes()).ok_or_else(invalid_encoding)
}

fn non_empty_path(path: &str) -> Option<PathBuf> {
    let path = path.trim();
    (!path.is_empty()).then(|| PathBuf::from(path))
}

fn invalid_encoding() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_buffer_encoding_required_hint(),
    )
}

#[cfg(test)]
mod tests {
    use super::{decode_buffer_bytes, encode_buffer_bytes, parse_buffer_io_spec};

    #[test]
    fn parses_optional_encoding_and_path() {
        let spec = parse_buffer_io_spec("-e windows-1252 exchange.txt").unwrap();

        assert_eq!(spec.path.unwrap().to_string_lossy(), "exchange.txt");
        assert_eq!(spec.encoding.unwrap().name(), "windows-1252");
    }

    #[test]
    fn parses_inline_encoding_without_path() {
        let spec = parse_buffer_io_spec("-eutf-8").unwrap();

        assert!(spec.path.is_none());
        assert_eq!(spec.encoding.unwrap().name(), "UTF-8");
    }

    #[test]
    fn converts_buffer_bytes_with_encoding() {
        let spec = parse_buffer_io_spec("-e windows-1252").unwrap();
        let decoded = decode_buffer_bytes(vec![0xe9], spec.encoding);
        let encoded = encode_buffer_bytes("é".as_bytes(), spec.encoding);

        assert_eq!(decoded, "é".as_bytes());
        assert_eq!(encoded, vec![0xe9]);
    }
}