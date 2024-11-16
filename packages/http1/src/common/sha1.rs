#[derive(Clone)]
pub struct Sha1 {
    state: [u32; 5],
    blocks: Vec<u8>,
    len: u64,
}

impl Sha1 {
    pub fn new() -> Self {
        Sha1 {
            state: [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0],
            blocks: Vec::new(),
            len: 0,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.blocks.extend_from_slice(data);
        self.len += data.len() as u64;
    }

    #[allow(clippy::needless_range_loop)]
    fn process_block(&mut self, block: &[u8; 64]) {
        let mut w = [0u32; 80];

        // Break chunk into sixteen 32-bit big-endian words
        for i in 0..16 {
            w[i] = ((block[i * 4] as u32) << 24)
                | ((block[i * 4 + 1] as u32) << 16)
                | ((block[i * 4 + 2] as u32) << 8)
                | (block[i * 4 + 3] as u32);
        }

        // Extend the sixteen 32-bit words into eighty 32-bit words
        for i in 16..80 {
            let val = w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16];
            w[i] = val.rotate_left(1);
        }

        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];

        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), 0x5A827999),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };

            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);

            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
    }

    pub fn finish(mut self) -> Vec<u8> {
        let len = self.len;

        self.blocks.push(0x80);

        while (self.blocks.len() + 8) % 64 != 0 {
            self.blocks.push(0);
        }

        let len_bits = len * 8;
        self.blocks.extend_from_slice(&[
            ((len_bits >> 56) & 0xFF) as u8,
            ((len_bits >> 48) & 0xFF) as u8,
            ((len_bits >> 40) & 0xFF) as u8,
            ((len_bits >> 32) & 0xFF) as u8,
            ((len_bits >> 24) & 0xFF) as u8,
            ((len_bits >> 16) & 0xFF) as u8,
            ((len_bits >> 8) & 0xFF) as u8,
            (len_bits & 0xFF) as u8,
        ]);

        let blocks = std::mem::take(&mut self.blocks);
        for chunk in blocks.chunks_exact(64) {
            let mut block = [0u8; 64];
            block.copy_from_slice(chunk);
            self.process_block(&block);
        }

        let mut result = Vec::with_capacity(20);
        for &word in &self.state {
            result.extend_from_slice(&[
                ((word >> 24) & 0xFF) as u8,
                ((word >> 16) & 0xFF) as u8,
                ((word >> 8) & 0xFF) as u8,
                (word & 0xFF) as u8,
            ]);
        }

        result
    }
}

impl Default for Sha1 {
    fn default() -> Self {
        Self::new()
    }
}

pub fn hash<S: AsRef<[u8]>>(data: S) -> Vec<u8> {
    let mut sha1 = Sha1::new();
    sha1.update(data.as_ref());
    sha1.finish()
}

#[cfg(test)]
mod tests {
    fn to_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    #[test]
    fn should_hash_to_sha1() {
        assert_eq!(
            to_hex(&super::hash("Hello Vi and Cait!")),
            "10eadf96d9e55276dcb88ec9b3cc3a468e57fece"
        );
    }
}
