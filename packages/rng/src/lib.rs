use std::cell::RefCell;

use random::{Random, RandomSequence};

pub mod random;
mod xorshift;

/// A random number generator source.
pub trait Rng {
    /// Fill a buffer with random bytes.
    fn fill_bytes(&mut self, buf: &mut [u8]);
}

thread_local! {
    static RNG: RefCell<xorshift::XorShiftRng128> = RefCell::new(xorshift::XorShiftRng128::new());
}

#[derive(Clone, Copy)]
pub struct LocalRng {
    _priv: (),
}

impl Rng for LocalRng {
    fn fill_bytes(&mut self, buf: &mut [u8]) {
        RNG.with_borrow_mut(|rng| rng.fill_bytes(buf))
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
        rng.fill_bytes(&mut buffer_empty);
        assert!(buffer_empty.is_empty(), "Empty buffer should remain empty.");

        // Test a 1-byte buffer
        let mut buffer_1 = [0u8; 1];
        rng.fill_bytes(&mut buffer_1);
        assert!(
            buffer_1.iter().any(|&b| b != 0),
            "1-byte buffer should contain random bytes."
        );

        // Test a 2-byte buffer
        let mut buffer_2 = [0u8; 2];
        rng.fill_bytes(&mut buffer_2);
        assert!(
            buffer_2.iter().any(|&b| b != 0),
            "2-byte buffer should contain random bytes."
        );

        // Test a 32-byte buffer
        let mut buffer_32 = [0u8; 32];
        rng.fill_bytes(&mut buffer_32);
        assert!(
            buffer_32.iter().any(|&b| b != 0),
            "32-byte buffer should contain random bytes."
        );

        // Test a 1024-byte buffer
        let mut buffer_1024 = [0u8; 1024];
        rng.fill_bytes(&mut buffer_1024);
        assert!(
            buffer_1024.iter().any(|&b| b != 0),
            "1024-byte buffer should contain random bytes."
        );
    }
}
