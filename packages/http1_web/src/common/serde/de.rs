use std::{
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    fmt::{Debug, Display},
    marker::PhantomData,
};

use super::{
    expected::{Expected, TypeMismatchError},
    visitor::Visitor,
};

#[derive(Debug)]
pub enum Error {
    Custom(String),
    Unexpected(Unexpected),
    Mismatch(TypeMismatchError),
    Other(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl Error {
    pub fn custom(msg: impl Into<String>) -> Self {
        Error::Custom(msg.into())
    }

    pub fn error<E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>>(error: E) -> Self {
        Error::Other(error.into())
    }

    pub fn mismatch<T>(this: T, expected: impl Into<String>) -> Self
    where
        T: Expected + 'static,
    {
        Error::Mismatch(TypeMismatchError::new(this, expected))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Custom(msg) => write!(f, "{msg}"),
            Error::Unexpected(unexpected) => write!(f, "unexpected {unexpected}"),
            Error::Other(err) => write!(f, "{err}"),
            Error::Mismatch(mismatch) => write!(f, "{mismatch}"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Unexpected {
    Bool(bool),
    Char(char),
    Unsigned(u128),
    Signed(i128),
    Float(f64),
    Str(String),
    Unit,
    Option,
    Seq,
    Map,
}

impl Display for Unexpected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unexpected::Bool(value) => write!(f, "boolean `{value}`"),
            Unexpected::Char(value) => write!(f, "char `{value}`"),
            Unexpected::Unsigned(value) => write!(f, "unsigned integer `{value}`"),
            Unexpected::Signed(value) => write!(f, "signed integer `{value}`"),
            Unexpected::Float(value) => write!(f, "float `{value}`"),
            Unexpected::Str(value) => write!(f, "string `{value}`"),
            Unexpected::Unit => write!(f, "unit type"),
            Unexpected::Option => write!(f, "option type"),
            Unexpected::Seq => write!(f, "sequence"),
            Unexpected::Map => write!(f, "map"),
        }
    }
}

pub trait Deserializer {
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor;
}

pub trait Deserialize: Sized {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, Error>;
}

struct UnitVisitor;
impl Visitor for UnitVisitor {
    type Value = ();

    fn visit_none(self) -> Result<Self::Value, Error> {
        Ok(())
    }

    fn visit_option<T: Deserializer>(self, _value: Option<T>) -> Result<Self::Value, Error> {
        self.visit_none()
    }
}

impl Deserialize for () {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, Error> {
        deserializer.deserialize_option(UnitVisitor)
    }
}

struct BoolVisitor;
impl Visitor for BoolVisitor {
    type Value = bool;
    fn visit_bool(self, value: bool) -> Result<Self::Value, Error> {
        Ok(value)
    }
}
impl Deserialize for bool {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, Error> {
        deserializer.deserialize_bool(BoolVisitor)
    }
}

struct CharVisitor;
impl Visitor for CharVisitor {
    type Value = char;
    fn visit_char(self, value: char) -> Result<Self::Value, Error> {
        Ok(value)
    }
}

impl Deserialize for char {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, Error> {
        deserializer.deserialize_char(CharVisitor)
    }
}

struct StringVisitor;
impl Visitor for StringVisitor {
    type Value = String;
    fn visit_string(self, value: String) -> Result<Self::Value, Error> {
        Ok(value)
    }
}

impl Deserialize for String {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, Error> {
        deserializer.deserialize_string(StringVisitor)
    }
}

macro_rules! impl_deserialize_seq {
    ($visitor:ident => $T:ty => $insert_method:ident $(where $($tt:tt)*)?) => {
        struct $visitor<V>(PhantomData<V>);
        impl<V: Deserialize> Visitor for $visitor<V> $(where $($tt)*)* {
            type Value = $T;

            fn visit_seq<Seq: super::visitor::SeqAccess>(self, mut seq: Seq) -> Result<Self::Value, Error> {
                let mut collection : $T = Default::default();

                loop {
                    match seq.next_element()? {
                        None => break,
                        Some(x) => {
                            collection.$insert_method(x);
                        },
                    }
                }

                Ok(collection)
            }
        }

        impl<V: Deserialize> Deserialize for $T $(where $($tt)*)* {
            fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, Error> {
                deserializer.deserialize_seq($visitor(PhantomData))
            }
        }

    };
}

impl_deserialize_seq!(VecVisitor => Vec<V> => push);
impl_deserialize_seq!(HashSetVisitor => HashSet<V> => insert where V: Eq + std::hash::Hash);
impl_deserialize_seq!(VecDequeVisitor => VecDeque<V> => push_back);
impl_deserialize_seq!(LinkedListVisitor => LinkedList<V> => push_back);
impl_deserialize_seq!(BTreeSetVisitor => BTreeSet<V> => insert where V: Ord);
impl_deserialize_seq!(BinaryHeapVisitor => BinaryHeap<V> => push where V: Ord);

macro_rules! impl_deserialize_map {
    ($visitor:ident => $T:ty => $insert_method:ident $(where $($tt:tt)*)?) => {
        struct $visitor<K, V>(PhantomData<(K,V)>);
        impl<K: Deserialize, V: Deserialize> Visitor for $visitor<K, V> $(where $($tt)*)* {
            type Value = $T;

            fn visit_map<Map: super::visitor::MapAccess>(self, mut map: Map) -> Result<Self::Value, Error> {
                let mut collection : $T = Default::default();

                loop {
                    match map.next_entry()? {
                        None => break,
                        Some((k, v)) => {
                            collection.$insert_method(k, v);
                        },
                    }
                }

                Ok(collection)
            }
        }

        impl<K: Deserialize, V: Deserialize> Deserialize for $T $(where $($tt)*)* {
            fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, Error> {
                deserializer.deserialize_map($visitor(PhantomData))
            }
        }

    };
}

impl_deserialize_map!(BTreeMapVisitor => BTreeMap<K, V> => insert where K: Ord);
impl_deserialize_map!(HashMapVisitor => HashMap<K, V> => insert where K: Eq + std::hash::Hash);

struct F32Visitor;
impl Visitor for F32Visitor {
    type Value = f32;

    fn visit_f32(self, value: f32) -> Result<Self::Value, Error> {
        Ok(value)
    }

    fn visit_f64(self, value: f64) -> Result<Self::Value, Error> {
        if value > f32::MAX as f64 {
            return Err(Error::custom("f32 overflow"));
        }

        if value < f32::MIN as f64 {
            return Err(Error::custom("f32 underflow"));
        }

        self.visit_f32(value as f32)
    }
}

struct F64Visitor;
impl Visitor for F64Visitor {
    type Value = f64;

    fn visit_f32(self, value: f32) -> Result<Self::Value, Error> {
        self.visit_f64(value as f64)
    }

    fn visit_f64(self, value: f64) -> Result<Self::Value, Error> {
        Ok(value)
    }
}

macro_rules! impl_deserialize_to_uint {
    ($T:ty : $base_method:ident => $U:ty : $uint_visitor:ident) => {
        fn $uint_visitor(self, value: $U) -> Result<Self::Value, Error> {
            let v = value.try_into().map_err(Error::error)?;
            self.$base_method(v)
        }
    };
}

macro_rules! impl_deserialize_to_int {
    ($T:ty : $base_method:ident => $U:ty : $int_visitor:ident) => {
        fn $int_visitor(self, value: $U) -> Result<Self::Value, Error> {
            let v = value.try_into().map_err(Error::error)?;
            self.$base_method(v)
        }
    };
}

macro_rules! impl_deserialize_number {
    ($visitor:ident: $T:ty => $deserialize_method:ident => $visitor_method:ident [$($tt:tt)*]) => {
        struct $visitor;

        impl Visitor for $visitor {
            type Value = $T;

            fn $visitor_method(self, value: $T) -> Result<Self::Value, Error> {
                Ok(value)
            }

            $($tt)*
        }

        impl Deserialize for $T {
            fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, Error> {
                deserializer.$deserialize_method($visitor)
            }
        }
    };
}

impl_deserialize_number!(U8Visitor: u8 => deserialize_u8 => visit_u8 [
    impl_deserialize_to_uint!(u8:visit_u8 => u16:visit_u16);
    impl_deserialize_to_uint!(u8:visit_u8 => u32:visit_u32);
    impl_deserialize_to_uint!(u8:visit_u8 => u64:visit_u64);
    impl_deserialize_to_uint!(u8:visit_u8 => u128:visit_u128);

    impl_deserialize_to_int!(u8:visit_u8 => i8:visit_i8);
    impl_deserialize_to_int!(u8:visit_u8 => i16:visit_i16);
    impl_deserialize_to_int!(u8:visit_u8 => i32:visit_i32);
    impl_deserialize_to_int!(u8:visit_u8 => i64:visit_i64);
    impl_deserialize_to_int!(u8:visit_u8 => i128:visit_i128);
]);

impl_deserialize_number!(U16Visitor: u16 => deserialize_u16 => visit_u16 [
    impl_deserialize_to_uint!(u16:visit_u16 => u8:visit_u8);
    impl_deserialize_to_uint!(u16:visit_u16 => u32:visit_u32);
    impl_deserialize_to_uint!(u16:visit_u16 => u64:visit_u64);
    impl_deserialize_to_uint!(u16:visit_u16 => u128:visit_u128);

    impl_deserialize_to_int!(u16:visit_u16 => i8:visit_i8);
    impl_deserialize_to_int!(u16:visit_u16 => i16:visit_i16);
    impl_deserialize_to_int!(u16:visit_u16 => i32:visit_i32);
    impl_deserialize_to_int!(u16:visit_u16 => i64:visit_i64);
    impl_deserialize_to_int!(u16:visit_u16 => i128:visit_i128);
]);

impl_deserialize_number!(U32Visitor: u32 => deserialize_u32 => visit_u32 [
    impl_deserialize_to_uint!(u32:visit_u32 => u8:visit_u8);
    impl_deserialize_to_uint!(u32:visit_u32 => u16:visit_u16);
    impl_deserialize_to_uint!(u32:visit_u32 => u64:visit_u64);
    impl_deserialize_to_uint!(u32:visit_u32 => u128:visit_u128);

    impl_deserialize_to_int!(u32:visit_u32 => i8:visit_i8);
    impl_deserialize_to_int!(u32:visit_u32 => i16:visit_i16);
    impl_deserialize_to_int!(u32:visit_u32 => i32:visit_i32);
    impl_deserialize_to_int!(u32:visit_u32 => i64:visit_i64);
    impl_deserialize_to_int!(u32:visit_u32 => i128:visit_i128);
]);

impl_deserialize_number!(U64Visitor: u64 => deserialize_u64 => visit_u64 [
    impl_deserialize_to_uint!(u64:visit_u64 => u8:visit_u8);
    impl_deserialize_to_uint!(u64:visit_u64 => u16:visit_u16);
    impl_deserialize_to_uint!(u64:visit_u64 => u32:visit_u32);
    impl_deserialize_to_uint!(u64:visit_u64 => u128:visit_u128);

    impl_deserialize_to_int!(u64:visit_u64 => i8:visit_i8);
    impl_deserialize_to_int!(u64:visit_u64 => i16:visit_i16);
    impl_deserialize_to_int!(u64:visit_u64 => i32:visit_i32);
    impl_deserialize_to_int!(u64:visit_u64 => i64:visit_i64);
    impl_deserialize_to_int!(u64:visit_u64 => i128:visit_i128);
]);

impl_deserialize_number!(U128Visitor: u128 => deserialize_u128 => visit_u128 [
    impl_deserialize_to_uint!(u128:visit_u128 => u8:visit_u8);
    impl_deserialize_to_uint!(u128:visit_u128 => u16:visit_u16);
    impl_deserialize_to_uint!(u128:visit_u128 => u32:visit_u32);
    impl_deserialize_to_uint!(u128:visit_u128 => u64:visit_u64);

    impl_deserialize_to_int!(u128:visit_u128 => i8:visit_i8);
    impl_deserialize_to_int!(u128:visit_u128 => i16:visit_i16);
    impl_deserialize_to_int!(u128:visit_u128 => i32:visit_i32);
    impl_deserialize_to_int!(u128:visit_u128 => i64:visit_i64);
    impl_deserialize_to_int!(u128:visit_u128 => i128:visit_i128);
]);

impl_deserialize_number!(I8Visitor: i8 => deserialize_i8 => visit_i8 [
    impl_deserialize_to_uint!(i8:visit_i8 => u8:visit_u8);
    impl_deserialize_to_uint!(i8:visit_i8 => u16:visit_u16);
    impl_deserialize_to_uint!(i8:visit_i8 => u32:visit_u32);
    impl_deserialize_to_uint!(i8:visit_i8 => u64:visit_u64);
    impl_deserialize_to_uint!(i8:visit_i8 => u128:visit_u128);

    impl_deserialize_to_int!(i8:visit_i8 => i16:visit_i16);
    impl_deserialize_to_int!(i8:visit_i8 => i32:visit_i32);
    impl_deserialize_to_int!(i8:visit_i8 => i64:visit_i64);
    impl_deserialize_to_int!(i8:visit_i8 => i128:visit_i128);
]);

impl_deserialize_number!(I16Visitor: i16 => deserialize_i16 => visit_i16 [
    impl_deserialize_to_uint!(i16:visit_i16 => u8:visit_u8);
    impl_deserialize_to_uint!(i16:visit_i16 => u16:visit_u16);
    impl_deserialize_to_uint!(i16:visit_i16 => u32:visit_u32);
    impl_deserialize_to_uint!(i16:visit_i16 => u64:visit_u64);
    impl_deserialize_to_uint!(i16:visit_i16 => u128:visit_u128);

    impl_deserialize_to_int!(i16:visit_i16 => i8:visit_i8);
    impl_deserialize_to_int!(i16:visit_i16 => i32:visit_i32);
    impl_deserialize_to_int!(i16:visit_i16 => i64:visit_i64);
    impl_deserialize_to_int!(i16:visit_i16 => i128:visit_i128);
]);

impl_deserialize_number!(I32Visitor: i32 => deserialize_i32 => visit_i32 [
    impl_deserialize_to_uint!(i32:visit_i32 => u8:visit_u8);
    impl_deserialize_to_uint!(i32:visit_i32 => u16:visit_u16);
    impl_deserialize_to_uint!(i32:visit_i32 => u32:visit_u32);
    impl_deserialize_to_uint!(i32:visit_i32 => u64:visit_u64);
    impl_deserialize_to_uint!(i32:visit_i32 => u128:visit_u128);

    impl_deserialize_to_int!(i32:visit_i32 => i8:visit_i8);
    impl_deserialize_to_int!(i32:visit_i32 => i16:visit_i16);
    impl_deserialize_to_int!(i32:visit_i32 => i64:visit_i64);
    impl_deserialize_to_int!(i32:visit_i32 => i128:visit_i128);
]);

impl_deserialize_number!(I64Visitor: i64 => deserialize_i64 => visit_i64 [
    impl_deserialize_to_uint!(i64:visit_i64 => u8:visit_u8);
    impl_deserialize_to_uint!(i64:visit_i64 => u16:visit_u16);
    impl_deserialize_to_uint!(i64:visit_i64 => u32:visit_u32);
    impl_deserialize_to_uint!(i64:visit_i64 => u64:visit_u64);
    impl_deserialize_to_uint!(i64:visit_i64 => u128:visit_u128);

    impl_deserialize_to_int!(i64:visit_i64 => i8:visit_i8);
    impl_deserialize_to_int!(i64:visit_i64 => i16:visit_i16);
    impl_deserialize_to_int!(i64:visit_i64 => i32:visit_i32);
    impl_deserialize_to_int!(i64:visit_i64 => i128:visit_i128);
]);

impl_deserialize_number!(I128Visitor: i128 => deserialize_i128 => visit_i128 [
    impl_deserialize_to_uint!(i128:visit_i128 => u8:visit_u8);
    impl_deserialize_to_uint!(i128:visit_i128 => u16:visit_u16);
    impl_deserialize_to_uint!(i128:visit_i128 => u32:visit_u32);
    impl_deserialize_to_uint!(i128:visit_i128 => u64:visit_u64);
    impl_deserialize_to_uint!(i128:visit_i128 => u128:visit_u128);

    impl_deserialize_to_int!(i128:visit_i128 => i8:visit_i8);
    impl_deserialize_to_int!(i128:visit_i128 => i16:visit_i16);
    impl_deserialize_to_int!(i128:visit_i128 => i32:visit_i32);
    impl_deserialize_to_int!(i128:visit_i128 => i64:visit_i64);
]);
