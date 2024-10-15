use std::{
    fmt::Debug,
    fs::File,
    io::{Cursor, Read},
    path::PathBuf,
};

use http1::common::temp_file::TempFile;

use super::form_data::Field;

/// The backing field that stores the data for a form field.
pub trait Storage: Read {}

/// A file in memory storage.
pub struct Memory(Cursor<Vec<u8>>);
impl Debug for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Memory").field(&self.0.get_ref()).finish()
    }
}

impl Storage for Memory {}
impl Read for Memory {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Read::read(&mut self.0, buf)
    }
}

/// A file in disk storage.
pub struct Disk {
    path: PathBuf,
    file: Option<File>,
}

impl Debug for Disk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Disk").field(&self.path).finish()
    }
}

impl Storage for Disk {}
impl Read for Disk {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let file = match self.file.as_mut() {
            Some(f) => f,
            None => {
                let f = File::open(&self.path)?;
                self.file.get_or_insert(f)
            }
        };

        Read::read(file, buf)
    }
}

/// A temporal file storage.
pub struct TempDisk {
    temp_file: TempFile,
    file: Option<File>,
}

impl Debug for TempDisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TempDisk").field(&self.temp_file).finish()
    }
}

impl Storage for TempDisk {}
impl Read for TempDisk {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let file = match self.file.as_mut() {
            Some(f) => f,
            None => {
                let f = self.temp_file.file().read(true).open()?;
                self.file.get_or_insert(f)
            }
        };

        Read::read(file, buf)
    }
}

/// Represents a form field.
#[derive(Debug)]
pub struct FormField<S> {
    name: String,
    filename: Option<String>,
    content_type: Option<String>,
    storage: S,
}

impl FormField<()> {
    pub fn from_parts<S>(
        name: String,
        filename: Option<String>,
        content_type: Option<String>,
        storage: S,
    ) -> FormField<S>
    where
        S: Storage,
    {
        FormField {
            name,
            filename,
            content_type,
            storage,
        }
    }

    pub fn memory(field: Field<'_>) -> std::io::Result<FormField<Memory>> {
        let name = field.name.clone();
        let filename = field.filename.clone();
        let content_type = field.content_type.clone();
        let bytes = field.bytes()?;
        let storage = Memory(Cursor::new(bytes));

        Ok(FormField {
            name,
            filename,
            content_type,
            storage,
        })
    }

    pub fn disk(
        field: Field<'_>,
        file_path: impl Into<PathBuf>,
    ) -> std::io::Result<FormField<Disk>> {
        let name = field.name.clone();
        let filename = field.filename.clone();
        let content_type = field.content_type.clone();
        let path = file_path.into();

        // Write the contents of the file in the given path
        let mut f = File::create_new(&path)?;
        std::io::copy(&mut field.reader(), &mut f)?;

        // Create storage
        let storage = Disk { path, file: None };

        Ok(FormField {
            name,
            filename,
            content_type,
            storage,
        })
    }

    pub fn temp(field: Field<'_>) -> std::io::Result<FormField<TempDisk>> {
        let name = field.name.clone();
        let filename = field.filename.clone();
        let content_type = field.content_type.clone();
        let temp_file = TempFile::random()?;

        // Write the contents of the file in the given path
        let mut f = temp_file.file().write(true).open()?;
        std::io::copy(&mut field.reader(), &mut f)?;

        // Create storage
        let storage = TempDisk {
            temp_file,
            file: None,
        };

        Ok(FormField {
            name,
            filename,
            content_type,
            storage,
        })
    }
}

impl<S> FormField<S> {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    pub fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }
}

impl<S: Storage> FormField<S> {
    pub fn bytes(self) -> std::io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        self.reader().read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    pub fn text(self) -> std::io::Result<String> {
        let mut buf = String::new();
        self.reader().read_to_string(&mut buf)?;
        Ok(buf)
    }

    pub fn reader(self) -> S {
        self.storage
    }
}
