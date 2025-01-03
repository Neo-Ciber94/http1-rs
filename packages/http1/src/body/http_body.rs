use std::{
    convert::Infallible,
    fs::File,
    io::{BufReader, Chain, Cursor, Empty, Read, Take},
    sync::Arc,
};

/// Represents an object that can be used to read the request or response body data.
pub trait HttpBody {
    type Err;
    type Data: Into<Vec<u8>>;

    /// Returns the next chunk of data.
    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err>;

    /// Returns the total size of the data to write.
    fn size_hint(&self) -> Option<usize> {
        None
    }

    /// Read all the chunks and returns a `Vec` containing all the bytes.
    fn read_all_bytes(&mut self) -> Result<Vec<u8>, Self::Err> {
        let capacity = self.size_hint().unwrap_or(4);
        let mut bytes = Vec::with_capacity(capacity);

        while let Some(chunk) = self.read_next()? {
            let chunk = chunk.into();
            bytes.extend(chunk);
        }

        Ok(bytes)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct BytesBuf<T>(Option<T>);

impl<T> BytesBuf<T> {
    pub fn new(data: T) -> Self {
        BytesBuf(Some(data))
    }
}

impl<T: AsRef<[u8]>> HttpBody for BytesBuf<T> {
    type Err = Infallible;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        match self.0.take() {
            Some(data) => Ok(Some(data.as_ref().to_vec())),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.0.as_ref().map(|x| x.as_ref().len())
    }
}

impl HttpBody for Empty {
    type Err = Infallible;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        Ok(None)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(0)
    }
}

impl HttpBody for () {
    type Err = Infallible;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        Ok(None)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(0)
    }
}

impl<R> HttpBody for BufReader<R>
where
    R: Read,
{
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        read_next_bytes(self)
    }
}

impl<R> HttpBody for Box<R>
where
    R: Read,
{
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        read_next_bytes(self)
    }
}

impl HttpBody for File {
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        read_next_bytes(self)
    }

    fn size_hint(&self) -> Option<usize> {
        self.metadata().map(|m| m.len() as usize).ok()
    }
}

impl HttpBody for Arc<File> {
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        read_next_bytes(self)
    }

    fn size_hint(&self) -> Option<usize> {
        self.metadata().map(|m| m.len() as usize).ok()
    }
}

impl<R: Read> HttpBody for Take<R> {
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        read_next_bytes(self)
    }
}

impl<T: Read, U: Read> HttpBody for Chain<T, U> {
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        read_next_bytes(self)
    }
}

impl<T: AsRef<[u8]>> HttpBody for Cursor<T> {
    type Err = std::io::Error;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        read_next_bytes(self)
    }
}

impl HttpBody for Vec<u8> {
    type Err = Infallible;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        if self.is_empty() {
            Ok(None)
        } else {
            let bytes = std::mem::take(self);
            Ok(Some(bytes))
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl HttpBody for String {
    type Err = Infallible;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        if self.is_empty() {
            Ok(None)
        } else {
            let s = std::mem::take(self);
            let bytes = s.into_bytes();
            Ok(Some(bytes))
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.as_bytes().len())
    }
}

impl<'a> HttpBody for &'a str {
    type Err = Infallible;
    type Data = Vec<u8>;

    fn read_next(&mut self) -> Result<Option<Self::Data>, Self::Err> {
        if self.is_empty() {
            Ok(None)
        } else {
            let s = std::mem::take(self);
            let bytes = s.as_bytes().to_vec();
            Ok(Some(bytes))
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.as_bytes().len())
    }
}

fn read_next_bytes<R: Read>(read: &mut R) -> std::io::Result<Option<Vec<u8>>> {
    const BUFFER_SIZE: usize = 4 * 1024; // 4KB

    let mut buf = vec![0; BUFFER_SIZE];
    match Read::read(read, &mut buf) {
        Ok(0) => Ok(None),
        Ok(n) => Ok(Some(buf[0..n].to_vec())),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{BufReader, Cursor, Empty, Write};
    use std::path::PathBuf;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;
    use std::time::UNIX_EPOCH;

    use crate::body::chunked_body::ChunkedBody;
    use crate::body::http_body::{BytesBuf, HttpBody};
    use crate::body::Body;

    fn read_all_body_data(body: &mut Body) -> Vec<u8> {
        let mut all_data = Vec::new();
        while let Ok(Some(chunk)) = body.read_next() {
            all_data.extend(chunk);
        }
        all_data
    }

    fn get_random_temp_file_path() -> PathBuf {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

        let temp_dir = std::env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let file_name = format!("{timestamp}{id}");
        temp_dir.join(file_name)
    }

    #[test]
    fn should_read_bytes_body() {
        let bytes = BytesBuf::new(b"hello".to_vec());
        let mut body = Body::new(bytes);

        assert_eq!(body.size_hint(), Some(5));
        let result = read_all_body_data(&mut body);
        assert_eq!(result, b"hello");
    }

    #[test]
    fn should_read_empty_body() {
        let mut body = Body::new(Empty::default());

        assert_eq!(body.size_hint(), Some(0));
        let result = read_all_body_data(&mut body);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn should_read_unit_body() {
        let mut body = Body::new(());

        assert_eq!(body.size_hint(), Some(0));
        let result = read_all_body_data(&mut body);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn should_read_chunked_body() {
        let (stream, sender) = ChunkedBody::new();

        std::thread::spawn(move || {
            sender.send("Hello World!").unwrap();
        })
        .join()
        .unwrap();

        let mut body = Body::new(stream);
        assert_eq!(body.size_hint(), None);

        let result = read_all_body_data(&mut body);
        assert_eq!(result, b"C\r\nHello World!\r\n0\r\n\r\n");
    }

    #[test]
    fn should_read_cursor_body() {
        let data = vec![1, 2, 3, 4, 5];
        let cursor = Cursor::new(data.clone());
        let mut body = Body::new(cursor);

        assert_eq!(body.size_hint(), None);
        let result = read_all_body_data(&mut body);
        assert_eq!(result, data);
    }

    #[test]
    fn should_read_bufreader_body() {
        let data = vec![1, 2, 3, 4, 5];
        let reader = BufReader::new(Cursor::new(data.clone()));
        let mut body = Body::new(reader);

        assert_eq!(body.size_hint(), None);
        let result = read_all_body_data(&mut body);
        assert_eq!(result, data);
    }

    #[test]
    fn should_read_file_body() {
        // Create a temporary file with some data
        let path = "temp_file.txt";
        {
            let mut file = File::create(path).unwrap();
            writeln!(file, "Hello, world!").unwrap();
        }

        let file = File::open(path).unwrap();
        let mut body = Body::new(file);

        assert_eq!(body.size_hint(), Some(14));
        let result = read_all_body_data(&mut body);
        assert_eq!(result, b"Hello, world!\n");

        // Clean up the temporary file
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn should_read_arc_file_body() {
        // Create a temporary file with some data
        let file_path = get_random_temp_file_path();
        {
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, "Hello, world!").unwrap();
        }

        let file = Arc::new(File::open(&file_path).unwrap());
        let mut body = Body::new(file);

        assert_eq!(body.size_hint(), Some(14));
        let result = read_all_body_data(&mut body);
        assert_eq!(result, b"Hello, world!\n");

        // Clean up the temporary file
        std::fs::remove_file(&file_path).unwrap();
    }

    #[test]
    fn should_read_vec_body() {
        let data = vec![1, 2, 3, 4, 5];
        let mut body = Body::from(data.clone());

        assert_eq!(body.size_hint(), Some(5));
        let result = read_all_body_data(&mut body);
        assert_eq!(result, data);
    }

    #[test]
    fn should_read_string_body() {
        let mut body = Body::from("Hello".to_string());

        assert_eq!(body.size_hint(), Some(5));
        let result = read_all_body_data(&mut body);
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn should_read_str_body() {
        let mut body = Body::from("Hello, world!");

        assert_eq!(body.size_hint(), Some(13));
        let result = read_all_body_data(&mut body);
        assert_eq!(result, b"Hello, world!");
    }
}
