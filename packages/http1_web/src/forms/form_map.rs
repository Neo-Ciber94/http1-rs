use std::{
    collections::HashMap,
    fmt::Debug,
    fs::File,
    io::{Cursor, Read},
    ops::{Deref, DerefMut},
};

use http1::common::temp_file::TempFile;

use crate::from_request::FromRequest;

use super::{
    form_data::{FormData, FormDataError},
    form_field::{FormField, Storage},
    one_or_many::OneOrMany,
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

/// One or many form fields.
pub type FormFields = OneOrMany<FormField<Data>>;

#[derive(Debug)]
pub struct FormMap(HashMap<String, FormFields>);

impl IntoIterator for FormMap {
    type Item = (String, FormFields);
    type IntoIter = std::collections::hash_map::IntoIter<String, FormFields>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Deref for FormMap {
    type Target = HashMap<String, FormFields>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FormMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromRequest for FormMap {
    type Rejection = FormDataError;

    fn from_request(
        req: &http1::request::Request<()>,
        payload: &mut http1::payload::Payload,
    ) -> Result<Self, Self::Rejection> {
        let mut form_data = FormData::from_request(req, payload)?;
        let mut map = HashMap::<String, FormFields>::new();

        loop {
            match form_data.next_field() {
                Ok(Some(field)) => {
                    let is_file = field.filename().is_some();
                    let name = field.name().to_owned();
                    let filename = field.filename().map(|s| s.to_owned());
                    let content_type = field.content_type().map(|s| s.to_owned());

                    let storage = if is_file {
                        let temp_file =
                            TempFile::random().map_err(|err| FormDataError::Other(err.into()))?;

                        let mut file = temp_file
                            .file()
                            .write(true)
                            .open()
                            .map_err(|err| FormDataError::Other(err.into()))?;

                        let mut data = field.reader();

                        std::io::copy(&mut data, &mut file)
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

                    match map.entry(name.clone()) {
                        std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
                            occupied_entry.get_mut().insert(form_field);
                        }
                        std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                            vacant_entry.insert(OneOrMany::One(form_field));
                        }
                    }
                }
                Ok(None) => break,
                Err(err) => return Err(FormDataError::Other(err.into())),
            }
        }

        Ok(FormMap(map))
    }
}
