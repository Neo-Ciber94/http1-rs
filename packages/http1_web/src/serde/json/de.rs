use std::io::{BufReader, Read};

use http1::common::map::OrderedMap;

use crate::serde::{
    de::{Deserializer, Error},
    expected::Expected,
};

use super::{
    number::Number,
    value::{JsonBytesAccess, JsonObjectAccess, JsonSeqAccess, JsonValue},
};

pub struct JsonDeserializer<R> {
    reader: BufReader<R>,
    next: Option<u8>,
    depth: usize,
}

impl JsonDeserializer<()> {
    pub fn new<R>(reader: R) -> JsonDeserializer<R>
    where
        R: Read,
    {
        JsonDeserializer {
            reader: BufReader::new(reader),
            next: None,
            depth: 0,
        }
    }
}

impl<R: Read> JsonDeserializer<R> {
    fn peek(&mut self) -> Option<u8> {
        match self.next {
            Some(b) => Some(b),
            None => {
                let next = self.read_byte();
                self.next = next;
                next
            }
        }
    }

    fn read_byte(&mut self) -> Option<u8> {
        if let Some(b) = self.next.take() {
            return Some(b);
        }

        let buf = &mut [0];
        match self.reader.read(buf) {
            Ok(n) => match n {
                0 => None,
                _ => Some(buf[0]),
            },
            Err(_) => None,
        }
    }

    fn read_until_next_non_whitespace(&mut self) -> Option<u8> {
        while let Some(b) = self.peek() {
            if b.is_ascii_whitespace() {
                self.read_byte();
                continue;
            }

            return Some(b);
        }

        None
    }

    fn read_until_byte(&mut self, expected: u8) -> Result<u8, Error> {
        match self.read_until_next_non_whitespace() {
            Some(b) => {
                if b == expected {
                    return Ok(b);
                }

                Err(Error::custom(format!(
                    "expected `{}` but was `{}`",
                    expected as char, b as char
                )))
            }
            None => Err(Error::custom(format!(
                "expected `{}` but was empty",
                expected as char
            ))),
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        match self.next.take() {
            None => self.reader.read(buf),
            Some(b) => {
                // We set the value we already have saved
                buf[0] = b;

                if buf.len() > 1 {
                    // And then fill the rest of the buffer
                    let chunk = &mut buf[1..];
                    let size = self.reader.read(chunk)?;
                    return Ok(size + 1);
                }

                Ok(1)
            }
        }
    }

    fn consume_rest(&mut self) -> Result<(), Error> {
        // We only consume the rest at the top level
        if self.depth > 0 {
            return Ok(());
        }

        if let Some(b) = self.next.take() {
            if !b.is_ascii_whitespace() {
                return Err(Error::custom(format!(
                    "expected only whitespace after end but found: `{}`",
                    b as char
                )));
            }
        }

        let mut buffer = [0; 64]; // buffer to hold a single byte

        loop {
            let read_count = self.reader.read(&mut buffer).map_err(Error::error)?;

            match read_count {
                0 => break,
                n => {
                    let chunk = &buffer[..n];
                    if !chunk.iter().all(u8::is_ascii_whitespace) {
                        let s = String::from_utf8_lossy(chunk);
                        return Err(Error::custom(format!(
                            "expected only whitespace after end but found: `{s}`"
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    fn parse_json(&mut self) -> Result<JsonValue, Error> {
        match self.read_until_next_non_whitespace() {
            Some(b) => match b {
                b't' | b'f' => self.parse_bool().map(JsonValue::Bool),
                b'n' => self.parse_null().map(|_| JsonValue::Null),
                b'0'..=b'9' | b'-' | b'e' | b'.' => self.parse_number().map(JsonValue::Number),
                b'"' => self.parse_string().map(JsonValue::String),
                b'[' => self.parse_array().map(JsonValue::Array),
                b'{' => self.parse_object().map(JsonValue::Object),
                _ => Err(Error::custom(format!(
                    "unexpected json token `{}`",
                    b as char,
                ))),
            },
            None => Err(Error::custom("empty value")),
        }
    }

    fn parse_null(&mut self) -> Result<(), Error> {
        let buf = &mut [0; 4];
        match self.read(buf) {
            Ok(4) if buf == b"null" => {
                self.consume_rest()?;
                Ok(())
            }
            Ok(n) => {
                let s = String::from_utf8_lossy(&buf[..n]);
                Err(Error::custom(format!("expected 'null' but was `{s}`")))
            }
            Err(err) => Err(Error::error(err)),
        }
    }

    fn parse_bool(&mut self) -> Result<bool, Error> {
        let (len, expected) = match self.peek() {
            Some(b't') => (4, "true"),  // length of "true"
            Some(b'f') => (5, "false"), // length of "false"
            _ => return Err(Error::custom("expected 'boolean'")),
        };

        let mut buf = [0u8; 5];
        let read = self.read(&mut buf[..len]).map_err(Error::error)?;

        if &buf[..read] != expected.as_bytes() {
            let s = String::from_utf8_lossy(&buf[..read]);
            return Err(Error::custom(format!("expected `{expected}` but was {s}",)));
        }

        self.consume_rest()?;
        Ok(expected == "true")
    }

    fn parse_number(&mut self) -> Result<Number, Error> {
        let mut s = String::with_capacity(2);

        loop {
            if self.peek().is_none() || self.peek().as_ref().is_some_and(u8::is_ascii_whitespace) {
                break;
            }

            match self.peek() {
                None => return Err(Error::custom("expected number")),
                Some(byte) => match byte {
                    b'-' | b'.' | b'e' => {
                        if byte == b'.' && s.contains(".") {
                            return Err(Error::custom("invalid decimal number"));
                        }

                        s.push(byte as char);
                    }
                    _ if byte.is_ascii_digit() => {
                        s.push(byte as char);
                    }
                    _ => break,
                },
            }

            self.read_byte();
        }

        self.consume_rest()?;

        if s.contains(".") || s.contains("e") {
            let f: f64 = s.parse().map_err(Error::error)?;
            return Ok(Number::from(f));
        }

        let is_negative = s.starts_with("-");

        if is_negative {
            let i: i128 = s.parse().map_err(Error::error)?;
            Ok(Number::from(i))
        } else {
            let u: u128 = s.parse().map_err(Error::error)?;
            Ok(Number::from(u))
        }
    }

    fn parse_string(&mut self) -> Result<String, Error> {
        let mut s = String::with_capacity(2);

        self.read_until_byte(b'"')?;
        self.read_byte();

        loop {
            match self.peek() {
                None => return Err(Error::custom("expected next string char")),
                Some(byte) => match byte {
                    b'\\' => match self.read_byte() {
                        Some(b'"') => s.push('"'),
                        Some(b'\\') => s.push('\\'),
                        Some(b'n') => s.push('\n'),
                        Some(b't') => s.push('\t'),
                        Some(other) => {
                            return Err(Error::custom(format!(
                                "unexpected escape sequence: \\{}",
                                other as char
                            )));
                        }
                        None => return Err(Error::custom("expected character after escape")),
                    },
                    b'"' => {
                        self.read_byte();
                        break;
                    }
                    _ => {
                        s.push(byte as char);
                    }
                },
            }

            self.read_byte();
        }

        self.consume_rest()?;

        Ok(s)
    }

    fn parse_array(&mut self) -> Result<Vec<JsonValue>, Error> {
        self.read_until_byte(b'[')?;
        self.read_byte(); // discard the `[`
        self.depth += 1;

        let mut vec = vec![];

        // If is just an empty array, exit
        if self.read_until_next_non_whitespace() == Some(b']') {
            self.read_byte();
            self.consume_rest()?;
            return Ok(vec);
        }

        loop {
            // Parse first element
            let item = self.parse_json()?;
            vec.push(item);

            match self.read_until_next_non_whitespace() {
                Some(b) => match b {
                    b',' => {
                        self.read_byte(); // discard ','
                        continue;
                    }
                    b']' => {
                        break;
                    }
                    _ => {}
                },
                None => break,
            }
        }

        // Ensure is ending with ']'
        self.read_until_byte(b']')?;
        self.read_byte();

        // Consume rest
        self.depth -= 1;
        self.consume_rest()?;

        Ok(vec)
    }

    fn parse_object(&mut self) -> Result<OrderedMap<String, JsonValue>, Error> {
        self.read_until_byte(b'{')?;
        self.read_byte(); // discard the `{`

        let mut map = OrderedMap::new();

        // If is just an empty object, exit
        if self.read_until_next_non_whitespace() == Some(b'}') {
            self.read_byte();
            self.consume_rest()?;
            return Ok(map);
        }

        self.depth += 1;

        loop {
            let key = self.parse_string()?;

            // Read separator
            self.read_until_byte(b':')?;
            self.read_byte();

            let value = self.parse_json()?;

            map.insert(key, value);

            match self.read_until_next_non_whitespace() {
                Some(b',') => {
                    self.read_byte(); // read next
                }
                Some(b'}') => {
                    self.read_byte(); // end
                    break;
                }
                Some(b) => {
                    return Err(Error::custom(format!(
                        "invalid object element token {}",
                        b as char
                    )))
                }
                None => return Err(Error::custom("expected object element but was empty")),
            }
        }

        self.depth -= 1;
        self.consume_rest()?;

        Ok(map)
    }
}

impl<R: Read> Deserializer for JsonDeserializer<R> {
    fn deserialize_unit<V>(mut self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        self.parse_null()?;
        visitor.visit_unit()
    }

    fn deserialize_bool<V>(mut self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let value = self.parse_bool()?;
        visitor.visit_bool(value)
    }

    fn deserialize_u8<V>(mut self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_u8()
            .ok_or_else(|| type_mismatch_error::<u8>(number))?;

        visitor.visit_u8(value)
    }

    fn deserialize_u16<V>(mut self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_u16()
            .ok_or_else(|| type_mismatch_error::<u16>(number))?;
        visitor.visit_u16(value)
    }

    fn deserialize_u32<V>(mut self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_u32()
            .ok_or_else(|| type_mismatch_error::<u32>(number))?;
        visitor.visit_u32(value)
    }

    fn deserialize_u64<V>(mut self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_u64()
            .ok_or_else(|| type_mismatch_error::<u64>(number))?;
        visitor.visit_u64(value)
    }

    fn deserialize_u128<V>(mut self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_u128()
            .ok_or_else(|| type_mismatch_error::<u128>(number))?;
        visitor.visit_u128(value)
    }

    fn deserialize_i8<V>(mut self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_i8()
            .ok_or_else(|| type_mismatch_error::<i8>(number))?;
        visitor.visit_i8(value)
    }

    fn deserialize_i16<V>(mut self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_i16()
            .ok_or_else(|| type_mismatch_error::<i16>(number))?;
        visitor.visit_i16(value)
    }

    fn deserialize_i32<V>(mut self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_i32()
            .ok_or_else(|| type_mismatch_error::<i32>(number))?;
        visitor.visit_i32(value)
    }

    fn deserialize_i64<V>(mut self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_i64()
            .ok_or_else(|| type_mismatch_error::<i64>(number))?;
        visitor.visit_i64(value)
    }

    fn deserialize_i128<V>(mut self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_i128()
            .ok_or_else(|| type_mismatch_error::<i128>(number))?;
        visitor.visit_i128(value)
    }

    fn deserialize_f32<V>(mut self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_f32()
            .ok_or_else(|| type_mismatch_error::<f32>(number))?;
        visitor.visit_f32(value)
    }

    fn deserialize_f64<V>(mut self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_f64()
            .ok_or_else(|| type_mismatch_error::<f64>(number))?;
        visitor.visit_f64(value)
    }

    fn deserialize_char<V>(mut self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let mut string = self.parse_string()?;

        if string.is_empty() {
            return Err(Error::custom(
                "expected char but was empty string".to_string(),
            ));
        }

        if string.len() > 1 {
            return Err(Error::custom(format!("expected char but was `{string}`")));
        }

        let c = string.pop().unwrap();
        visitor.visit_char(c)
    }

    fn deserialize_string<V>(mut self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let string = self.parse_string()?;
        visitor.visit_string(string)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let seq = self.parse_array()?;
        visitor.visit_seq(JsonSeqAccess(seq.into_iter()))
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let map = self.parse_object()?;
        visitor.visit_map(JsonObjectAccess::new(map.into_iter()))
    }

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let json_value = self.parse_json()?;
        json_value.deserialize_any(visitor)
    }

    fn deserialize_option<V>(mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let json_value = self.parse_json()?;
        json_value.deserialize_option(visitor)
    }

    fn deserialize_bytes<V>(mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let json_value = self.parse_json()?;
        match json_value {
            JsonValue::String(s) => {
                let bytes = s.into_bytes();
                visitor.visit_bytes(bytes)
            }
            _ => Err(Error::mismatch(json_value, "bytes")),
        }
    }

    fn deserialize_bytes_seq<V>(mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        let json_value = self.parse_json()?;
        match json_value {
            JsonValue::String(value) => {
                let bytes = value.into_bytes();
                visitor.visit_bytes_seq(JsonBytesAccess::new(bytes))
            }
            _ => Err(Error::custom("expected bytes")),
        }
    }
}

fn type_mismatch_error<T>(this: impl Expected + Send + Sync + 'static) -> Error
where
    T: 'static,
{
    Error::mismatch(this, std::any::type_name::<T>())
}

#[cfg(test)]
mod tests {
    use crate::{
        impl_deserialize_struct,
        serde::json::{from_str, value::JsonValue},
    };

    #[test]
    fn should_deserialize_null() {
        assert_eq!(from_str::<JsonValue>("null").unwrap(), JsonValue::Null);
        assert_eq!(from_str::<()>("null").unwrap(), ());
    }

    #[test]
    fn should_deserialize_bool() {
        assert_eq!(from_str::<bool>("true").unwrap(), true);
        assert_eq!(from_str::<bool>("false").unwrap(), false);
    }

    #[test]
    fn should_deserialize_integer() {
        // u8 range: 0 to 255
        assert_eq!(from_str::<u8>("23").unwrap(), 23);
        assert_eq!(from_str::<u8>("255").unwrap(), 255);

        // u16 range: 0 to 65535
        assert_eq!(from_str::<u16>("5090").unwrap(), 5090);
        assert_eq!(from_str::<u16>("65535").unwrap(), 65535);

        // u32 range: 0 to 4,294,967,295
        assert_eq!(from_str::<u32>("239000000").unwrap(), 239000000);
        assert_eq!(from_str::<u32>("4294967295").unwrap(), 4294967295);

        // u64 range: 0 to 18,446,744,073,709,551,615
        assert_eq!(
            from_str::<u64>("239000000000000000").unwrap(),
            239000000000000000
        );
        assert_eq!(
            from_str::<u64>("18446744073709551615").unwrap(),
            18446744073709551615
        );

        // u128 range: 0 to 340,282,366,920,938,463,463,374,607,431,768,211,455
        assert_eq!(
            from_str::<u128>("239000000000000000000000000").unwrap(),
            239000000000000000000000000
        );
        assert_eq!(
            from_str::<u128>("340282366920938463463374607431768211455").unwrap(),
            340282366920938463463374607431768211455
        );
    }

    #[test]
    fn should_deserialize_signed_integer() {
        // i8 range: -128 to 127
        assert_eq!(from_str::<i8>("-128").unwrap(), -128);
        assert_eq!(from_str::<i8>("-1").unwrap(), -1);

        // i16 range: -32,768 to 32,767
        assert_eq!(from_str::<i16>("-32768").unwrap(), -32768);
        assert_eq!(from_str::<i16>("-1000").unwrap(), -1000);

        // i32 range: -2,147,483,648 to 2,147,483,647
        assert_eq!(from_str::<i32>("-2147483648").unwrap(), -2147483648);
        assert_eq!(from_str::<i32>("-123456789").unwrap(), -123456789);

        // i64 range: -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807
        assert_eq!(
            from_str::<i64>("-9223372036854775808").unwrap(),
            -9223372036854775808
        );
        assert_eq!(from_str::<i64>("-239000000000").unwrap(), -239000000000);

        // i128 range: -170,141,183,460,469,231,731,687,303,715,884,105,728 to 170,141,183,460,469,231,731,687,303,715,884,105,727
        assert_eq!(
            from_str::<i128>("-170141183460469231731687303715884105728").unwrap(),
            -170141183460469231731687303715884105728
        );
        assert_eq!(
            from_str::<i128>("-239000000000000000").unwrap(),
            -239000000000000000
        );
    }

    #[test]
    fn should_deserialize_f32() {
        // Positive decimal numbers
        assert_eq!(from_str::<f32>("3.14").unwrap(), 3.14);
        assert_eq!(from_str::<f32>("0.001").unwrap(), 0.001);

        // Negative decimal numbers
        assert_eq!(from_str::<f32>("-2.718").unwrap(), -2.718);
        assert_eq!(from_str::<f32>("-0.001").unwrap(), -0.001);

        // Numbers using exponents (positive)
        assert_eq!(from_str::<f32>("1e3").unwrap(), 1e3); // 1000
        assert_eq!(from_str::<f32>("5.67e-2").unwrap(), 5.67e-2); // 0.0567

        // Numbers using exponents (negative)
        assert_eq!(from_str::<f32>("-1e3").unwrap(), -1e3); // -1000
        assert_eq!(from_str::<f32>("-3.21e-4").unwrap(), -3.21e-4); // -0.000321
    }

    #[test]
    fn should_deserialize_f64() {
        // Positive decimal numbers
        assert_eq!(
            from_str::<f64>("123456789.987654321").unwrap(),
            123456789.987654321
        );
        assert_eq!(from_str::<f64>("0.0000001").unwrap(), 0.0000001);

        // Negative decimal numbers
        assert_eq!(
            from_str::<f64>("-987654321.123456789").unwrap(),
            -987654321.123456789
        );
        assert_eq!(from_str::<f64>("-0.0000001").unwrap(), -0.0000001);

        // Numbers using exponents (positive)
        assert_eq!(from_str::<f64>("1.23e10").unwrap(), 1.23e10); // 12,300,000,000
        assert_eq!(from_str::<f64>("7.89e-3").unwrap(), 7.89e-3); // 0.00789

        // Numbers using exponents (negative)
        assert_eq!(from_str::<f64>("-4.56e9").unwrap(), -4.56e9); // -4,560,000,000
        assert_eq!(from_str::<f64>("-9.87e-6").unwrap(), -9.87e-6); // -0.00000987
    }

    #[test]
    fn should_deserialize_string() {
        assert_eq!(
            from_str::<String>("\"Hello World!\"").unwrap(),
            String::from("Hello World!")
        );
    }

    #[test]
    fn should_deserialize_array() {
        // Empty array
        assert_eq!(from_str::<Vec<char>>("[]").unwrap(), vec![]);
        assert_eq!(from_str::<Vec<char>>("[ ]").unwrap(), vec![]);
        assert_eq!(from_str::<Vec<char>>(" [] ").unwrap(), vec![]);

        // Array of numbers
        assert_eq!(
            from_str::<Vec<i32>>("[4,-23,10]").unwrap(),
            vec![4, -23, 10]
        );

        assert_eq!(
            from_str::<Vec<i32>>("[-5, 49, -10]").unwrap(),
            vec![-5, 49, -10]
        );

        // Array of strings
        assert_eq!(
            from_str::<Vec<String>>(r#"["five", "red", "apple"]"#).unwrap(),
            vec![
                String::from("five"),
                String::from("red"),
                String::from("apple")
            ]
        );
    }

    #[test]
    fn should_deserialize_tuple() {
        // Arity 1
        assert_eq!(from_str::<(u32,)>("[12]").unwrap(), (12,));

        // Arity 2
        assert_eq!(from_str::<(u32, bool)>("[12, true]").unwrap(), (12, true));

        // Arity 3
        assert_eq!(
            from_str::<(u32, bool, String)>("[12, true, \"hello\"]").unwrap(),
            (12, true, String::from("hello"))
        );

        // Arity 4
        assert_eq!(
            from_str::<(u32, bool, String, f64)>("[12, true, \"hello\", 3.14]").unwrap(),
            (12, true, String::from("hello"), 3.14)
        );

        // Arity 5
        assert_eq!(
            from_str::<(u32, bool, String, f64, u8)>("[12, true, \"hello\", 3.14, 42]").unwrap(),
            (12, true, String::from("hello"), 3.14, 42u8)
        );

        // Arity 6
        assert_eq!(
            from_str::<(u32, bool, String, f64, u8, char)>(
                "[12, true, \"hello\", 3.14, 42, \"a\"]"
            )
            .unwrap(),
            (12, true, String::from("hello"), 3.14, 42u8, 'a')
        );

        // Arity 7
        assert_eq!(
            from_str::<(u32, bool, String, f64, u8, char, i64)>(
                "[12, true, \"hello\", 3.14, 42, \"a\", -99]"
            )
            .unwrap(),
            (12, true, String::from("hello"), 3.14, 42u8, 'a', -99)
        );

        // Arity 8
        assert_eq!(
            from_str::<(u32, bool, String, f64, u8, char, i64, f32)>(
                "[12, true, \"hello\", 3.14, 42, \"a\", -99, 0.1]"
            )
            .unwrap(),
            (
                12,
                true,
                String::from("hello"),
                3.14,
                42u8,
                'a',
                -99,
                0.1f32
            )
        );

        // Arity 9
        assert_eq!(
            from_str::<(u32, bool, String, f64, u8, char, i64, f32, u16)>(
                "[12, true, \"hello\", 3.14, 42, \"a\", -99, 0.1, 65535]"
            )
            .unwrap(),
            (
                12,
                true,
                String::from("hello"),
                3.14,
                42u8,
                'a',
                -99,
                0.1f32,
                65535u16
            )
        );

        // Arity 10
        assert_eq!(
            from_str::<(u32, bool, String, f64, u8, char, i64, f32, u16, i8)>(
                "[12, true, \"hello\", 3.14, 42, \"a\", -99, 0.1, 65535, -128]"
            )
            .unwrap(),
            (
                12,
                true,
                String::from("hello"),
                3.14,
                42u8,
                'a',
                -99,
                0.1f32,
                65535u16,
                -128i8
            )
        );
    }

    #[test]
    fn should_deserialize_empty_object() {
        let value = from_str::<JsonValue>("{}").unwrap();
        assert!(matches!(value, JsonValue::Object(_)));

        let obj = value.as_map().unwrap();
        assert_eq!(obj.len(), 0);
    }

    #[test]
    fn should_deserialize_object() {
        let value = from_str::<JsonValue>(
            r#"{
            "name": "Yatora Yaguchi",
            "age": 18,
            "likes_art": true,
            "friends": [
                {
                    "name": "Ryuji Ayukawa",
                    "age": 18
                },
                {
                    "name": "Maru Mori",
                    "age": 21
                }
            ]
        }"#,
        )
        .unwrap();

        assert_eq!(
            value.select("name").unwrap(),
            &JsonValue::from("Yatora Yaguchi")
        );
        assert_eq!(value.select("age").unwrap(), &JsonValue::from(18));
        assert_eq!(value.select("likes_art").unwrap(), &JsonValue::from(true));
        assert_eq!(
            value.select("friends.0.name").unwrap(),
            &JsonValue::from("Ryuji Ayukawa")
        );
        assert_eq!(value.select("friends.1.age").unwrap(), &JsonValue::from(21));
    }

    #[test]
    fn should_deserialize_to_struct() {
        #[derive(Debug)]
        struct BluePeriodCharacter {
            name: String,
            age: u32,
            likes_art: bool,
            friends: Vec<BluePeriodCharacter>,
        }

        impl crate::serde::de::Deserialize for BluePeriodCharacter {
            fn deserialize<D: crate::serde::de::Deserializer>(
                deserializer: D,
            ) -> Result<Self, crate::serde::de::Error> {
                struct BluePeriodCharacterVisitor;
                impl crate::serde::visitor::Visitor for BluePeriodCharacterVisitor {
                    type Value = BluePeriodCharacter;

                    fn visit_map<Map: crate::serde::visitor::MapAccess>(
                        self,
                        mut map: Map,
                    ) -> Result<Self::Value, crate::serde::de::Error> {
                        let mut name: Result<String, crate::serde::de::Error> =
                            Err(crate::serde::de::Error::custom("missing field 'name'"));
                        let mut age: Result<u32, crate::serde::de::Error> =
                            Err(crate::serde::de::Error::custom("missing field 'age'"));
                        let mut likes_art: Result<bool, crate::serde::de::Error> =
                            Err(crate::serde::de::Error::custom("missing field 'likes_art'"));
                        let mut friends: Result<Vec<BluePeriodCharacter>, crate::serde::de::Error> =
                            Err(crate::serde::de::Error::custom("missing field 'friends'"));

                        while let Some(k) = map.next_key::<String>()? {
                            match k.as_str() {
                                "name" => {
                                    name = match map.next_value::<String>()? {
                                        Some(x) => Ok(x),
                                        None => {
                                            return Err(crate::serde::de::Error::custom(
                                                "missing field 'name'",
                                            ))
                                        }
                                    };
                                }
                                "age" => {
                                    age = match map.next_value::<u32>()? {
                                        Some(x) => Ok(x),
                                        None => {
                                            return Err(crate::serde::de::Error::custom(
                                                "missing field 'age'",
                                            ))
                                        }
                                    };
                                }
                                "likes_art" => {
                                    likes_art = match map.next_value::<bool>()? {
                                        Some(x) => Ok(x),
                                        None => {
                                            return Err(crate::serde::de::Error::custom(
                                                "missing field 'likes_art'",
                                            ))
                                        }
                                    };
                                }
                                "friends" => {
                                    friends = match map.next_value::<Vec<BluePeriodCharacter>>()? {
                                        Some(x) => Ok(x),
                                        None => {
                                            return Err(crate::serde::de::Error::custom(
                                                "missing field 'friends'",
                                            ))
                                        }
                                    };
                                }
                                _ => {
                                    return Err(crate::serde::de::Error::custom(format!(
                                        "Unknown field '{k}'"
                                    )));
                                }
                            }
                        }

                        Ok(BluePeriodCharacter {
                            name: name?,
                            age: age?,
                            likes_art: likes_art?,
                            friends: friends?,
                        })
                    }
                }

                deserializer.deserialize_map(BluePeriodCharacterVisitor)
            }
        }

        let value = from_str::<BluePeriodCharacter>(
            r#"{
            "name": "Yatora Yaguchi",
            "age": 18,
            "likes_art": true,
            "friends": [
                {
                    "name": "Ryuji Ayukawa",
                    "age": 18,
                    "likes_art": false,
                    "friends": []
                },
                {
                    "name": "Maru Mori",
                    "age": 21,
                    "likes_art": true,
                    "friends": []
                }
            ]
        }"#,
        )
        .unwrap();

        assert_eq!(value.name, "Yatora Yaguchi");
        assert_eq!(value.age, 18);
        assert_eq!(value.likes_art, true);
        assert_eq!(value.friends.len(), 2);

        // Check the friends
        assert_eq!(value.friends[0].name, "Ryuji Ayukawa");
        assert_eq!(value.friends[0].age, 18);
        assert_eq!(value.friends[0].likes_art, false);
        assert_eq!(value.friends[1].name, "Maru Mori");
        assert_eq!(value.friends[1].age, 21);
        assert_eq!(value.friends[1].likes_art, true);
    }

    #[test]
    fn should_impl_deserialize_to_struct() {
        #[derive(Debug, PartialEq)]
        struct MyStruct {
            string: String,
            number: u32,
            boolean: bool,
            array: Vec<MyStruct>,
        }

        impl_deserialize_struct!(MyStruct =>  {
            string: String,
            number: u32,
            boolean: bool,
            array: Vec<MyStruct>
        });

        let json = r#"
        {
            "string": "test",
            "number": 42,
            "boolean": true,
            "array": [
                {
                    "string": "nested",
                    "number": 7,
                    "boolean": false,
                    "array": []
                }
            ]
        }
        "#;

        let result = from_str::<MyStruct>(json).unwrap();

        let expected = MyStruct {
            string: "test".to_string(),
            number: 42,
            boolean: true,
            array: vec![MyStruct {
                string: "nested".to_string(),
                number: 7,
                boolean: false,
                array: vec![],
            }],
        };

        assert_eq!(result, expected);
    }
}
