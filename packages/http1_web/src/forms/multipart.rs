#![allow(clippy::serde_api_misuse)]

use std::{
    fmt::{Debug, Display},
    io::Write,
};

use http1::{
    common::temp_file::{TempFile, TempFileOpen},
    status::StatusCode,
};

use crate::{from_request::FromRequest, IntoResponse};
use serde::{
    bytes::BytesBufferDeserializer,
    de::{Deserialize, Deserializer},
    string::{DeserializeFromStr, DeserializeOnlyString},
    visitor::{MapAccess, Visitor},
};

use super::{
    form_data::FormDataError,
    form_field::FormField,
    form_map::{Data, FormMap},
};

#[derive(Debug)]
pub struct FormFile {
    name: String,
    filename: Option<String>,
    content_type: Option<String>,
    data: TempFile,
}

impl FormFile {
    // pub fn bytes(&self) -> std::io::Result<Vec<u8>> {
    //     let mut f = self.0.file().read(true).open()?;
    //     let mut buf = Vec::new();
    //     std::io::Read::read_to_end(&mut f, &mut buf)?;
    //     Ok(buf)
    // }

    // pub fn text(&self) -> std::io::Result<String> {
    //     let mut f = self.0.file().read(true).open()?;
    //     let mut buf = String::new();
    //     std::io::Read::read_to_string(&mut f, &mut buf)?;
    //     Ok(buf)
    // }

    // pub fn file(&self) -> TempFileOpen {
    //     self.0.file()
    // }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }

    pub fn file(&self) -> TempFileOpen {
        self.data.file()
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
    DeserializationError(serde::de::Error),
    FormError(FormDataError),
}

impl Display for MultipartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultipartError::DeserializationError(error) => {
                write!(f, "serialization error: {error}")
            }
            MultipartError::FormError(form_error) => write!(f, "form data error: {form_error}"),
        }
    }
}

impl std::error::Error for MultipartError {}

impl IntoResponse for MultipartError {
    fn into_response(self) -> http1::response::Response<http1::body::Body> {
        log::error!("{self}");
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
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, serde::de::Error> {
        struct FormFileVisitor;

        impl Visitor for FormFileVisitor {
            type Value = FormFile;

            fn expected(&self) -> &'static str {
                "file"
            }

            fn visit_map<Map: MapAccess>(
                self,
                mut map: Map,
            ) -> Result<Self::Value, serde::de::Error> {
                let mut name: Result<String, serde::de::Error> =
                    Err(serde::de::Error::error("FormFile `name` not found"));
                let mut filename: Result<Option<String>, serde::de::Error> = Ok(None);
                let mut content_type: Result<Option<String>, serde::de::Error> = Ok(None);
                let mut data: Result<TempFile, serde::de::Error> =
                    Err(serde::de::Error::error("FormFile `data` not found"));

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "name" => {
                            if let Some(x) = map.next_value::<String>()? {
                                name = Ok(x);
                            }
                        }
                        "filename" => {
                            filename = map.next_value::<String>();
                        }
                        "content_type" => {
                            content_type = map.next_value::<String>();
                        }
                        "data" => {
                            if let Some(TempFileData(temp_file)) =
                                map.next_value::<TempFileData>()?
                            {
                                data = Ok(temp_file);
                            }
                        }
                        _ => {
                            return Err(serde::de::Error::error(format!(
                                "unknown FormField: `{key}`"
                            )))
                        }
                    }
                }

                Ok(FormFile {
                    name: name?,
                    filename: filename?,
                    content_type: content_type?,
                    data: data?,
                })
            }
        }

        deserializer.deserialize_map(FormFileVisitor)
    }
}

struct TempFileData(TempFile);

impl Deserialize for TempFileData {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, serde::de::Error> {
        struct TempFileVisitor;
        impl Visitor for TempFileVisitor {
            type Value = TempFileData;

            fn expected(&self) -> &'static str {
                "file"
            }

            fn visit_string(self, value: String) -> Result<Self::Value, serde::de::Error> {
                let temp_file = TempFile::random().map_err(serde::de::Error::error)?;
                let mut f = temp_file
                    .file()
                    .write(true)
                    .open()
                    .map_err(serde::de::Error::error)?;

                f.write_all(value.as_bytes())
                    .map_err(serde::de::Error::error)?;

                Ok(TempFileData(temp_file))
            }

            fn visit_bytes_buf(self, bytes: Vec<u8>) -> Result<Self::Value, serde::de::Error> {
                let temp_file = TempFile::random().map_err(serde::de::Error::error)?;
                let mut f = temp_file
                    .file()
                    .write(true)
                    .open()
                    .map_err(serde::de::Error::error)?;

                f.write_all(&bytes).map_err(serde::de::Error::error)?;

                Ok(TempFileData(temp_file))
            }

            fn visit_bytes_seq<B: serde::visitor::BytesAccess>(
                self,
                mut bytes: B,
            ) -> Result<Self::Value, serde::de::Error> {
                let temp_file = TempFile::random().map_err(serde::de::Error::error)?;
                let mut f = temp_file
                    .file()
                    .write(true)
                    .open()
                    .map_err(serde::de::Error::error)?;

                bytes.next_bytes(&mut f)?;
                Ok(TempFileData(temp_file))
            }
        }

        deserializer.deserialize_bytes_seq(TempFileVisitor)
    }
}

// impl Deserialize for FormFile {
//     fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, serde::de::Error> {
//         struct FormFileVisitor;
//         impl Visitor for FormFileVisitor {
//             type Value = FormFile;

//             fn expected(&self) -> &'static str {
//                 "file"
//             }

//             fn visit_string(self, value: String) -> Result<Self::Value, serde::de::Error> {
//                 let temp_file = TempFile::random().map_err(serde::de::Error::error)?;
//                 let mut f = temp_file
//                     .file()
//                     .write(true)
//                     .open()
//                     .map_err(serde::de::Error::error)?;

//                 f.write_all(value.as_bytes())
//                     .map_err(serde::de::Error::error)?;

//                 Ok(FormFile(temp_file))
//             }

//             fn visit_bytes_buf(self, bytes: Vec<u8>) -> Result<Self::Value, serde::de::Error> {
//                 let temp_file = TempFile::random().map_err(serde::de::Error::error)?;
//                 let mut f = temp_file
//                     .file()
//                     .write(true)
//                     .open()
//                     .map_err(serde::de::Error::error)?;

//                 f.write_all(&bytes).map_err(serde::de::Error::error)?;

//                 Ok(FormFile(temp_file))
//             }

//             fn visit_bytes_seq<B: serde::visitor::BytesAccess>(
//                 self,
//                 mut bytes: B,
//             ) -> Result<Self::Value, serde::de::Error> {
//                 let temp_file = TempFile::random().map_err(serde::de::Error::error)?;
//                 let mut f = temp_file
//                     .file()
//                     .write(true)
//                     .open()
//                     .map_err(serde::de::Error::error)?;

//                 bytes.next_bytes(&mut f)?;
//                 Ok(FormFile(temp_file))
//             }
//         }

//         deserializer.deserialize_bytes_seq(FormFileVisitor)
//     }
// }

pub struct MultipartDeserializer(FormMap);

impl Deserializer for MultipartDeserializer {
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error("cannot deserialize form to `any`"))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        visitor.visit_unit()
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error("cannot deserialize form to `bool`"))
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error("cannot deserialize form to `u8`"))
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error("cannot deserialize form to `u16`"))
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error("cannot deserialize form to `u32`"))
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error("cannot deserialize form to `u64`"))
    }

    fn deserialize_u128<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error("cannot deserialize form to `u128`"))
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error("cannot deserialize form to `i8`"))
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error("cannot deserialize form to `i16`"))
    }

    fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error("cannot deserialize form to `i32`"))
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error("cannot deserialize form to `i64`"))
    }

    fn deserialize_i128<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error("cannot deserialize form to `i128`"))
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error("cannot deserialize form to `f32`"))
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error("cannot deserialize form to `f64`"))
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error("cannot deserialize form to `char`"))
    }

    fn deserialize_string<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form to `string`",
        ))
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error("cannot deserialize form to `seq`"))
    }

    fn deserialize_bytes_buf<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form to `bytes`",
        ))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        visitor.visit_map(FormMapAccess {
            iter: self.0.into_iter(),
            value: None,
        })
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: serde::visitor::Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form to `option`",
        ))
    }

    fn deserialize_bytes_seq<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form to `bytes`",
        ))
    }
}

struct FormMapAccess<I> {
    iter: I,
    value: Option<FormField<Data>>,
}

impl<I: Iterator<Item = (String, FormField<Data>)>> MapAccess for FormMapAccess<I> {
    fn next_key<K: serde::de::Deserialize>(&mut self) -> Result<Option<K>, serde::de::Error> {
        match self.iter.next() {
            Some((k, v)) => {
                self.value = Some(v);
                let key = K::deserialize(DeserializeOnlyString(k))?;
                Ok(Some(key))
            }
            None => Ok(None),
        }
    }

    fn next_value<V: serde::de::Deserialize>(&mut self) -> Result<Option<V>, serde::de::Error> {
        match self.value.take() {
            Some(field) => {
                if field.filename().is_some() {
                    // let deserializer = BytesBufferDeserializer(field.reader());
                    // let value = V::deserialize(deserializer)?;
                    // Ok(Some(value))
                    let deserializer = FormFieldDeserializer(field);
                    let value = V::deserialize(deserializer)?;
                    Ok(Some(value))
                } else {
                    let s = field.text().map_err(serde::de::Error::error)?;
                    let value = V::deserialize(DeserializeFromStr::Str(s))?;
                    Ok(Some(value))
                }
            }
            None => Ok(None),
        }
    }
}

struct FormFieldDeserializer(FormField<Data>);
impl Deserializer for FormFieldDeserializer {
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `any`",
        ))
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `unit`",
        ))
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `bool`",
        ))
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `u8`",
        ))
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `u16`",
        ))
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `u32`",
        ))
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `u64`",
        ))
    }

    fn deserialize_u128<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `u128`",
        ))
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `i8`",
        ))
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `i16`",
        ))
    }

    fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `i32`",
        ))
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `i64`",
        ))
    }

    fn deserialize_i128<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `i128`",
        ))
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `f32`",
        ))
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `f64`",
        ))
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `char`",
        ))
    }

    fn deserialize_string<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `string`",
        ))
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `seq`",
        ))
    }

    fn deserialize_bytes_seq<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `bytes`",
        ))
    }

    fn deserialize_bytes_buf<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `bytes`",
        ))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        visitor.visit_map(FormFieldAccess {
            step: ReadingStep::Name,
            name: Some(self.0.name().to_string()),
            filename: self.0.filename().map(|x| x.to_owned()),
            content_type: self.0.content_type().map(|x| x.to_owned()),
            data: Some(self.0.bytes().unwrap().as_slice()),
        })
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, serde::de::Error>
    where
        V: Visitor,
    {
        Err(serde::de::Error::error(
            "cannot deserialize form file to `option`",
        ))
    }
}

enum ReadingStep {
    Name,
    FileName,
    ContentType,
    Data,
    Finished,
}

struct FormFieldAccess<R> {
    name: Option<String>,
    filename: Option<String>,
    content_type: Option<String>,
    data: Option<R>,
    step: ReadingStep,
}

impl<R: std::io::Read> MapAccess for FormFieldAccess<R> {
    fn next_key<K: Deserialize>(&mut self) -> Result<Option<K>, serde::de::Error> {
        match self.step {
            ReadingStep::Name => {
                K::deserialize(DeserializeFromStr::Str(String::from("name"))).map(Some)
            }
            ReadingStep::FileName => {
                K::deserialize(DeserializeFromStr::Str(String::from("filename"))).map(Some)
            }
            ReadingStep::ContentType => {
                K::deserialize(DeserializeFromStr::Str(String::from("content_type"))).map(Some)
            }
            ReadingStep::Data => {
                K::deserialize(DeserializeFromStr::Str(String::from("data"))).map(Some)
            }
            ReadingStep::Finished => Ok(None),
        }
    }

    fn next_value<V: Deserialize>(&mut self) -> Result<Option<V>, serde::de::Error> {
        match self.step {
            ReadingStep::Name => {
                let _ = std::mem::replace(&mut self.step, ReadingStep::FileName);
                let value = self.name.take().expect("`name` was already read");
                V::deserialize(DeserializeFromStr::Str(value)).map(Some)
            }
            ReadingStep::FileName => {
                let _ = std::mem::replace(&mut self.step, ReadingStep::ContentType);

                if let Some(value) = self.filename.take() {
                    V::deserialize(DeserializeFromStr::Str(value)).map(Some)
                } else {
                    Ok(None)
                }
            }
            ReadingStep::ContentType => {
                let _ = std::mem::replace(&mut self.step, ReadingStep::Data);

                if let Some(value) = self.content_type.take() {
                    V::deserialize(DeserializeFromStr::Str(value)).map(Some)
                } else {
                    Ok(None)
                }
            }
            ReadingStep::Data => {
                let _ = std::mem::replace(&mut self.step, ReadingStep::Finished);
                let data = self.data.take().expect("`data` was already read");
                let deserializer = BytesBufferDeserializer(data);
                let value = V::deserialize(deserializer)?;
                Ok(Some(value))
            }
            ReadingStep::Finished => Ok(None),
        }
    }
}
