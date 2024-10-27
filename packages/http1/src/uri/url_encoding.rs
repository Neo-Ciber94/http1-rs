/// Alphabet used for percent encoding.
pub trait Alphabet {
    /// Whether if this alphabet contains the given value.
    fn contains(&self, value: u8) -> bool;
}

/// Percent encode alphabet.
pub struct UrlASCII;
impl Alphabet for UrlASCII {
    fn contains(&self, value: u8) -> bool {
        matches!(value, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~')
    }
}

/// Encodes a URI component by using the `UrlComponentEncode` alphabet for percent-encoding.
/// This function wraps `encode_uri_component_with` with the default alphabet.
///
/// # Parameters
/// - `input`: The string to be encoded.
///
/// # Returns
/// A `String` representing the encoded URI component.
pub fn encode<S: AsRef<str>>(input: S) -> String {
    encode_with(input, UrlASCII)
}

/// Encodes a URI component using a custom alphabet for percent-encoding.
///
/// This function iterates over the input string and encodes each character that is not part of
/// the given `alphabet` as a percent-encoded value (e.g., `%20` for a space).
///
/// # Parameters
/// - `input`: The string to be encoded.
/// - `alphabet`: The set of allowed characters for the URI component encoding.
///
/// # Returns
/// A `String` representing the encoded URI component.
pub fn encode_with<S: AsRef<str>>(input: S, alphabet: impl Alphabet) -> String {
    // Create an empty `String` to store the encoded result.
    let mut encoded = String::new();

    // Iterate over the bytes of the input string.
    for byte in input.as_ref().bytes() {
        // If the byte is allowed by the alphabet, append it as-is to the result.
        match byte {
            _ if alphabet.contains(byte) => {
                encoded.push(byte as char);
            }
            // Otherwise, percent-encode the byte and append the result.
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }

    // Return the encoded string.
    encoded
}

#[derive(Debug)]
/// An error type used when URI component decoding fails.
pub struct InvalidUriComponent;

/// Decodes a URI component that may contain percent-encoded characters back into a plain string.
///
/// The function will look for `%` signs and attempt to decode the following two characters as hex digits
/// representing a byte value. If the encoding is invalid, the function returns an error.
///
/// # Parameters
/// - `input`: The percent-encoded string to decode.
///
/// # Returns
/// A `Result<String, InvalidUriComponent>` where:
/// - `Ok` contains the decoded string.
/// - `Err` contains the `InvalidUriComponent` error if the input is not a valid URI component.
pub fn decode<S: AsRef<str>>(input: S) -> Result<String, InvalidUriComponent> {
    // Vector to hold the decoded bytes.
    let mut decoded = Vec::new();
    let mut chars = input.as_ref().chars();

    // Iterate over the characters of the input string.
    while let Some(c) = chars.next() {
        if c == '%' {
            // If the character is '%', try to decode the next two characters as a hex value.
            let hex1 = chars.next().ok_or(InvalidUriComponent)?;
            let hex2 = chars.next().ok_or(InvalidUriComponent)?;
            let hex = format!("{}{}", hex1, hex2);

            // Convert the hex string to a byte.
            let byte = u8::from_str_radix(&hex, 16).map_err(|_| InvalidUriComponent)?;
            decoded.push(byte);
        } else if c == '+' {
            // If the character is '+', treat it as a space.
            decoded.push(b' '); // Push a space byte
        } else {
            // If the character is not a percent-encoded value, just push it as-is.
            decoded.push(c as u8);
        }
    }

    // Attempt to convert the vector of bytes to a UTF-8 string.
    // If this fails, return an error.
    String::from_utf8(decoded).map_err(|_| InvalidUriComponent)
}

#[cfg(test)]
mod tests {
    use super::{decode, encode};

    #[test]
    fn should_encode_special_characters() {
        assert_eq!(encode("hello@world"), "hello%40world");
        assert_eq!(encode("100% free"), "100%25%20free");
        assert_eq!(encode("a+b=c"), "a%2Bb%3Dc");
        assert_eq!(encode("rust-lang.org"), "rust-lang.org");
    }

    #[test]
    fn should_encode_reserved_characters() {
        assert_eq!(encode("! * ' ( ) ; : @ & = + $ , / ? # [ ]"),
            "%21%20%2A%20%27%20%28%20%29%20%3B%20%3A%20%40%20%26%20%3D%20%2B%20%24%20%2C%20%2F%20%3F%20%23%20%5B%20%5D");
    }

    #[test]
    fn should_encode_unicode_characters() {
        assert_eq!(
            encode("こんにちは"),
            "%E3%81%93%E3%82%93%E3%81%AB%E3%81%A1%E3%81%AF"
        );
        assert_eq!(encode("你好"), "%E4%BD%A0%E5%A5%BD");
        assert_eq!(encode("😊"), "%F0%9F%98%8A");
    }

    #[test]
    fn should_decode_str() {
        assert_eq!(decode("hello%20world").unwrap(), "hello world");
    }

    #[test]
    fn should_decode_special_characters() {
        assert_eq!(decode("hello%40world").unwrap(), "hello@world");
        assert_eq!(decode("100%25%20free").unwrap(), "100% free");
        assert_eq!(decode("a%2Bb%3Dc").unwrap(), "a+b=c");
    }

    #[test]
    fn should_decode_unicode_characters() {
        assert_eq!(
            decode("%E3%81%93%E3%82%93%E3%81%AB%E3%81%A1%E3%81%AF").unwrap(),
            "こんにちは"
        );
        assert_eq!(decode("%E4%BD%A0%E5%A5%BD").unwrap(), "你好");
        assert_eq!(decode("%F0%9F%98%8A").unwrap(), "😊");
    }

    #[test]
    fn should_decode_incomplete_percent_encoding() {
        assert!(decode("hello%2world").is_err());
        assert!(decode("hello%").is_err());
    }

    #[test]
    fn should_handle_empty_string() {
        assert_eq!(encode(""), "");
        assert_eq!(decode("").unwrap(), "");
    }
}
