use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    rc::Rc,
    sync::Arc,
};

use http1::common::map::OrderedMap;

/// Serialize a sequence of items like a `Vec<T>` or array,
pub trait SequenceSerializer {
    type Ok;
    type Err: std::error::Error;

    /// Serializes the next element.
    fn serialize_element<T: Serialize>(&mut self, value: &T) -> Result<(), Self::Err>;

    /// Finishes the serialization.
    fn end(self) -> Result<Self::Ok, Self::Err>;
}

/// Serializes a stream of bytes.
pub trait BytesSerializer {
    type Ok;
    type Err: std::error::Error;

    /// Serialize the next chunk of bytes.
    fn serialize_bytes<T: Serialize>(&mut self, buf: &[u8]) -> Result<(), Self::Err>;

    /// Finishes the serialization.
    fn end(self) -> Result<Self::Ok, Self::Err>;
}

/// Serializes a key-value pair collection.
pub trait MapSerializer {
    type Ok;
    type Err: std::error::Error;

    /// Serializes the key-value pair.
    fn serialize_entry<K: Serialize, V: Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Err>;

    /// Finishes the serialization.
    fn end(self) -> Result<Self::Ok, Self::Err>;
}

/// Provides a mechanism for convert a value to other representation.
pub trait Serializer: Sized {
    type Ok;
    type Err: std::error::Error;
    type Bytes: BytesSerializer;
    type Seq: SequenceSerializer<Ok = Self::Ok, Err = Self::Err>;
    type Map: MapSerializer<Ok = Self::Ok, Err = Self::Err>;

    fn serialize_unit(self) -> Result<Self::Ok, Self::Err>;

    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Err> {
        self.serialize_i128(value.into())
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Err> {
        self.serialize_i128(value.into())
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Err> {
        self.serialize_i128(value.into())
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Err> {
        self.serialize_i128(value.into())
    }

    fn serialize_i128(self, value: i128) -> Result<Self::Ok, Self::Err>;

    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Err> {
        self.serialize_u128(value.into())
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Err> {
        self.serialize_u128(value.into())
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Err> {
        self.serialize_u128(value.into())
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Err> {
        self.serialize_u128(value.into())
    }

    fn serialize_u128(self, value: u128) -> Result<Self::Ok, Self::Err>;

    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Err>;

    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Err>;

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Err>;

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Err>;

    fn serialize_string(self, value: String) -> Result<Self::Ok, Self::Err> {
        self.serialize_str(&value)
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Err> {
        let mut buf = [0; 4];
        self.serialize_str(value.encode_utf8(&mut buf))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Err>;

    fn serialize_option<T: Serialize>(self, value: &Option<T>) -> Result<Self::Ok, Self::Err> {
        match value {
            Some(x) => x.serialize(self),
            None => self.serialize_none(),
        }
    }

    fn serialize_slice<T: Serialize>(self, value: &[T]) -> Result<Self::Ok, Self::Err> {
        let mut seq_serializer = self.serialize_sequence()?;

        for x in value {
            seq_serializer.serialize_element(x)?;
        }

        seq_serializer.end()
    }

    fn serialize_array<T: Serialize, const N: usize>(
        self,
        value: [T; N],
    ) -> Result<Self::Ok, Self::Err> {
        self.serialize_slice(&value)
    }

    fn serialize_vec<T: Serialize>(self, value: &Vec<T>) -> Result<Self::Ok, Self::Err> {
        self.serialize_slice(value.as_slice())
    }

    fn serialize_bytes<T: Serialize>(self, value: &Vec<u8>) -> Result<Self::Ok, Self::Err> {
        self.serialize_slice(value.as_slice())
    }

    fn serialize_byte_seq(self) -> Result<Self::Bytes, Self::Err>;

    fn serialize_sequence(self) -> Result<Self::Seq, Self::Err>;

    fn serialize_map(self) -> Result<Self::Map, Self::Err>;
}

/// Allow a type to be serialize to other.
pub trait Serialize {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err>;
}

// Implementations

macro_rules! impl_serialize_primitive {
    ($($name:ident => $method:ident $(as $cast:ty)?),*) => {
        $(
            impl Serialize for $name {
                fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
                    serializer.$method(*self $(as $cast)*)
                }
            }
        )*
    };
}

impl_serialize_primitive!(
    u8 => serialize_u8,
    u16 => serialize_u16,
    u32 => serialize_u32,
    u64 => serialize_u64,
    u128 => serialize_u128,
    usize => serialize_u64 as u64,
    i8 => serialize_i8,
    i16 => serialize_i16,
    i32 => serialize_i32,
    i64 => serialize_i64,
    i128 => serialize_i128,
    isize => serialize_i64 as i64,
    f32 => serialize_f32,
    f64 => serialize_f64,
    char => serialize_char,
    bool => serialize_bool
);

// TODO: We only serialized OWNED types, str or Cow<'a, str> makes no sense here

impl Serialize for () {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        serializer.serialize_none()
    }
}

impl<'a> Serialize for &'a str {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        serializer.serialize_str(self)
    }
}

impl Serialize for String {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        serializer.serialize_str(self)
    }
}

impl<'a> Serialize for Cow<'a, str> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        serializer.serialize_str(self)
    }
}

impl<'a, T: Serialize> Serialize for &'a T {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        (*self).serialize(serializer)
    }
}

impl<T: Serialize> Serialize for Option<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        serializer.serialize_option(self)
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        serializer.serialize_vec(self)
    }
}

impl<T: Serialize> Serialize for HashSet<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        let mut seq = serializer.serialize_sequence()?;

        for item in self.iter() {
            seq.serialize_element(item)?;
        }

        seq.end()
    }
}

impl<T: Serialize> Serialize for VecDeque<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        let mut seq = serializer.serialize_sequence()?;

        for item in self.iter() {
            seq.serialize_element(item)?;
        }

        seq.end()
    }
}

impl<T: Serialize> Serialize for LinkedList<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        let mut seq = serializer.serialize_sequence()?;

        for item in self.iter() {
            seq.serialize_element(item)?;
        }

        seq.end()
    }
}

impl<T: Serialize> Serialize for BinaryHeap<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        let mut seq = serializer.serialize_sequence()?;

        for item in self.iter() {
            seq.serialize_element(item)?;
        }

        seq.end()
    }
}

impl<T: Serialize> Serialize for BTreeSet<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        let mut seq = serializer.serialize_sequence()?;

        for item in self.iter() {
            seq.serialize_element(item)?;
        }

        seq.end()
    }
}

impl<K, V> Serialize for HashMap<K, V>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        let mut map = serializer.serialize_map()?;

        for (key, value) in self.iter() {
            map.serialize_entry(key, value)?;
        }

        map.end()
    }
}

impl<K, V> Serialize for BTreeMap<K, V>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        let mut map = serializer.serialize_map()?;

        for (key, value) in self.iter() {
            map.serialize_entry(key, value)?;
        }

        map.end()
    }
}

impl<K, V> Serialize for OrderedMap<K, V>
where
    K: Serialize + std::cmp::Eq + std::hash::Hash,
    V: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        let mut map = serializer.serialize_map()?;

        for (key, value) in self.iter() {
            map.serialize_entry(key, value)?;
        }

        map.end()
    }
}

impl<T: Serialize, const N: usize> Serialize for [T; N] {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        serializer.serialize_slice(self)
    }
}

impl<'a, T: Serialize> Serialize for &'a [T] {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        serializer.serialize_slice(self)
    }
}

impl<T: Serialize> Serialize for Box<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        (**self).serialize(serializer)
    }
}

impl<T: Serialize> Serialize for Box<[T]> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        serializer.serialize_slice(&*self)
    }
}

impl<T: Serialize> Serialize for Rc<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        (**self).serialize(serializer)
    }
}

impl<T: Serialize> Serialize for Arc<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        (**self).serialize(serializer)
    }
}

macro_rules! impl_serialize_tuple {
    ($($T:ident),*) => {
        #[allow(non_snake_case)]
        impl<$($T),*> Serialize for ($($T),*,)
            where $($T : Serialize),* {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
                let mut seq = serializer.serialize_sequence()?;

                let ($($T),*,) = &self;
                $(
                    seq.serialize_element($T)?;
                )*

                seq.end()
            }
        }
    };
}

impl_serialize_tuple!(T1);
impl_serialize_tuple!(T1, T2);
impl_serialize_tuple!(T1, T2, T3);
impl_serialize_tuple!(T1, T2, T3, T4);
impl_serialize_tuple!(T1, T2, T3, T4, T5);
impl_serialize_tuple!(T1, T2, T3, T4, T5, T6);
impl_serialize_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_serialize_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_serialize_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_serialize_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_serialize_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_serialize_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
