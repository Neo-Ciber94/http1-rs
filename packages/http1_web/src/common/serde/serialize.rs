pub trait Serializer {
    fn write_unit(&mut self) {}

    fn write_i8(&mut self, value: i8) {
        self.write_i128(value.into());
    }

    fn write_i16(&mut self, value: i16) {
        self.write_i128(value.into());
    }

    fn write_i32(&mut self, value: i32) {
        self.write_i128(value.into());
    }

    fn write_i64(&mut self, value: i64) {
        self.write_i128(value.into());
    }

    fn write_i128(&mut self, value: i128);

    fn write_u8(&mut self, value: u8) {
        self.write_u128(value.into());
    }

    fn write_u16(&mut self, value: u16) {
        self.write_u128(value.into());
    }

    fn write_u32(&mut self, value: u32) {
        self.write_u128(value.into());
    }

    fn write_u64(&mut self, value: u64) {
        self.write_u128(value.into());
    }

    fn write_u128(&mut self, value: u128);

    fn write_f32(&mut self, value: f32) {
        self.write_f64(value.into());
    }

    fn write_f64(&mut self, value: f64);

    fn write_bool(&mut self, value: bool);

    fn write_str(&mut self, value: &str);

    fn write_string(&mut self, value: String) {
        self.write_str(&value);
    }

    fn write_char(&mut self, value: char) {
        self.write_string(value.to_string());
    }

    fn write_slice<T: Serialize>(&mut self, value: &[T]);

    fn write_array<T: Serialize, const N: usize>(&mut self, value: [T; N]) {
        self.write_slice(&value);
    }

    fn write_vec<T: Serialize>(&mut self, value: Vec<T>) {
        self.write_slice(&value);
    }

    fn write_map<E: IntoIterator<Item = (String, T)>, T: Serialize>(&mut self, data: E);
}

pub trait MapIterator {
    fn write_next<T>(&mut self, name: &str, value: T) -> bool;
}

pub trait Serialize {
    fn serialize<S: Serializer>(&self, serializer: &mut S);
}
