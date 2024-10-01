pub trait SequenceSerializer {
    type Err: std::error::Error;
    fn serialize_next<T: Serialize>(&mut self, value: &T) -> Result<(), Self::Err>;
}

pub trait MapSerializer {
    type Err: std::error::Error;
    fn serialize_entry<K: Serialize, V: Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Err>;
}

pub trait Serializer {
    type Err: std::error::Error;
    type Seq: SequenceSerializer<Err = Self::Err>;
    type Map: MapSerializer<Err = Self::Err>;

    fn serialize_unit(&mut self) -> Result<(), Self::Err> {
        Ok(())
    }

    fn serialize_i8(&mut self, value: i8) -> Result<(), Self::Err> {
        self.serialize_i128(value.into())
    }

    fn serialize_i16(&mut self, value: i16) -> Result<(), Self::Err> {
        self.serialize_i128(value.into())
    }

    fn serialize_i32(&mut self, value: i32) -> Result<(), Self::Err> {
        self.serialize_i128(value.into())
    }

    fn serialize_i64(&mut self, value: i64) -> Result<(), Self::Err> {
        self.serialize_i128(value.into())
    }

    fn serialize_i128(&mut self, value: i128) -> Result<(), Self::Err>;

    fn serialize_u8(&mut self, value: u8) -> Result<(), Self::Err> {
        self.serialize_u128(value.into())
    }

    fn serialize_u16(&mut self, value: u16) -> Result<(), Self::Err> {
        self.serialize_u128(value.into())
    }

    fn serialize_u32(&mut self, value: u32) -> Result<(), Self::Err> {
        self.serialize_u128(value.into())
    }

    fn serialize_u64(&mut self, value: u64) -> Result<(), Self::Err> {
        self.serialize_u128(value.into())
    }

    fn serialize_u128(&mut self, value: u128) -> Result<(), Self::Err>;

    fn serialize_f32(&mut self, value: f32) -> Result<(), Self::Err> {
        self.serialize_f64(value.into())
    }

    fn serialize_f64(&mut self, value: f64) -> Result<(), Self::Err>;

    fn serialize_bool(&mut self, value: bool) -> Result<(), Self::Err>;

    fn serialize_str(&mut self, value: &str) -> Result<(), Self::Err>;

    fn serialize_string(&mut self, value: String) -> Result<(), Self::Err> {
        self.serialize_str(&value)
    }

    fn serialize_char(&mut self, value: char) -> Result<(), Self::Err> {
        self.serialize_string(value.to_string())
    }

    fn serialize_slice<T: Serialize>(&mut self, value: &[T]) -> Result<(), Self::Err> {
        let mut seq = self.serialize_sequence()?;

        for x in value {
            seq.serialize_next(x)?;
        }

        Ok(())
    }

    fn serialize_array<T: Serialize, const N: usize>(
        &mut self,
        value: [T; N],
    ) -> Result<(), Self::Err> {
        self.serialize_slice(&value)
    }

    fn serialize_vec<T: Serialize>(&mut self, value: Vec<T>) -> Result<(), Self::Err> {
        self.serialize_slice(&value)
    }

    fn serialize_sequence(&mut self) -> Result<Self::Seq, Self::Err>;

    fn serialize_map(&mut self) -> Result<Self::Map, Self::Err>;
}

pub trait MapIterator {
    fn serialize_next<T>(&mut self, name: &str, value: T) -> bool;
}

pub trait Serialize {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Err>;
}
