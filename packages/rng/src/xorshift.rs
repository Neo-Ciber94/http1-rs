use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::{SystemTime, UNIX_EPOCH},
};

use super::Rng;

#[derive(Debug)]
pub struct XorShiftRng128([u32; 4]);

impl XorShiftRng128 {
    pub fn new() -> Self {
        let state = get_seed();
        XorShiftRng128::with_state(state)
    }

    pub fn with_state(state: u128) -> Self {
        assert!(state != 0, "state cannot be zero");

        let mut s: [u32; 4] = [0, 0, 0, 0];
        s[0] = (state >> 96) as u32;
        s[1] = (state >> 64) as u32;
        s[2] = (state >> 32) as u32;
        s[3] = (state & 0xFFFFFFFF) as u32;

        XorShiftRng128(s)
    }

    #[rustfmt::skip]
    pub fn next(&mut self) -> u128 {
        let state = &mut self.0;
        let mut t = state[3];

        let s = state[0];
        state[3] = state[2];
        state[2] = state[1];
        state[1] = s;

        t ^= t << 11;
        t ^= t >> 8;

        state[0] = t ^ s ^ (s >> 19);

        // Convert state to u128
        ((state[0] as u128) << 96) | ((state[1] as u128) << 64) | ((state[2] as u128) << 32) | (state[3] as u128)
    }
}

impl Rng for XorShiftRng128 {
    fn fill_bytes(&mut self, buf: &mut [u8]) {
        let mut pos = 0;

        while pos < buf.len() {
            let bytes = self.next().to_ne_bytes();
            let count = bytes.len().min(buf.len() - pos);

            let src = &bytes[..count];
            let dst = &mut buf[pos..(pos + count)];
            dst.copy_from_slice(src);
            pos += bytes.len();
        }
    }
}

fn get_seed() -> u128 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards");
    let ms = duration.as_millis();
    let ns = duration.subsec_nanos() as u128;
    let timestamp = (ms << 64) | ns;

    let mut hasher = DefaultHasher::new();

    // Process & thread
    std::process::id().hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);

    // Pointer
    (&0 as *const i32).hash(&mut hasher);

    // Mix
    let hash = hasher.finish() as u128;
    timestamp ^ hash
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::XorShiftRng128;

    #[test]
    fn should_produce_same_result_for_different_states() {
        let mut rng1 = XorShiftRng128::with_state(12345);
        let mut rng2 = XorShiftRng128::with_state(12345);

        assert_eq!(rng1.next(), rng2.next());
        assert_eq!(rng1.next(), rng2.next());
        assert_eq!(rng1.next(), rng2.next());
        assert_eq!(rng1.next(), rng2.next());
    }

    #[test]
    fn should_produce_unique_values() {
        let mut rng = XorShiftRng128::new();

        let values = (0..1000).map(|_| rng.next()).collect::<HashSet<_>>();
        assert_eq!(values.len(), 1000);
    }
}
