use std::io::{BufReader, Read};

use crate::common::serde::{
    de::{Deserializer, Error},
    visitor::{MapAccess, SeqAccess},
};

use super::{
    map::OrderedMap,
    number::Number,
    value::{JsonValue, JsonValueDeserializer},
};

pub struct JsonDeserializer<R> {
    reader: BufReader<R>,
    next: Option<u8>,
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
            Ok(0) => None,
            Ok(_) => Some(buf[0]),
            Err(_) => None,
        }
    }

    fn read_until_next_non_whitespace(&mut self) -> Option<u8> {
        loop {
            match self.peek() {
                Some(b) => {
                    if !b.is_ascii_whitespace() {
                        return Some(b);
                    }

                    let _ = self.read_byte();
                }
                None => return None,
            }
        }
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

    fn parse_json(&mut self) -> Result<JsonValue, Error> {
        match self.read_until_next_non_whitespace() {
            Some(b) => match b {
                b't' | b'f' => self.parse_bool().map(JsonValue::Bool),
                b'n' => self.parse_null().map(|_| JsonValue::Null),
                b'-' | b'0'..b'9' => self.parse_number().map(JsonValue::Number),
                b'"' => self.parse_string().map(JsonValue::String),
                b'[' => self.parse_array().map(JsonValue::Array),
                b'{' => self.parse_object().map(JsonValue::Object),
                _ => Err(Error::custom(format!(
                    "unexpected json token {}",
                    b as char,
                ))),
            },
            None => Err(Error::custom("empty value")),
        }
    }

    fn parse_null(&mut self) -> Result<(), Error> {
        let buf = &mut [0; 4];
        match self.reader.read(buf) {
            Ok(4) if buf == b"null" => Ok(()),
            Ok(_) => Err(Error::custom("expected 'null'")),
            Err(err) => Err(Error::error(err)),
        }
    }

    fn parse_bool(&mut self) -> Result<bool, Error> {
        let buf = &mut [0; 5];
        match self.reader.read(buf) {
            Ok(4) if buf.as_slice() == b"true" => Ok(true),
            Ok(5) if buf.as_slice() == b"false" => Ok(false),
            Ok(_) => Err(Error::custom("expected 'boolean'")),
            Err(err) => Err(Error::error(err)),
        }
    }

    fn parse_number(&mut self) -> Result<Number, Error> {
        let mut s = String::with_capacity(2);

        loop {
            if self.peek().is_none() {
                break;
            }

            match self.read_byte() {
                None => return Err(Error::custom("expected number")),
                Some(byte) => {
                    match byte {
                        b'0'..b'9' | b'-' | b'.' | b'e' => {
                            if s.len() > 1 {
                                return Err(Error::custom("expected number sign"));
                            }

                            if byte == b'.' && s.contains(".") {
                                return Err(Error::custom("invalid decimal number"));
                            }
                        }
                        _ => break,
                    }

                    s.push(byte as char);
                }
            }
        }

        if s.contains(".") || s.contains("e") {
            let f: f64 = s.parse().map_err(Error::error)?;
            return Ok(Number::from(f));
        }

        let is_negative = s.starts_with("-");

        if is_negative {
            let i: u128 = s.parse().map_err(Error::error)?;
            Ok(Number::from(i))
        } else {
            let u: i128 = s.parse().map_err(Error::error)?;
            Ok(Number::from(u))
        }
    }

    fn parse_string(&mut self) -> Result<String, Error> {
        let mut s = String::with_capacity(2);

        if self.peek() == Some(b'"') {
            let _ = self.read_byte();
        } else {
            return Err(Error::custom("expected start of string"));
        }

        loop {
            match self.read_byte() {
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
                    b'"' => break,
                    _ => {
                        s.push(byte as char);
                    }
                },
            }
        }

        Ok(s)
    }

    fn parse_array(&mut self) -> Result<Vec<JsonValue>, Error> {
        self.read_until_byte(b'[')?;
        self.read_byte(); // discard the `[`

        let mut vec = vec![];

        loop {
            let item = self.parse_json()?;
            vec.push(item);

            match self.read_until_next_non_whitespace() {
                Some(b',') => {
                    self.read_byte(); // read next
                }
                Some(b']') => {
                    self.read_byte(); // end
                    break;
                }
                Some(b) => {
                    return Err(Error::custom(format!(
                        "invalid array element token {}",
                        b as char
                    )))
                }
                None => return Err(Error::custom("expected array element but was empty")),
            }
        }

        Ok(vec)
    }

    fn parse_object(&mut self) -> Result<OrderedMap<String, JsonValue>, Error> {
        self.read_until_byte(b'{')?;
        self.read_byte(); // discard the `{`

        let mut map = OrderedMap::new();

        loop {
            let key = self.parse_string()?;
            self.read_until_byte(b':')?;
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

        Ok(map)
    }
}

impl<R: Read> Deserializer for JsonDeserializer<R> {
    fn deserialize_unit<V>(
        mut self,
        visitor: V,
    ) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        self.parse_null()?;
        visitor.visit_unit()
    }

    fn deserialize_bool<V>(
        mut self,
        visitor: V,
    ) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        let value = self.parse_bool()?;
        visitor.visit_bool(value)
    }

    fn deserialize_u8<V>(mut self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_u8()
            .ok_or_else(|| Error::custom("u8 was expected"))?;
        visitor.visit_u8(value)
    }

    fn deserialize_u16<V>(mut self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_u16()
            .ok_or_else(|| Error::custom("u16 was expected"))?;
        visitor.visit_u16(value)
    }

    fn deserialize_u32<V>(mut self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_u32()
            .ok_or_else(|| Error::custom("u32 was expected"))?;
        visitor.visit_u32(value)
    }

    fn deserialize_u64<V>(mut self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_u64()
            .ok_or_else(|| Error::custom("u64 was expected"))?;
        visitor.visit_u64(value)
    }

    fn deserialize_u128<V>(
        mut self,
        visitor: V,
    ) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_u128()
            .ok_or_else(|| Error::custom("u128 was expected"))?;
        visitor.visit_u128(value)
    }

    fn deserialize_i8<V>(mut self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_i8()
            .ok_or_else(|| Error::custom("i8 was expected"))?;
        visitor.visit_i8(value)
    }

    fn deserialize_i16<V>(mut self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_i16()
            .ok_or_else(|| Error::custom("i16 was expected"))?;
        visitor.visit_i16(value)
    }

    fn deserialize_i32<V>(mut self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_i32()
            .ok_or_else(|| Error::custom("i32 was expected"))?;
        visitor.visit_i32(value)
    }

    fn deserialize_i64<V>(mut self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_i64()
            .ok_or_else(|| Error::custom("i64 was expected"))?;
        visitor.visit_i64(value)
    }

    fn deserialize_i128<V>(
        mut self,
        visitor: V,
    ) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_i128()
            .ok_or_else(|| Error::custom("i128 was expected"))?;
        visitor.visit_i128(value)
    }

    fn deserialize_f32<V>(mut self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_f32()
            .ok_or_else(|| Error::custom("f32 was expected"))?;
        visitor.visit_f32(value)
    }

    fn deserialize_f64<V>(mut self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        let number = self.parse_number()?;
        let value = number
            .as_f64()
            .ok_or_else(|| Error::custom("f64 was expected"))?;
        visitor.visit_f64(value)
    }

    fn deserialize_char<V>(
        mut self,
        visitor: V,
    ) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        let mut string = self.parse_string()?;

        if string.len() > 1 {
            return Err(Error::custom(format!("expected char but was `{string}`")));
        }

        let c = string.pop().unwrap();
        visitor.visit_char(c)
    }

    fn deserialize_string<V>(
        mut self,
        visitor: V,
    ) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        let string = self.parse_string()?;
        visitor.visit_string(string)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        let seq = self.parse_array()?;
        visitor.visit_seq(JsonSeqAccess(seq.into_iter()))
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        let map = self.parse_object()?;
        visitor.visit_map(JsonObjectAccess(map.into_iter()))
    }
}

struct JsonSeqAccess(std::vec::IntoIter<JsonValue>);
impl SeqAccess for JsonSeqAccess {
    fn next_element<T: crate::common::serde::de::Deserialize>(
        &mut self,
    ) -> Result<Option<T>, Error> {
        match self.0.next() {
            Some(x) => {
                let value = T::deserialize(JsonValueDeserializer(x))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}

struct JsonObjectAccess<I: Iterator<Item = (String, JsonValue)>>(I);
impl<I: Iterator<Item = (String, JsonValue)>> MapAccess for JsonObjectAccess<I> {
    fn next_entry<
        K: crate::common::serde::de::Deserialize,
        V: crate::common::serde::de::Deserialize,
    >(
        &mut self,
    ) -> Result<Option<(K, V)>, Error> {
        match self.0.next() {
            Some((k, v)) => {
                let key = K::deserialize(JsonValueDeserializer(JsonValue::String(k)))?;
                let value = V::deserialize(JsonValueDeserializer(v))?;
                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }
}
