use std::{
    collections::HashMap,
    fs::File,
    io::{Cursor, Read},
    ops::Deref,
};

use http1::common::temp_file::TempFile;

use crate::from_request::FromRequest;

use super::{
    form_data::{FormData, FormDataError},
    form_field::{FormField, Storage},
};

#[derive(Debug)]
pub struct TempFileHandle {
    file: File,

    // This file caries the ownership of the file, when drop the actual file will be deleted
    _temp_file: TempFile,
}

impl TempFileHandle {
    pub fn new() -> std::io::Result<Self> {
        let temp_file = TempFile::random()?;
        Self::with_tempfile(temp_file)
    }

    pub fn with_tempfile(temp_file: TempFile) -> std::io::Result<Self> {
        let file = temp_file.file().read(true).write(true).open()?;
        Ok(TempFileHandle {
            _temp_file: temp_file,
            file,
        })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub enum Data {
    Temp(TempFileHandle),
    Memory(Cursor<Vec<u8>>),
}

impl Storage for Data {}

impl Read for Data {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Data::Temp(handle) => std::io::Read::read(&mut handle.file, buf),
            Data::Memory(cursor) => std::io::Read::read(cursor, buf),
        }
    }
}

#[derive(Debug)]
pub struct FormMap(HashMap<String, FormField<Data>>);

impl FormMap {
    pub fn into_iter(self) -> std::collections::hash_map::IntoIter<String, FormField<Data>> {
        self.0.into_iter()
    }
}

impl Deref for FormMap {
    type Target = HashMap<String, FormField<Data>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for FormMap {
    type Rejection = FormDataError;

    fn from_request(
        req: http1::request::Request<http1::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        let mut form_data = FormData::from_request(req)?;
        let mut map = HashMap::new();

        loop {
            match form_data.next_field() {
                Ok(Some(field)) => {
                    let is_file = field.filename().is_some();
                    let name = field.name().to_owned();
                    let filename = field.filename().map(|s| s.to_owned());
                    let content_type = field.filename().map(|s| s.to_owned());

                    let storage = if is_file {
                        let temp_file =
                            TempFile::random().map_err(|err| FormDataError::Other(err.into()))?;

                        let mut file = temp_file
                            .file()
                            .write(true)
                            .open()
                            .map_err(|err| FormDataError::Other(err.into()))?;

                        let bytes = field
                            .bytes()
                            .map_err(|err| FormDataError::Other(err.into()))?;

                        std::io::copy(&mut bytes.as_slice(), &mut file)
                            .map_err(|err| FormDataError::Other(err.into()))?;

                        let handle = TempFileHandle::with_tempfile(temp_file)
                            .map_err(|err| FormDataError::Other(err.into()))?;

                        Data::Temp(handle)
                    } else {
                        let bytes = field
                            .bytes()
                            .map_err(|err| FormDataError::Other(err.into()))?;
                        Data::Memory(Cursor::new(bytes))
                    };

                    let form_field =
                        FormField::from_parts(name.clone(), filename, content_type, storage);
                    map.insert(name, form_field);
                }
                Ok(None) => break,
                Err(err) => return Err(FormDataError::Other(err.into())),
            }
        }

        Ok(FormMap(map))
    }
}
