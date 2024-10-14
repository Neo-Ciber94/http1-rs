use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
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
    _temp_file: TempFile,
}

impl TempFileHandle {
    fn new() -> std::io::Result<Self> {
        let _temp_file = TempFile::random()?;
        let file = _temp_file.file().read(true).write(true).open()?;
        Ok(TempFileHandle { _temp_file, file })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub enum FieldStorage {
    Temp(TempFileHandle),
    Memory(Vec<u8>),
}

impl Storage for FieldStorage {}

impl Read for FieldStorage {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            FieldStorage::Temp(handle) => std::io::Read::read(&mut handle.file, buf),
            FieldStorage::Memory(vec) => std::io::Read::read(&mut vec.as_slice(), buf),
        }
    }
}

impl Write for FieldStorage {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            FieldStorage::Temp(handle) => std::io::Write::write(&mut handle.file, buf),
            FieldStorage::Memory(vec) => std::io::Write::write(vec, buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            FieldStorage::Temp(handle) => std::io::Write::flush(&mut handle.file),
            FieldStorage::Memory(vec) => std::io::Write::flush(vec),
        }
    }
}

pub struct FormMap(HashMap<String, FormField<FieldStorage>>);

impl Deref for FormMap {
    type Target = HashMap<String, FormField<FieldStorage>>;

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
                    let storage = if is_file {
                        let handle = TempFileHandle::new().map_err(|err| FormDataError::Other(err.into()))?;
                        FieldStorage::Temp(handle)
                    } else {
                        FieldStorage::Memory(Vec::new())
                    };

                    let name = field.name().to_owned();
                    let form_field = FormField::new(field, storage).map_err(|err| FormDataError::Other(err.into()))?;
                    map.insert(name, form_field);
                }
                Ok(None) => break,
                Err(err) => return Err(FormDataError::Other(err.into())),
            }
        }

        Ok(FormMap(map))
    }
}
