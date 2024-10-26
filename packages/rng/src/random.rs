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
        rng.next_32() > u32::MAX / 2
    }
}

impl Random for f32 {
    type Output = f32;
    fn random<R: Rng>(rng: &mut R) -> Self {
        // Generate a random u32 and convert it to f32 in the range [0.0, 1.0)
        let n = rng.next_32();
        n as f32 / u32::MAX as f32
    }
}

impl Random for f64 {
    type Output = f64;
    fn random<R: Rng>(rng: &mut R) -> Self {
        // Generate a random u64 and convert it to f64 in the range [0.0, 1.0)
        let n = rng.next_64();
        n as f64 / u64::MAX as f64
    }
}

impl Random for u8 {
    type Output = u8;
    fn random<R: Rng>(rng: &mut R) -> Self {
        (rng.next_32() % u8::MAX as u32) as u8
    }
}

impl Random for u16 {
    type Output = u16;
    fn random<R: Rng>(rng: &mut R) -> Self {
        (rng.next_32() % u16::MAX as u32) as u16
    }
}

impl Random for u32 {
    type Output = u32;
    fn random<R: Rng>(rng: &mut R) -> Self {
        rng.next_32() as u32
    }
}

impl Random for u64 {
    type Output = u64;
    fn random<R: Rng>(rng: &mut R) -> Self {
        rng.next_64()
    }
}

impl Random for u128 {
    type Output = u128;
    fn random<R: Rng>(rng: &mut R) -> Self {
        rng.next_128()
    }
}

impl Random for i8 {
    type Output = i8;
    fn random<R: Rng>(rng: &mut R) -> Self {
        (rng.next_32() % u8::MAX as u32) as i8
    }
}

impl Random for i16 {
    type Output = i16;
    fn random<R: Rng>(rng: &mut R) -> Self {
        (rng.next_32() % u16::MAX as u32) as i16
    }
}

impl Random for i32 {
    type Output = i32;
    fn random<R: Rng>(rng: &mut R) -> Self {
        rng.next_32() as i32
    }
}

impl Random for i64 {
    type Output = i64;
    fn random<R: Rng>(rng: &mut R) -> Self {
        rng.next_64() as i64
    }
}

impl Random for i128 {
    type Output = i128;
    fn random<R: Rng>(rng: &mut R) -> Self {
        rng.next_128() as i128
    }
}

impl Random for usize {
    type Output = usize;
    fn random<R: Rng>(rng: &mut R) -> Self {
        rng.next_128() as usize
    }
}

impl Random for isize {
    type Output = isize;
    fn random<R: Rng>(rng: &mut R) -> Self {
        rng.next_128() as isize
    }
}

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
