use std::error::Error;
use std::fmt;

const BASE64_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const BASE64_PAD: u8 = b'=';

#[derive(Debug)]
pub enum Base64Error {
    InvalidInput(String),
    InvalidPadding,
    InvalidCharacter(u8),
    InvalidUtf8,
}

impl fmt::Display for Base64Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Base64Error::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Base64Error::InvalidPadding => write!(f, "Invalid base64 padding"),
            Base64Error::InvalidCharacter(c) => write!(f, "Invalid base64 character: {}", c),
            Base64Error::InvalidUtf8 => write!(f, "Invalid UTF-8 in decoded bytes"),
        }
    }
}

impl Error for Base64Error {}

pub fn encode(s: impl AsRef<str>) -> String {
    let bytes = s.as_ref().as_bytes();
    String::from_utf8(encode_to_bytes(bytes)).unwrap_or_default()
}

pub fn decode(s: impl AsRef<str>) -> Result<String, Base64Error> {
    let bytes = s.as_ref().as_bytes();
    let decoded_bytes = decode_from_bytes(bytes)?;
    String::from_utf8(decoded_bytes).map_err(|_| Base64Error::InvalidUtf8)
}

pub fn encode_to_bytes<S: AsRef<[u8]>>(s: S) -> Vec<u8> {
    let bytes = s.as_ref();
    let mut result = Vec::with_capacity(((bytes.len() + 2) / 3) * 4);
    let mut i = 0;

    while i + 3 <= bytes.len() {
        let chunk = (bytes[i] as u32) << 16 | (bytes[i + 1] as u32) << 8 | (bytes[i + 2] as u32);

        result.push(BASE64_ALPHABET[(chunk >> 18) as usize]);
        result.push(BASE64_ALPHABET[((chunk >> 12) & 0x3F) as usize]);
        result.push(BASE64_ALPHABET[((chunk >> 6) & 0x3F) as usize]);
        result.push(BASE64_ALPHABET[(chunk & 0x3F) as usize]);

        i += 3;
    }

    let remaining = bytes.len() - i;
    if remaining > 0 {
        let mut chunk = (bytes[i] as u32) << 16;
        if remaining == 2 {
            chunk |= (bytes[i + 1] as u32) << 8;
        }

        result.push(BASE64_ALPHABET[(chunk >> 18) as usize]);
        result.push(BASE64_ALPHABET[((chunk >> 12) & 0x3F) as usize]);

        if remaining == 2 {
            result.push(BASE64_ALPHABET[((chunk >> 6) & 0x3F) as usize]);
            result.push(BASE64_PAD);
        } else {
            result.push(BASE64_PAD);
            result.push(BASE64_PAD);
        }
    }

    result
}

pub fn decode_from_bytes(s: &[u8]) -> Result<Vec<u8>, Base64Error> {
    if s.is_empty() {
        return Ok(Vec::new());
    }

    // Validate input length
    if s.len() % 4 != 0 {
        return Err(Base64Error::InvalidInput(
            "Input length must be divisible by 4".to_string(),
        ));
    }

    let mut result = Vec::with_capacity((s.len() / 4) * 3);
    let mut temp = 0u32;
    let mut count = 0;
    let mut padding_count = 0;

    for (i, &byte) in s.iter().enumerate() {
        if byte == BASE64_PAD {
            if i < s.len() - 2 {
                return Err(Base64Error::InvalidPadding);
            }
            padding_count += 1;
            continue;
        }

        if padding_count > 0 {
            return Err(Base64Error::InvalidPadding);
        }

        let value = BASE64_ALPHABET
            .iter()
            .position(|&c| c == byte)
            .ok_or(Base64Error::InvalidCharacter(byte))?;

        temp = (temp << 6) | (value as u32);
        count += 1;

        if count == 4 {
            result.push((temp >> 16) as u8);
            result.push((temp >> 8 & 0xFF) as u8);
            result.push((temp & 0xFF) as u8);
            temp = 0;
            count = 0;
        }
    }

    // Handle padding
    match (count, padding_count) {
        (0, 0) => Ok(result),
        (2, 2) => {
            result.push((temp >> 4) as u8);
            Ok(result)
        }
        (3, 1) => {
            result.push((temp >> 10) as u8);
            result.push((temp >> 2) as u8);
            Ok(result)
        }
        _ => Err(Base64Error::InvalidPadding),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_success() {
        // Basic test
        assert_eq!(encode("Hello Jinx!"), "SGVsbG8gSmlueCE=");
        assert_eq!(decode("SGVsbG8gSmlueCE=").unwrap(), "Hello Jinx!");

        // Empty string
        assert_eq!(encode(""), "");
        assert_eq!(decode("").unwrap(), "");

        // Single/double/triple chars (padding variations)
        assert_eq!(encode("a"), "YQ==");
        assert_eq!(decode("YQ==").unwrap(), "a");

        assert_eq!(encode("ab"), "YWI=");
        assert_eq!(decode("YWI=").unwrap(), "ab");

        assert_eq!(encode("abc"), "YWJj");
        assert_eq!(decode("YWJj").unwrap(), "abc");

        // UTF-8 character
        assert_eq!(encode("🦀"), "8J+mgA==");
        assert_eq!(decode("8J+mgA==").unwrap(), "🦀");
    }

    #[test]
    fn test_decode_errors() {
        // Invalid length
        assert!(matches!(
            decode("SGVsbA"),
            Err(Base64Error::InvalidInput(_))
        ));

        // Invalid character
        assert!(matches!(
            decode("SGVs!G8="),
            Err(Base64Error::InvalidCharacter(_))
        ));

        // Invalid padding
        assert!(matches!(
            decode("SGVsbG8=Z==="),
            Err(Base64Error::InvalidPadding)
        ));

        // Padding in wrong position
        assert!(matches!(
            decode("SG=sbG8="),
            Err(Base64Error::InvalidPadding)
        ));
    }
}
