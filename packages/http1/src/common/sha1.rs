#[derive(Clone)]
pub struct Sha1 {}

impl Sha1 {
    pub fn new() -> Self {
        Sha1 {}
    }

    pub fn update(&mut self, data: &[u8]) {
        todo!()
    }

    pub fn finish(self) -> Vec<u8> {
        todo!()
    }
}

pub fn hash<S: AsRef<[u8]>>(data: S) -> Vec<u8> {
    let mut sha1 = Sha1::new();
    sha1.update(data.as_ref());
    sha1.finish()
}
