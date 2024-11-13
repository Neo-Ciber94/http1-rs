use std::string::FromUtf8Error;

const BASE64_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const BASE64_PAD: u8 = b'=';

pub fn encode(s: impl AsRef<str>) -> Result<String, FromUtf8Error> {
    let bytes = s.as_ref().as_bytes();
    let encoded_bytes = encode_to_bytes(bytes);
    String::from_utf8(encoded_bytes)
}

pub fn decode(s: impl AsRef<str>) -> Result<String, FromUtf8Error>  {
    let bytes = s.as_ref().as_bytes();
    let decoded_bytes = decode_from_bytes(bytes);
    String::from_utf8(decoded_bytes)
}

pub fn encode_to_bytes<S: AsRef<[u8]>>(s: S) -> Vec<u8> {
    let bytes = s.as_ref();
    let mut result = Vec::new();
    let mut i = 0;

    // Process 3 bytes at a time
    while i + 3 <= bytes.len() {
        let chunk = (bytes[i] as u32) << 16 | (bytes[i + 1] as u32) << 8 | (bytes[i + 2] as u32);

        result.push(BASE64_ALPHABET[(chunk >> 18) as usize]);
        result.push(BASE64_ALPHABET[((chunk >> 12) & 0x3F) as usize]);
        result.push(BASE64_ALPHABET[((chunk >> 6) & 0x3F) as usize]);
        result.push(BASE64_ALPHABET[(chunk & 0x3F) as usize]);

        i += 3;
    }

    // Handle padding if the length isn't a multiple of 3
    let remaining = bytes.len() % 3;
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

pub fn decode_from_bytes(s: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut temp = 0u32;
    let mut count = 0;

    for &byte in s {
        if byte == BASE64_PAD {
            break;
        }

        // Find the position of the byte in the alphabet
        let index = BASE64_ALPHABET.iter().position(|&c| c == byte);
        if let Some(idx) = index {
            temp = (temp << 6) | (idx as u32);
            count += 1;

            // Once we have 4 Base64 characters, decode them
            if count == 4 {
                result.push((temp >> 16) as u8);
                result.push((temp >> 8 & 0xFF) as u8);
                result.push((temp & 0xFF) as u8);
                temp = 0;
                count = 0;
            }
        }
    }

    // Handle remaining characters
    if count > 1 {
        result.push((temp >> 16) as u8);
    }
    if count > 2 {
        result.push((temp >> 8 & 0xFF) as u8);
    }

    result
}
