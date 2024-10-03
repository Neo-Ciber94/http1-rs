use std::marker::PhantomData;

use super::serialize::{MapSerializer, SequenceSerializer, Serialize, Serializer};

enum Void {}

pub struct Impossible<E> {
    void: Void,
    _error: PhantomData<E>,
}

impl<E> Serializer for Impossible<E>
where
    E: std::error::Error,
{
    type Err = E;
    type Seq = Impossible<E>;
    type Map = Impossible<E>;

    fn serialize_i128(self, _value: i128) -> Result<(), Self::Err> {
         match self.void {}
    }

    fn serialize_u128(self, _value: u128) -> Result<(), Self::Err> {
         match self.void {}
    }

    fn serialize_f64(self, _value: f64) -> Result<(), Self::Err> {
         match self.void {}
    }

    fn serialize_bool(self, _value: bool) -> Result<(), Self::Err> {
         match self.void {}
    }

    fn serialize_str(self, _value: &str) -> Result<(), Self::Err> {
         match self.void {}
    }

    fn serialize_option<T: Serialize>(self, _value: Option<T>) -> Result<(), Self::Err> {
         match self.void {}
    }

    fn serialize_sequence(self) -> Result<Self::Seq, Self::Err> {
         match self.void {}
    }

    fn serialize_map(self) -> Result<Self::Map, Self::Err> {
         match self.void {}
    }
}

impl<E> MapSerializer for Impossible<E>
where
    E: std::error::Error,
{
    type Err = E;

    fn serialize_entry<K: Serialize, V: Serialize>(
        &mut self,
        _key: &K,
        _value: &V,
    ) -> Result<(), Self::Err> {
        match self.void {}
    }

    fn end(self) -> Result<(), Self::Err> {
        match self.void {}
    }
}

impl<E> SequenceSerializer for Impossible<E>
where
    E: std::error::Error,
{
    type Err = E;

    fn serialize_element<T: Serialize>(&mut self, _value: &T) -> Result<(), Self::Err> {
        match self.void {}
    }

    fn end(self) -> Result<(), Self::Err> {
        match self.void {}
    }
}
