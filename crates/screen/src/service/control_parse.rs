use std::io;

pub(super) fn parse_resize_payload(payload: &str) -> io::Result<(u16, u16)> {
    let mut parts = payload.split_whitespace();
    let Some(cols) = parts.next().and_then(|value| value.parse::<u16>().ok()) else {
        return Err(invalid_resize_payload());
    };
    let Some(rows) = parts.next().and_then(|value| value.parse::<u16>().ok()) else {
        return Err(invalid_resize_payload());
    };
    if cols == 0 || rows == 0 || parts.next().is_some() {
        return Err(invalid_resize_payload());
    }
    Ok((cols, rows))
}

pub(super) fn control_command_payload(inline_payload: &str, args: &[String]) -> String {
    let mut payload = String::new();
    if !inline_payload.is_empty() {
        payload.push_str(inline_payload);
    }
    for arg in args {
        if !payload.is_empty() {
            payload.push(' ');
        }
        payload.push_str(arg);
    }
    payload
}

pub(super) fn decode_stuff_payload(payload: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(payload.len());
    let mut chars = payload.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            push_utf8(&mut bytes, ch);
            continue;
        }

        match chars.next() {
            Some('n') => bytes.push(b'\n'),
            Some('r') => bytes.push(b'\r'),
            Some('t') => bytes.push(b'\t'),
            Some('\\') => bytes.push(b'\\'),
            Some(first @ '0'..='7') => bytes.push(decode_octal_escape(first, &mut chars)),
            Some(other) => {
                bytes.push(b'\\');
                push_utf8(&mut bytes, other);
            }
            None => bytes.push(b'\\'),
        }
    }

    bytes
}

fn decode_octal_escape(first: char, chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> u8 {
    let mut value = first.to_digit(8).unwrap_or_default();
    for _ in 0..2 {
        let Some(next) = chars.peek().copied() else {
            break;
        };
        let Some(digit) = next.to_digit(8) else {
            break;
        };
        chars.next();
        value = value * 8 + digit;
    }
    value.min(0xff) as u8
}

fn invalid_resize_payload() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_resize_required_hint(),
    )
}

fn push_utf8(bytes: &mut Vec<u8>, ch: char) {
    let mut buf = [0u8; 4];
    bytes.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
}

#[cfg(test)]
mod tests {
    use super::{control_command_payload, decode_stuff_payload, parse_resize_payload};

    #[test]
    fn parses_resize_payload() {
        assert_eq!(parse_resize_payload("120 40").unwrap(), (120, 40));
        assert!(parse_resize_payload("0 40").is_err());
        assert!(parse_resize_payload("120").is_err());
    }

    #[test]
    fn combines_inline_and_argument_payload() {
        let args = vec![String::from("two"), String::from("three")];

        assert_eq!(control_command_payload("one", &args), "one two three");
    }

    #[test]
    fn decodes_stuff_escape_sequences() {
        assert_eq!(decode_stuff_payload("a\\n\\t"), b"a\n\t".to_vec());
        assert_eq!(decode_stuff_payload("run\\015"), b"run\r".to_vec());
        assert_eq!(decode_stuff_payload("\\033[A"), vec![0x1b, b'[', b'A']);
        assert_eq!(decode_stuff_payload("a\\x"), b"a\\x".to_vec());
    }
}