use std::fmt::{Debug, Display};

/// A structure representing a UUID (Universally Unique Identifier).
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uuid([u32; 4]);

impl Uuid {
    /// Returns a nil (all zeros) UUID.
    pub fn nil() -> Self {
        Uuid([0, 0, 0, 0])
    }

    /// Creates a new UUID from 4 parts represented as an array of `u32`.
    pub fn from_parts(value: [u32; 4]) -> Self {
        Uuid(value)
    }

    /// Converts a `u128` value into a UUID by splitting it into four `u32` parts.
    pub fn from_u128(value: u128) -> Self {
        let a = (value >> 96) as u32;
        let b = ((value >> 64) & 0xFFFFFFFF) as u32;
        let c = ((value >> 32) & 0xFFFFFFFF) as u32;
        let d = (value & 0xFFFFFFFF) as u32;

        Uuid::from_parts([a, b, c, d])
    }

    /// Generates a new random UUID following version 4 (random).
    pub fn new_v4() -> Self {
        Uuid::from_u128(
            crate::rng::random::<u128>() & 0xFFFFFFFFFFFF4FFFBFFFFFFFFFFFFFFF
                | 0x40008000000000000000,
        )
    }

    /// Returns a reference to the internal array of 4 `u32` values.
    pub fn as_bytes(&self) -> &[u32] {
        &self.0
    }

    /// Converts the UUID into a single `u128` value.
    pub fn as_u128(&self) -> u128 {
        let part1 = (self.0[0] as u128) << 96;
        let part2 = (self.0[1] as u128) << 64;
        let part3 = (self.0[2] as u128) << 32;
        let part4 = self.0[3] as u128;

        part1 | part2 | part3 | part4
    }

    /// Formats the UUID as a hyphenated string (e.g., `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`).
    pub fn as_hyphened(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        let a = self.0[0];
        let b = (self.0[1] >> 16) & 0xFFFF;
        let c = self.0[1] & 0xFFFF;
        let d = (self.0[2] >> 16) & 0xFFFF;
        let e = ((self.0[2] & 0xFFFF) as u64) << 32 | self.0[3] as u64;

        write!(f, "{a:08x}-{b:04x}-{c:04x}-{d:04x}-{e:012x}")
    }

    /// Formats the UUID within parentheses string (e.g., `{xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx}`).
    pub fn as_parentheses(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        let a = self.0[0];
        let b = (self.0[1] >> 16) & 0xFFFF;
        let c = self.0[1] & 0xFFFF;
        let d = (self.0[2] >> 16) & 0xFFFF;
        let e = ((self.0[2] & 0xFFFF) as u64) << 32 | self.0[3] as u64;

        write!(f, "{{{a:08x}-{b:04x}-{c:04x}-{d:04x}-{e:012x}}}")
    }

    /// Formats the UUID as a simple string without hyphens (e.g., `xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx`).
    pub fn as_simple(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        let a = self.0[0];
        let b = (self.0[1] >> 16) & 0xFFFF;
        let c = self.0[1] & 0xFFFF;
        let d = (self.0[2] >> 16) & 0xFFFF;
        let e = ((self.0[2] & 0xFFFF) as u64) << 32 | self.0[3] as u64;

        write!(f, "{a:08x}{b:04x}{c:04x}{d:04x}{e:012x}")
    }
}

impl Debug for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_parentheses(f)
    }
}

impl Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_hyphened(f)
    }
}

#[cfg(test)]
mod tests {
    use super::Uuid;

    #[test]
    fn should_display_nil_uuid() {
        let v = Uuid::nil();
        let mut hyphened_str = String::new();
        v.as_hyphened(&mut hyphened_str).unwrap();
        assert_eq!(hyphened_str, "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn should_create_uuid_from_u128() {
        let value: u128 = 0x1234567890ABCDEF1234567890ABCDEF;
        let uuid = Uuid::from_u128(value);
        assert_eq!(uuid.as_u128(), value);
    }

    #[test]
    fn should_convert_uuid_to_u128() {
        let uuid = Uuid::from_parts([0x12345678, 0x90ABCDEF, 0x12345678, 0x90ABCDEF]);
        let expected_value: u128 = 0x1234567890ABCDEF1234567890ABCDEF;
        assert_eq!(uuid.as_u128(), expected_value);
    }

    #[test]
    fn should_display_hyphened_uuid() {
        let uuid = Uuid::from_parts([0x12345678, 0x90ABCDEF, 0x12345678, 0x90ABCDEF]);
        let mut hyphened_str = String::new();
        uuid.as_hyphened(&mut hyphened_str).unwrap();
        assert_eq!(hyphened_str, "12345678-90ab-cdef-1234-567890abcdef");
    }

    #[test]
    fn should_display_simple_uuid() {
        let uuid = Uuid::from_parts([0x12345678, 0x90ABCDEF, 0x12345678, 0x90ABCDEF]);
        let mut simple_str = String::new();
        uuid.as_simple(&mut simple_str).unwrap();
        assert_eq!(simple_str, "1234567890abcdef1234567890abcdef");
    }

    #[test]
    fn should_display_parentheses_uuid() {
        let uuid = Uuid::from_parts([0x12345678, 0x90ABCDEF, 0x12345678, 0x90ABCDEF]);
        let mut s = String::new();
        uuid.as_parentheses(&mut s).unwrap();
        assert_eq!(s, "{12345678-90ab-cdef-1234-567890abcdef}");
    }

    #[test]
    fn should_create_and_format_new_v4_uuid() {
        let uuid = Uuid::new_v4();
        let mut hyphened_str = String::new();
        uuid.as_hyphened(&mut hyphened_str).unwrap();
        assert!(
            hyphened_str.len() > 0,
            "new_v4 should create a non-empty UUID string"
        );
    }

    #[test]
    fn should_compare_uuids_correctly() {
        let uuid1 = Uuid::from_parts([0x12345678, 0x90ABCDEF, 0x12345678, 0x90ABCDEF]);
        let uuid2 = Uuid::from_parts([0x12345678, 0x90ABCDEF, 0x12345678, 0x90ABCDEF]);
        let uuid3 = Uuid::from_parts([0x11111111, 0x22222222, 0x33333333, 0x44444444]);

        assert_eq!(uuid1, uuid2);
        assert_ne!(uuid1, uuid3);
        assert!(uuid1 > uuid3);
    }

    #[test]
    fn should_handle_nil_uuid_as_u128() {
        let nil_uuid = Uuid::nil();
        assert_eq!(nil_uuid.as_u128(), 0);
    }
}
