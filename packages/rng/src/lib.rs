use std::{cell::RefCell, ops::RangeBounds};

mod pick;
mod random;
mod shuffle;
mod xorshift;

pub use {pick::*, random::*, shuffle::*};

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

/// Gets a random value from the given source.
pub fn pick<T: Pick>(source: &T) -> Option<&T::Output> {
    let mut rng = local_rng();
    source.pick(&mut rng)
}

/// Reorder the values of the value in place.
pub fn shuffle_in_place<T: Shuffle>(value: &mut T) {
    let mut rng = local_rng();
    value.shuffle_in_place(&mut rng);
}

/// Reorder the values of the value.
pub fn shuffle<T: Shuffle>(value: T) -> T {
    let mut rng = local_rng();
    value.shuffle(&mut rng)
}

#[doc(hidden)]
pub trait Numeric: Sized {
    fn one() -> Self;
    fn min_value() -> Self;
    fn max_value() -> Self;
}

macro_rules! impl_numeric {
    ($($T:ident),*) => {
        $(
            impl Numeric for $T {
                fn one() -> Self {
                    1 as Self
                }

                fn min_value() -> Self {
                    $T::MIN
                }

                fn max_value() -> Self {
                    $T::MAX
                }
            }
        )*
    };
}

impl_numeric!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);

/// Gets a random number in the given range.
pub fn range<T>(range: impl RangeBounds<T>) -> T
where
    T: Random<Output = T>
        + PartialOrd
        + std::ops::Sub<Output = T>
        + std::ops::Add<Output = T>
        + std::ops::Rem<Output = T>
        + std::fmt::Debug
        + std::fmt::Display
        + Numeric
        + Clone,
{
    let mut rng = local_rng();
    range_with(range, &mut rng)
}

/// Gets a random number in the given range using the specified `Rng`.
pub fn range_with<T>(range: impl RangeBounds<T>, rng: &mut impl Rng) -> T
where
    T: Random<Output = T>
        + PartialOrd
        + std::ops::Sub<Output = T>
        + std::ops::Add<Output = T>
        + std::ops::Rem<Output = T>
        + std::fmt::Debug
        + std::fmt::Display
        + Numeric
        + Clone,
{
    let min = match range.start_bound().cloned() {
        std::ops::Bound::Included(x) => x,
        std::ops::Bound::Excluded(x) => x + T::one(),
        std::ops::Bound::Unbounded => T::min_value(),
    };

    let max = match range.end_bound().cloned() {
        std::ops::Bound::Included(x) => x + T::one(),
        std::ops::Bound::Excluded(x) => x,
        std::ops::Bound::Unbounded => T::max_value(),
    };

    assert!(min < max, "max must be greater than min: {max} < {min}");

    dbg!(&min, &max);

    let n = T::random(rng);
    min.clone() + (n % (max - min))
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
