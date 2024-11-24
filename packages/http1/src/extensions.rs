use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::common::any_map::{AnyMap, CloneableAnyMap};

/// Request extensions.
#[derive(Default)]
pub struct Extensions(AnyMap);

impl Extensions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn extend<U: Into<AnyMap>>(&mut self, other: U) {
        self.0.extend(other.into());
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

impl From<AnyMap> for Extensions {
    fn from(value: AnyMap) -> Self {
        Extensions(value)
    }
}

// impl From<CloneableAnyMap> for Extensions {
//     fn from(value: CloneableAnyMap) -> Self {
//         Extensions(value.into())
//     }
// }
