use crate::{Pick, Rng};

/// Allow to reorder the elements.
pub trait Shuffle: Sized {
    /// Reorder all the elements in place.
    fn shuffle_in_place(&mut self, rng: &mut impl Rng);

    /// Reorder all the elements and return it.
    fn shuffle(mut self, rng: &mut impl Rng) -> Self {
        self.shuffle_in_place(rng);
        self
    }
}

impl<'a, T> Shuffle for &'a mut [T] {
    fn shuffle_in_place(&mut self, rng: &mut impl Rng) {
        for i in 0..self.len() {
            let idx = (i..self.len()).pick(rng);
            self.swap(i, idx);
        }
    }
}

impl<const N: usize, T> Shuffle for [T; N] {
    fn shuffle_in_place(&mut self, rng: &mut impl Rng) {
        self.as_mut_slice().shuffle_in_place(rng);
    }
}

impl<'a, const N: usize, T> Shuffle for &'a mut [T; N] {
    fn shuffle_in_place(&mut self, rng: &mut impl Rng) {
        self.as_mut_slice().shuffle_in_place(rng);
    }
}

impl Shuffle for String {
    fn shuffle_in_place(&mut self, rng: &mut impl Rng) {
        fn swap(s: &mut String, from: usize, to: usize) {
            let from_idx = from.checked_sub(1).unwrap_or_default();
            let to_idx = to.checked_sub(1).unwrap_or_default();

            let a = s.remove(from);
            let b = s.remove(to_idx);

            s.insert(to_idx, a);
            s.insert(from_idx, b);
        }

        let count = self.chars().count();

        for i in 0..count {
            let idx = (i..count).pick(rng);
            swap(self, i, idx);
        }
    }
}
