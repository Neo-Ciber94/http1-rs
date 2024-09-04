pub fn encode_uri_component<S: AsRef<str>>(input: S) -> String {
    let mut encoded = String::new();

    for byte in input.as_ref().bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }

    encoded
}

#[derive(Debug)]
pub struct InvalidUriComponent;

pub fn decode_uri_component<S: AsRef<str>>(input: S) -> Result<String, InvalidUriComponent> {
    let mut decoded = Vec::new();
    let mut chars = input.as_ref().chars();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex1 = chars.next().ok_or(InvalidUriComponent)?;
            let hex2 = chars.next().ok_or(InvalidUriComponent)?;
            let hex = format!("{}{}", hex1, hex2);
            let byte = u8::from_str_radix(&hex, 16).map_err(|_| InvalidUriComponent)?;
            decoded.push(byte);
        } else {
            decoded.push(c as u8);
        }
    }

    String::from_utf8(decoded).map_err(|_| InvalidUriComponent)
}

#[cfg(test)]
mod tests {
    use super::{decode_uri_component, encode_uri_component};

    #[test]
    fn should_encode_special_characters() {
        assert_eq!(encode_uri_component("hello@world"), "hello%40world");
        assert_eq!(encode_uri_component("100% free"), "100%25%20free");
        assert_eq!(encode_uri_component("a+b=c"), "a%2Bb%3Dc");
        assert_eq!(encode_uri_component("rust-lang.org"), "rust-lang.org");
    }

    #[test]
    fn should_encode_reserved_characters() {
        assert_eq!(encode_uri_component("! * ' ( ) ; : @ & = + $ , / ? # [ ]"),
            "%21%20%2A%20%27%20%28%20%29%20%3B%20%3A%20%40%20%26%20%3D%20%2B%20%24%20%2C%20%2F%20%3F%20%23%20%5B%20%5D");
    }

    #[test]
    fn should_encode_unicode_characters() {
        assert_eq!(
            encode_uri_component("こんにちは"),
            "%E3%81%93%E3%82%93%E3%81%AB%E3%81%A1%E3%81%AF"
        );
        assert_eq!(encode_uri_component("你好"), "%E4%BD%A0%E5%A5%BD");
        assert_eq!(encode_uri_component("😊"), "%F0%9F%98%8A");
    }

    #[test]
    fn should_decode_str() {
        assert_eq!(
            decode_uri_component("hello%20world").unwrap(),
            "hello world"
        );
    }

    #[test]
    fn should_decode_special_characters() {
        assert_eq!(
            decode_uri_component("hello%40world").unwrap(),
            "hello@world"
        );
        assert_eq!(decode_uri_component("100%25%20free").unwrap(), "100% free");
        assert_eq!(decode_uri_component("a%2Bb%3Dc").unwrap(), "a+b=c");
    }

    #[test]
    fn should_decode_unicode_characters() {
        assert_eq!(
            decode_uri_component("%E3%81%93%E3%82%93%E3%81%AB%E3%81%A1%E3%81%AF").unwrap(),
            "こんにちは"
        );
        assert_eq!(decode_uri_component("%E4%BD%A0%E5%A5%BD").unwrap(), "你好");
        assert_eq!(decode_uri_component("%F0%9F%98%8A").unwrap(), "😊");
    }

    #[test]
    fn should_decode_incomplete_percent_encoding() {
        assert!(decode_uri_component("hello%2world").is_err());
        assert!(decode_uri_component("hello%").is_err());
    }

    #[test]
    fn should_handle_empty_string() {
        assert_eq!(encode_uri_component(""), "");
        assert_eq!(decode_uri_component("").unwrap(), "");
    }
}
