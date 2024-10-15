use std::{
    fmt::{Debug, Display},
    io::Write,
};

use http1::{
    common::temp_file::{TempFile, TempFileOpen},
    status::StatusCode,
};

use crate::{
    from_request::FromRequest,
    into_response::IntoResponse,
    serde::{
        bytes::BytesBufferDeserializer,
        de::{Deserialize, Deserializer},
        string::{DeserializeFromStr, DeserializeOnlyString},
        visitor::{MapAccess, Visitor},
    },
};

use super::{
    form_data::FormDataError,
    form_field::FormField,
    form_map::{Data, FormMap},
};

pub struct FormFile(TempFile);

impl FormFile {
    pub fn bytes(&self) -> std::io::Result<Vec<u8>> {
        let mut f = self.0.file().read(true).open()?;
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut f, &mut buf)?;
        Ok(buf)
    }

    pub fn text(&self) -> std::io::Result<String> {
        let mut f = self.0.file().read(true).open()?;
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut f, &mut buf)?;
        Ok(buf)
    }

    pub fn file(&self) -> TempFileOpen {
        self.0.file()
    }
}

impl Debug for FormFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FormFile").finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Multipart<T>(pub T);

impl<T> Multipart<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

#[derive(Debug)]
pub enum MultipartError {
    DeserializationError(crate::serde::de::Error),
    FormError(FormDataError),
}

impl Display for MultipartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultipartError::DeserializationError(error) => {
                writeln!(f, "serialization error: {error}")
            }
            MultipartError::FormError(form_error) => writeln!(f, "form data error: {form_error}"),
        }
    }
}

impl IntoResponse for MultipartError {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        eprintln!("{self}");
        StatusCode::UNPROCESSABLE_CONTENT.into_response()
    }
}

impl<T: Deserialize> FromRequest for Multipart<T> {
    type Rejection = MultipartError;

    fn from_request(
        req: http1::request::Request<http1::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        let map = FormMap::from_request(req).map_err(MultipartError::FormError)?;
        let deserializer = MultipartDeserializer(map);

        T::deserialize(deserializer)
            .map(Multipart)
            .map_err(MultipartError::DeserializationError)
    }
}

impl Deserialize for FormFile {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, crate::serde::de::Error> {
        struct FormFileVisitor;
        impl Visitor for FormFileVisitor {
            type Value = FormFile;

            fn expected(&self) -> &'static str {
                "file"
            }

            fn visit_string(self, value: String) -> Result<Self::Value, crate::serde::de::Error> {
                let temp_file = TempFile::random().map_err(crate::serde::de::Error::error)?;
                let mut f = temp_file
                    .file()
                    .write(true)
                    .open()
                    .map_err(crate::serde::de::Error::error)?;

                f.write_all(value.as_bytes())
                    .map_err(crate::serde::de::Error::error)?;

                Ok(FormFile(temp_file))
            }

            fn visit_bytes_buf(
                self,
                bytes: Vec<u8>,
            ) -> Result<Self::Value, crate::serde::de::Error> {
                let temp_file = TempFile::random().map_err(crate::serde::de::Error::error)?;
                let mut f = temp_file
                    .file()
                    .write(true)
                    .open()
                    .map_err(crate::serde::de::Error::error)?;

                f.write_all(&bytes)
                    .map_err(crate::serde::de::Error::error)?;

                Ok(FormFile(temp_file))
            }

            fn visit_bytes_seq<B: crate::serde::visitor::BytesAccess>(
                self,
                mut bytes: B,
            ) -> Result<Self::Value, crate::serde::de::Error> {
                let temp_file = TempFile::random().map_err(crate::serde::de::Error::error)?;
                let mut f = temp_file
                    .file()
                    .write(true)
                    .open()
                    .map_err(crate::serde::de::Error::error)?;

                bytes.next_bytes(&mut f)?;
                Ok(FormFile(temp_file))
            }
        }

        deserializer.deserialize_bytes_seq(FormFileVisitor)
    }
}

pub struct MultipartDeserializer(FormMap);

impl Deserializer for MultipartDeserializer {
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `any`",
        ))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        visitor.visit_unit()
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `bool`",
        ))
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `u8`",
        ))
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `u16`",
        ))
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `u32`",
        ))
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `u64`",
        ))
    }

    fn deserialize_u128<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `u128`",
        ))
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `i8`",
        ))
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `i16`",
        ))
    }

    fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `i32`",
        ))
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `i64`",
        ))
    }

    fn deserialize_i128<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `i128`",
        ))
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `f32`",
        ))
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `f64`",
        ))
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `char`",
        ))
    }

    fn deserialize_string<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `string`",
        ))
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `seq`",
        ))
    }

    fn deserialize_bytes_buf<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `bytes`",
        ))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        visitor.visit_map(FormMapAccess {
            iter: self.0.into_iter(),
            value: None,
        })
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: crate::serde::visitor::Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `option`",
        ))
    }

    fn deserialize_bytes_seq<V>(self, _visitor: V) -> Result<V::Value, crate::serde::de::Error>
    where
        V: Visitor,
    {
        Err(crate::serde::de::Error::custom(
            "cannot deserialize form to `bytes`",
        ))
    }
}

struct FormMapAccess<I> {
    iter: I,
    value: Option<FormField<Data>>,
}

impl<I: Iterator<Item = (String, FormField<Data>)>> MapAccess for FormMapAccess<I> {
    fn next_key<K: crate::serde::de::Deserialize>(
        &mut self,
    ) -> Result<Option<K>, crate::serde::de::Error> {
        match self.iter.next() {
            Some((k, v)) => {
                self.value = Some(v);
                let key = K::deserialize(DeserializeOnlyString(k))?;
                Ok(Some(key))
            }
            None => Ok(None),
        }
    }

    fn next_value<V: crate::serde::de::Deserialize>(
        &mut self,
    ) -> Result<Option<V>, crate::serde::de::Error> {
        match self.value.take() {
            Some(field) => {
                if field.filename().is_some() {
                    let s = field.bytes().map_err(crate::serde::de::Error::error)?;
                    let deserializer = BytesBufferDeserializer(s);
                    let value = V::deserialize(deserializer)?;
                    Ok(Some(value))
                } else {
                    let s = field.text().map_err(crate::serde::de::Error::error)?;
                    let value = V::deserialize(DeserializeFromStr::Str(s))?;
                    Ok(Some(value))
                }
            }
            None => Ok(None),
        }
    }
}
