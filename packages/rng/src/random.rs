use std::marker::PhantomData;

use super::Rng;

/// An iterator for a random sequence of values.
pub struct RandomSequence<R, T> {
    rng: R,
    marker: PhantomData<T>,
}

impl<R: Rng, T: Random> Iterator for RandomSequence<R, T> {
    type Item = T::Output;

    fn next(&mut self) -> Option<Self::Item> {
        Some(T::random(&mut self.rng))
    }
}

/// A random value generator.
pub trait Random: Sized {
    type Output;

    fn random<R: Rng>(rng: &mut R) -> Self::Output;
    fn sequence<R: Rng>(rng: R) -> RandomSequence<R, Self> {
        RandomSequence {
            rng,
            marker: PhantomData,
        }
    }
}

impl Random for bool {
    type Output = bool;
    fn random<R: Rng>(rng: &mut R) -> Self {
        u32::random(rng) & 1 == 1
    }
}

impl Random for f32 {
    type Output = f32;
    fn random<R: Rng>(rng: &mut R) -> Self {
        // Generate a random u32 and convert it to f32 in the range [0.0, 1.0)
        let n = u32::random(rng);
        n as f32 / u32::MAX as f32
    }
}

impl Random for f64 {
    type Output = f64;
    fn random<R: Rng>(rng: &mut R) -> Self {
        // Generate a random u64 and convert it to f64 in the range [0.0, 1.0)
        let n = u64::random(rng);
        n as f64 / u64::MAX as f64
    }
}

macro_rules! impl_primitive_random {
    ($($T:ident),*) => {
       $(
            impl Random for $T {
                type Output = $T;
                fn random<R: Rng>(rng: &mut R) -> Self {
                    let mut bytes = (0 as Self).to_ne_bytes();
                    rng.fill_bytes(&mut bytes);
                    $T::from_ne_bytes(bytes)
                }
            }
       )*
    };
}

impl_primitive_random!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

pub struct Alphabetic;
impl Random for Alphabetic {
    type Output = char;
    fn random<R: Rng>(rng: &mut R) -> char {
        static ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

        let len = ALPHABET.len();
        let idx = crate::random_range(rng, 0, len);
        let n = ALPHABET.as_bytes()[idx] as u32;
        char::from_u32(n).expect("failed to get char")
    }
}

pub struct Alphanumeric;
impl Random for Alphanumeric {
    type Output = char;
    fn random<R: Rng>(rng: &mut R) -> char {
        static ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

        let len = ALPHABET.len();
        let idx = crate::random_range(rng, 0, len);
        let n = ALPHABET.as_bytes()[idx] as u32;
        char::from_u32(n).expect("failed to get char")
    }
}

pub struct Ascii;

impl Random for Ascii {
    type Output = char;

    fn random<R: Rng>(rng: &mut R) -> char {
        let min = 32;
        let max = 126;

        let n = crate::random_range(rng, min, max + 1) as u32;
        char::from_u32(n).expect("failed to get char")
    }
}
