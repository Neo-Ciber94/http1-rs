use crate::Rng;

pub struct PickMultipleIter<'a, T, R> {
    src: &'a T,
    rng: &'a mut R,
}

impl<'a, T, R> Iterator for PickMultipleIter<'a, T, R>
where
    T: Pick<Output = T>,
    R: Rng,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.src.pick(self.rng)
    }
}

/// Picks a random value.
pub trait Pick: Sized {
    type Output;

    /// Returns a random value from this.
    fn pick(&self, rng: &mut impl Rng) -> Option<&Self::Output>;

    /// Returns an iterator over that picks multiple values from this.
    fn pick_multiple<'a, R>(&'a self, rng: &'a mut R) -> PickMultipleIter<'a, Self, R> {
        PickMultipleIter { src: self, rng }
    }
}

impl<'a, T> Pick for &'a [T] {
    type Output = T;

    fn pick(&self, rng: &mut impl Rng) -> Option<&Self::Output> {
        let idx = crate::range_with(0..self.len(), rng);
        self.get(idx)
    }
}

impl<const N: usize, T> Pick for [T; N]
where
    T: Clone,
{
    type Output = T;

    fn pick(&self, rng: &mut impl Rng) -> Option<&Self::Output> {
        let idx = crate::range_with(0..self.len(), rng);
        self.get(idx)
    }
}

impl<'a, const N: usize, T> Pick for &'a [T; N] {
    type Output = T;

    fn pick(&self, rng: &mut impl Rng) -> Option<&Self::Output> {
        let idx = crate::range_with(0..self.len(), rng);
        self.get(idx)
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

            fn pick(&self, rng: &mut impl Rng) -> Option<&Self::Output> {
                let len = count!($($index),+);
                let idx = crate::range_with(0..len, rng);

                match idx {
                    $(
                        n if n == $index => Some(&self.$index),
                    )*
                    _ => None,
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
