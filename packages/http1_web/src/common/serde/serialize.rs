use std::{borrow::Cow, rc::Rc, sync::Arc};

pub trait SequenceSerializer {
    type Ok;
    type Err: std::error::Error;

    fn serialize_element<T: Serialize>(&mut self, value: &T) -> Result<Self::Ok, Self::Err>;
    fn end(self) -> Result<Self::Ok, Self::Err>;
}

pub trait MapSerializer {
    type Ok;
    type Err: std::error::Error;

    fn serialize_entry<K: Serialize, V: Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<Self::Ok, Self::Err>;
    fn end(self) -> Result<Self::Ok, Self::Err>;
}

pub trait Serializer: Sized {
    type Ok;
    type Err: std::error::Error;
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

    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Err> {
        self.serialize_f64(value.into())
    }

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

    fn serialize_slice<T: Serialize>(self, value: &[T]) -> Result<Self::Ok, Self::Err>;

    fn serialize_array<T: Serialize, const N: usize>(
        self,
        value: [T; N],
    ) -> Result<Self::Ok, Self::Err> {
        self.serialize_slice(&value)
    }

    fn serialize_vec<T: Serialize>(self, value: &Vec<T>) -> Result<Self::Ok, Self::Err> {
        self.serialize_slice(&value)
    }

    fn serialize_sequence(self) -> Result<Self::Seq, Self::Err>;

    fn serialize_map(self) -> Result<Self::Map, Self::Err>;
}

pub trait MapIterator {
    fn serialize_element<T>(&mut self, name: &str, value: T) -> bool;
}

pub trait Serialize {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err>;
}

// Implementations

macro_rules! impl_serialize_primitive {
    ($($name:ident => $method:ident),*) => {
        $(
            impl Serialize for $name {
                fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
                    serializer.$method(*self)
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
    i8 => serialize_i8,
    i16 => serialize_i16,
    i32 => serialize_i32,
    i64 => serialize_i64,
    i128 => serialize_i128,
    f32 => serialize_f32,
    f64 => serialize_f64,
    char => serialize_char,
    bool => serialize_bool
);

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
