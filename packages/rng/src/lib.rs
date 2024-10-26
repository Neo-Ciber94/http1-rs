use std::cell::RefCell;

use random::{Random, RandomSequence};

pub mod random;
mod xorshift;

pub trait Rng {
    fn next_32(&mut self) -> u32;

    fn next_64(&mut self) -> u64 {
        let high = self.next_32() as u64;
        let low = self.next_32() as u64;
        (high << 32) | low
    }

    fn next_128(&mut self) -> u128 {
        let high = self.next_64();
        let low = self.next_64();
        (high as u128) << 64 | low as u128
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
