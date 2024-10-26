use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
};

use rng::random::Alphanumeric;

/// Represents a temporal file which is deleted after drop.
#[derive(Debug)]
pub struct TempFile(PathBuf);

impl TempFile {
    pub fn with_dir(sub_dir: impl AsRef<Path>) -> std::io::Result<Self> {
        Self::create(Some(sub_dir))
    }

    pub fn random() -> std::io::Result<Self> {
        Self::create::<PathBuf>(None)
    }

    fn create<P: AsRef<Path>>(path: Option<P>) -> std::io::Result<Self> {
        let mut temp_path = std::env::temp_dir();
        let file_name = rng::sequence::<Alphanumeric>()
            .take(20)
            .collect::<String>();

        if let Some(p) = path {
            temp_path.push(p.as_ref());

            // Create the directory
            std::fs::create_dir_all(&temp_path)?;
        }

        // Create the file
        temp_path.push(file_name);
        std::fs::File::create_new(&temp_path)?;

        Ok(TempFile(temp_path))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn file(&self) -> TempFileOpen {
        TempFileOpen {
            path: &self.0,
            file: OpenOptions::new(),
        }
    }

    pub fn read(&self) -> std::io::Result<File> {
        self.file().read(true).open()
    }
}

pub struct TempFileOpen<'a> {
    path: &'a Path,
    file: OpenOptions,
}

impl<'a> TempFileOpen<'a> {
    pub fn append(mut self, append: bool) -> Self {
        self.file.append(append);
        self
    }

    pub fn read(mut self, read: bool) -> Self {
        self.file.read(read);
        self
    }

    pub fn write(mut self, write: bool) -> Self {
        self.file.write(write);
        self
    }

    pub fn truncate(mut self, write: bool) -> Self {
        self.file.truncate(write);
        self
    }

    pub fn open(self) -> std::io::Result<File> {
        self.file.open(self.path)
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::TempFile;
    use std::io::{Seek, Write};

    #[test]
    fn should_exists_random_file() {
        let temp_file = TempFile::random().unwrap();

        let p = temp_file.path().to_path_buf();
        assert!(p.exists());
        assert!(p.is_file());
    }

    #[test]
    fn should_create_file_on_subdir() {
        let temp_file = TempFile::with_dir("/sub_dir").unwrap();

        let p = temp_file.path().to_path_buf();
        assert!(p.exists());
        assert!(p.is_file());

        drop(temp_file);
        assert!(p.exists() == false);
    }

    #[test]
    fn should_remove_file_after_drop() {
        let temp_file = TempFile::random().unwrap();

        let p = temp_file.path().to_path_buf();
        drop(temp_file);
        assert!(p.exists() == false);
    }

    #[test]
    fn should_write_and_read() {
        let temp_file = TempFile::random().unwrap();

        let mut f = temp_file.file().write(true).read(true).open().unwrap();
        write!(f, "Hello World!").unwrap();

        f.seek(std::io::SeekFrom::Start(0)).unwrap(); // Move the start of the file
        let text = std::io::read_to_string(f).unwrap();
        assert_eq!(text, "Hello World!");

        let p = temp_file.path().to_path_buf();

        drop(temp_file);
        assert!(p.exists() == false);
    }
}
