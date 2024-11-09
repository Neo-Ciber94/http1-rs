use std::ops::{Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};

use crate::{random::Random, Rng};

trait Numeric: Sized {
    fn min_value() -> Self;
    fn max_value() -> Self;
}

macro_rules! impl_min_max {
    ($($T:ident),*) => {
        $(
            impl Numeric for $T {
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

impl_min_max!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);

/// Picks a random value.
pub trait Pick {
    type Output;

    /// Gets a random value from this.
    fn pick(&self, rng: &mut impl Rng) -> Self::Output;
}

impl<T> Pick for Range<T>
where
    T: Random<Output = T>
        + PartialOrd
        + std::ops::Sub<Output = T>
        + std::ops::Add<Output = T>
        + std::ops::Rem<Output = T>
        + std::fmt::Debug
        + std::fmt::Display
        + Clone,
{
    type Output = T;

    fn pick(&self, rng: &mut impl Rng) -> Self::Output {
        let min = self.start.clone();
        let max = self.end.clone();
        assert!(min < max, "max must be greater than min: {max} < {min}");
        let n = T::random(rng);
        min.clone() + (n % (max - min))
    }
}

impl<T> Pick for RangeInclusive<T>
where
    T: Random<Output = T>
        + PartialOrd
        + std::ops::Sub<Output = T>
        + std::ops::Add<Output = T>
        + std::ops::Rem<Output = T>
        + std::fmt::Debug
        + std::fmt::Display
        + Clone,
{
    type Output = T;

    fn pick(&self, rng: &mut impl Rng) -> Self::Output {
        let min = self.start().clone();
        let max = self.end().clone();
        assert!(
            min <= max,
            "max must be greater than or equal to min: {max} <= {min}"
        );
        let n = T::random(rng);
        min.clone() + (n % (max - min + T::random(rng)))
    }
}

impl<T> Pick for RangeFrom<T>
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
    type Output = T;

    fn pick(&self, rng: &mut impl Rng) -> Self::Output {
        let min = self.start.clone();
        let max = T::max_value();
        (min..=max).pick(rng)
    }
}

impl<T> Pick for RangeToInclusive<T>
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
    type Output = T;

    fn pick(&self, rng: &mut impl Rng) -> Self::Output {
        let min = T::min_value();
        let max = self.end.clone();
        (min..=max).pick(rng)
    }
}

impl<T> Pick for RangeTo<T>
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
    type Output = T;

    fn pick(&self, rng: &mut impl Rng) -> Self::Output {
        let min = T::min_value();
        let max = self.end.clone();
        (min..max).pick(rng)
    }
}

impl<const N: usize, T> Pick for [T; N]
where
    T: Clone,
{
    type Output = T;

    fn pick(&self, rng: &mut impl Rng) -> Self::Output {
        let idx = (0..self.len()).pick(rng);
        self[idx].clone()
    }
}

impl<'a, const N: usize, T> Pick for &'a [T; N] {
    type Output = &'a T;

    fn pick(&self, rng: &mut impl Rng) -> Self::Output {
        self.as_slice().pick(rng)
    }
}

impl<'a, T> Pick for &'a [T] {
    type Output = &'a T;

    fn pick(&self, rng: &mut impl Rng) -> Self::Output {
        let idx = (0..self.len()).pick(rng);
        &self[idx]
    }
}

impl<'a> Pick for &'a str {
    type Output = char;

    fn pick(&self, rng: &mut impl Rng) -> Self::Output {
        let idx = (0..self.len()).pick(rng);
        self.char_indices()
            .find(|(i, _)| *i == idx)
            .map(|(_, c)| c)
            .unwrap()
    }
}

impl Pick for String {
    type Output = char;

    fn pick(&self, rng: &mut impl Rng) -> Self::Output {
        self.as_str().pick(rng)
    }
}

macro_rules! count {
    () => (0);
    ($t:tt $(,$rest:tt)*) => (1 + count!($($rest),*));
}

macro_rules! impl_tuple_pick {
    ($($T:ident),+ => $($index:tt),+) => {
        impl<T> Pick for ( $($T,)+ )
        where
            T: Clone,
        {
            type Output = T;

            fn pick(&self, rng: &mut impl Rng) -> Self::Output {
                let len = count!($($index),+);
                let idx = (0..len).pick(rng);

                match idx {
                    $(
                        n if n == $index => self.$index.clone(),
                    )*
                    _ => unreachable!(),
                }
            }
        }
    };
}

impl_tuple_pick!(T => 0);
impl_tuple_pick!(T, T => 0, 1);
impl_tuple_pick!(T, T, T => 0, 1, 2);
impl_tuple_pick!(T, T, T, T => 0, 1, 2, 3);
impl_tuple_pick!(T, T, T, T, T => 0, 1, 2, 3, 4);
impl_tuple_pick!(T, T, T, T, T, T => 0, 1, 2, 3, 4, 5);
impl_tuple_pick!(T, T, T, T, T, T, T => 0, 1, 2, 3, 4, 5, 6);
impl_tuple_pick!(T, T, T, T, T, T, T, T => 0, 1, 2, 3, 4, 5, 6, 7);
impl_tuple_pick!(T, T, T, T, T, T, T, T, T => 0, 1, 2, 3, 4, 5, 6, 7, 8);
impl_tuple_pick!(T, T, T, T, T, T, T, T, T, T => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
impl_tuple_pick!(T, T, T, T, T, T, T, T, T, T, T => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
impl_tuple_pick!(T, T, T, T, T, T, T, T, T, T, T, T => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
