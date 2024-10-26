use std::cell::RefCell;

use random::{Random, RandomSequence};

pub mod random;
mod xorshift;

/// A random number generator source.
pub trait Rng {
    /// Returns a random 32 bits number.
    fn next_32(&mut self) -> u32;

    /// Returns a random 64 bits number.
    fn next_64(&mut self) -> u64 {
        let high = self.next_32() as u64;
        let low = self.next_32() as u64;
        (high << 32) | low
    }

    /// Returns a random 128 bits number.
    fn next_128(&mut self) -> u128 {
        let high = self.next_64();
        let low = self.next_64();
        (high as u128) << 64 | low as u128
    }

    /// Fill a buffer with random bytes.
    fn fill(&mut self, buffer: &mut [u8]) {
        let mut i = 0;

        while i < buffer.len() {
            let value = self.next_128();

            // Try to copy at least the entire 128 bits (16 bytes).
            let bytes = (buffer.len() - i).min(16);
            let chunk = &mut buffer[i..(i + bytes)];
            chunk.copy_from_slice(&value.to_be_bytes()[..bytes]);
            i += bytes;
        }
    }
}

thread_local! {
    static RNG: RefCell<xorshift::XorShiftRng128> = RefCell::new(xorshift::XorShiftRng128::new());
}

#[derive(Clone, Copy)]
pub struct LocalRng {
    _priv: (),
}

impl Rng for LocalRng {
    fn next_32(&mut self) -> u32 {
        RNG.with_borrow_mut(|rng| rng.next_32())
    }

    fn next_64(&mut self) -> u64 {
        RNG.with_borrow_mut(|rng| rng.next_64())
    }
}

/// Returns a random number generator source.
pub fn local_rng() -> LocalRng {
    LocalRng { _priv: () }
}

/// Generate a random value of type `T`.
pub fn random<T: random::Random>() -> T::Output {
    let mut rng = local_rng();
    T::random(&mut rng)
}

/// Returns an iterator that generate random values of type `T`.
pub fn sequence<T: random::Random>() -> RandomSequence<LocalRng, T> {
    let rng = local_rng();
    T::sequence(rng)
}

/// Returns a value within the given range.
pub fn random_range<T>(rng: &mut impl Rng, min: T, max: T) -> T
where
    T: Random<Output = T>
        + PartialOrd
        + std::ops::Sub<Output = T>
        + std::ops::Add<Output = T>
        + std::ops::Rem<Output = T>
        + std::fmt::Debug
        + std::fmt::Display
        + Copy,
{
    assert!(min < max, "max must be greater than min: {max} < {min}");
    let n = T::random(rng);
    min + (n % (max - min))
}

#[cfg(test)]
mod tests {
    use super::{local_rng, Rng};

    #[test]
    fn should_fill_specific_buffer_sizes() {
        let mut rng = local_rng();

        // Test an empty buffer
        let mut buffer_empty = [];
        rng.fill(&mut buffer_empty);
        assert!(buffer_empty.is_empty(), "Empty buffer should remain empty.");

        // Test a 1-byte buffer
        let mut buffer_1 = [0u8; 1];
        rng.fill(&mut buffer_1);
        assert!(
            buffer_1.iter().any(|&b| b != 0),
            "1-byte buffer should contain random bytes."
        );

        // Test a 2-byte buffer
        let mut buffer_2 = [0u8; 2];
        rng.fill(&mut buffer_2);
        assert!(
            buffer_2.iter().any(|&b| b != 0),
            "2-byte buffer should contain random bytes."
        );

        // Test a 32-byte buffer
        let mut buffer_32 = [0u8; 32];
        rng.fill(&mut buffer_32);
        assert!(
            buffer_32.iter().any(|&b| b != 0),
            "32-byte buffer should contain random bytes."
        );

        // Test a 1024-byte buffer
        let mut buffer_1024 = [0u8; 1024];
        rng.fill(&mut buffer_1024);
        assert!(
            buffer_1024.iter().any(|&b| b != 0),
            "1024-byte buffer should contain random bytes."
        );
    }
}
