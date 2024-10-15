use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, Write},
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
    fn new() -> std::io::Result<Self> {
        let _temp_file = TempFile::random()?;
        let file = _temp_file.file().read(true).write(true).open()?;
        Ok(TempFileHandle { _temp_file, file })
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub enum Data {
    Temp(TempFileHandle),
    Memory(Vec<u8>),
}

impl Storage for Data {}

impl Read for Data {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Data::Temp(handle) => std::io::Read::read(&mut handle.file, buf),
            Data::Memory(vec) => std::io::Read::read(&mut vec.as_slice(), buf),
        }
    }
}

impl Write for Data {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Data::Temp(handle) => std::io::Write::write(&mut handle.file, buf),
            Data::Memory(vec) => std::io::Write::write(vec, buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Data::Temp(handle) => std::io::Write::flush(&mut handle.file),
            Data::Memory(vec) => std::io::Write::flush(vec),
        }
    }
}

impl Data {
    pub fn reset(&mut self) {
        match self {
            Data::Temp(handle) => {
                handle.file.seek(std::io::SeekFrom::Start(0)).ok();
            }
            Data::Memory(vec) => {}
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
                    let storage = if is_file {
                        let handle = TempFileHandle::new()
                            .map_err(|err| FormDataError::Other(err.into()))?;
                        Data::Temp(handle)
                    } else {
                        Data::Memory(Vec::new())
                    };

                    let name = field.name().to_owned();
                    let form_field = FormField::new(field, storage)
                        .map_err(|err| FormDataError::Other(err.into()))?;
                    map.insert(name, form_field);
                }
                Ok(None) => break,
                Err(err) => return Err(FormDataError::Other(err.into())),
            }
        }

        Ok(FormMap(map))
    }
}
