use std::marker::PhantomData;

pub trait SequenceSerializer {
    type Err: std::error::Error;
    fn serialize_next<T: Serialize>(&mut self, value: &T) -> Result<(), Self::Err>;
    fn end(self) -> Result<(), Self::Err>;
}

pub trait MapSerializer {
    type Err: std::error::Error;
    fn serialize_entry<K: Serialize, V: Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Err>;
    fn end(self) -> Result<(), Self::Err>;
}

pub trait Serializer: Sized {
    type Err: std::error::Error;
    type Seq: SequenceSerializer<Err = Self::Err>;
    type Map: MapSerializer<Err = Self::Err>;

    fn serialize_unit(self) -> Result<(), Self::Err> {
        Ok(())
    }

    fn serialize_i8(self, value: i8) -> Result<(), Self::Err> {
        self.serialize_i128(value.into())
    }

    fn serialize_i16(self, value: i16) -> Result<(), Self::Err> {
        self.serialize_i128(value.into())
    }

    fn serialize_i32(self, value: i32) -> Result<(), Self::Err> {
        self.serialize_i128(value.into())
    }

    fn serialize_i64(self, value: i64) -> Result<(), Self::Err> {
        self.serialize_i128(value.into())
    }

    fn serialize_i128(self, value: i128) -> Result<(), Self::Err>;

    fn serialize_u8(self, value: u8) -> Result<(), Self::Err> {
        self.serialize_u128(value.into())
    }

    fn serialize_u16(self, value: u16) -> Result<(), Self::Err> {
        self.serialize_u128(value.into())
    }

    fn serialize_u32(self, value: u32) -> Result<(), Self::Err> {
        self.serialize_u128(value.into())
    }

    fn serialize_u64(self, value: u64) -> Result<(), Self::Err> {
        self.serialize_u128(value.into())
    }

    fn serialize_u128(self, value: u128) -> Result<(), Self::Err>;

    fn serialize_f32(self, value: f32) -> Result<(), Self::Err> {
        self.serialize_f64(value.into())
    }

    fn serialize_f64(self, value: f64) -> Result<(), Self::Err>;

    fn serialize_bool(self, value: bool) -> Result<(), Self::Err>;

    fn serialize_str(self, value: &str) -> Result<(), Self::Err>;

    fn serialize_string(self, value: String) -> Result<(), Self::Err> {
        self.serialize_str(&value)
    }

    fn serialize_char(self, value: char) -> Result<(), Self::Err> {
        self.serialize_string(value.to_string())
    }

    fn serialize_option<T: Serialize>(self, value: Option<T>) -> Result<(), Self::Err>;

    fn serialize_slice<T: Serialize>(self, value: &[T]) -> Result<(), Self::Err> {
        let mut seq = self.serialize_sequence()?;

        for x in value {
            seq.serialize_next(x)?;
        }

        Ok(())
    }

    fn serialize_array<T: Serialize, const N: usize>(self, value: [T; N]) -> Result<(), Self::Err> {
        self.serialize_slice(&value)
    }

    fn serialize_vec<T: Serialize>(self, value: Vec<T>) -> Result<(), Self::Err> {
        self.serialize_slice(&value)
    }

    fn serialize_sequence(self) -> Result<Self::Seq, Self::Err>;

    fn serialize_map(self) -> Result<Self::Map, Self::Err>;
}

pub trait MapIterator {
    fn serialize_element<T>(&mut self, name: &str, value: T) -> bool;
}

pub trait Serialize {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<(), S::Err>;
}

