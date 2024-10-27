use serde::{de::{Deserialize, Deserializer}, ser::{Serialize, Serializer}};

use crate::DateTime;

impl Serialize for DateTime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        serializer.serialize_u128(self.as_millis())
    }
}

impl Deserialize for DateTime {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, serde::de::Error> {
        let ms = u128::deserialize(deserializer)?;
        Ok(DateTime::with_millis(ms))
    }
}
