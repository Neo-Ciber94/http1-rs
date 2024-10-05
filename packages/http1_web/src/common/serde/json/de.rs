use std::io::{BufReader, Read};

use crate::common::serde::de::{Deserializer, Error};

use super::number::Number;

pub struct JsonDeserializer<R> {
    reader: BufReader<R>,
    next: Option<u8>,
}

impl<R: Read> JsonDeserializer<R> {
    fn peek(&mut self) -> Option<u8> {
        match self.next {
            Some(b) => Some(b),
            None => {
                let next = self.next_byte();
                self.next = next;
                next
            }
        }
    }

    fn next_byte(&mut self) -> Option<u8> {
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

    fn parse_unit(&mut self) -> Result<(), Error> {
        self.parse_null()
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

            match self.next_byte() {
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
            let _ = self.next_byte();
        }

        loop {
            match self.next_byte() {
                None => return Err(Error::custom("expected next string char")),
                Some(byte) => {
                    match byte {
                        b'\\' => {
                            // handle escaping
                        }
                        b'"' => break,
                        _ => {}
                    }

                    s.push(byte as char);
                }
            }
        }

        Ok(s)
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
        self.parse_unit()?;
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

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, crate::common::serde::de::Error>
    where
        V: crate::common::serde::visitor::Visitor,
    {
        todo!()
    }
}
