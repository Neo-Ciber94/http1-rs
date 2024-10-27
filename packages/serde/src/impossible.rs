use std::{convert::Infallible, marker::PhantomData};

use super::ser::{BytesSerializer, MapSerializer, SequenceSerializer, Serialize, Serializer};

/// A helper struct to represent any type that cannot be serialized
pub struct Impossible<R, E> {
    void: Infallible,
    _marker: PhantomData<(R, E)>,
}

impl<R, E> Serializer for Impossible<R, E>
where
    E: std::error::Error,
{
    type Ok = R;
    type Err = E;
    type Bytes = Impossible<R, E>;
    type Seq = Impossible<R, E>;
    type Map = Impossible<R, E>;

    fn serialize_unit(self) -> Result<Self::Ok, Self::Err> {
        match self.void {}
    }

    fn serialize_i128(self, _value: i128) -> Result<Self::Ok, Self::Err> {
        match self.void {}
    }

    fn serialize_u128(self, _value: u128) -> Result<Self::Ok, Self::Err> {
        match self.void {}
    }

    fn serialize_f32(self, _value: f32) -> Result<Self::Ok, Self::Err> {
        match self.void {}
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok, Self::Err> {
        match self.void {}
    }

    fn serialize_bool(self, _value: bool) -> Result<Self::Ok, Self::Err> {
        match self.void {}
    }

    fn serialize_str(self, _value: &str) -> Result<Self::Ok, Self::Err> {
        match self.void {}
    }

    fn serialize_sequence(self) -> Result<Self::Seq, Self::Err> {
        match self.void {}
    }

    fn serialize_map(self) -> Result<Self::Map, Self::Err> {
        match self.void {}
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Err> {
        match self.void {}
    }

    fn serialize_slice<T: Serialize>(self, _value: &[T]) -> Result<Self::Ok, Self::Err> {
        match self.void {}
    }

    fn serialize_byte_seq(self) -> Result<Self::Bytes, Self::Err> {
        match self.void {}
    }
}

impl<R, E> MapSerializer for Impossible<R, E>
where
    E: std::error::Error,
{
    type Ok = R;
    type Err = E;

    fn serialize_entry<K: Serialize, V: Serialize>(
        &mut self,
        _key: &K,
        _value: &V,
    ) -> Result<(), Self::Err> {
        match self.void {}
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        match self.void {}
    }
}

impl<R, E> SequenceSerializer for Impossible<R, E>
where
    E: std::error::Error,
{
    type Ok = R;
    type Err = E;

    fn serialize_element<T: Serialize>(&mut self, _value: &T) -> Result<(), Self::Err> {
        match self.void {}
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        match self.void {}
    }
}

impl<R, E> BytesSerializer for Impossible<R, E>
where
    E: std::error::Error,
{
    type Ok = R;
    type Err = E;

    fn serialize_bytes<T: Serialize>(&mut self, _buf: &[u8]) -> Result<(), Self::Err> {
        match self.void {}
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        match self.void {}
    }
}
