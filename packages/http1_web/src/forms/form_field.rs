use std::{fs::File, io::Read, path::PathBuf};

use http1::common::temp_file::TempFile;

use super::form_data::Field;

pub trait Storage: Read {}

/// A file in memory storage.
pub struct Memory(Vec<u8>);

impl Storage for Memory {}
impl Read for Memory {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Read::read(&mut self.0.as_slice(), buf)
    }
}

/// A file in disk storage.
pub struct Disk(PathBuf);

impl Storage for Disk {}
impl Read for Disk {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut file = File::open(&self.0)?;
        Read::read(&mut file, buf)
    }
}

/// A temporal file storage.
pub struct TempDisk(TempFile);

impl Storage for TempDisk {}
impl Read for TempDisk {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut file = self.0.file().open()?;
        Read::read(&mut file, buf)
    }
}

/// Represents a form field.
pub struct FormField<S> {
    name: String,
    filename: Option<String>,
    content_type: Option<String>,
    storage: S,
}

impl FormField<()> {
    pub fn memory(field: Field<'_>) -> std::io::Result<FormField<Memory>> {
        let name = field.name.clone();
        let filename = field.filename.clone();
        let content_type = field.content_type.clone();
        let bytes = field.bytes()?;
        let storage = Memory(bytes);

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
        let file_path = file_path.into();

        // Write the contents of the file in the given path
        let mut f = File::create_new(&file_path)?;
        std::io::copy(&mut field.reader(), &mut f)?;

        // Create storage
        let storage = Disk(file_path);

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
        let temp = TempFile::random()?;

        // Write the contents of the file in the given path
        let mut f = temp.file().write(true).open()?;
        std::io::copy(&mut field.reader(), &mut f)?;

        // Create storage
        let storage = TempDisk(temp);

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
