use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::common::any_map::AnyMap;

/// Request extensions.
#[derive(Default)]
pub struct Extensions(AnyMap);

impl Extensions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn extend(&mut self, other: Extensions) {
        self.0.extend(other.0);
    }
}

impl Deref for Extensions {
    type Target = AnyMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Extensions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Debug for Extensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Extensions").finish_non_exhaustive()
    }
}
