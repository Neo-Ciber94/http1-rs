pub mod bytes;
pub mod de;
pub mod expected;
pub mod impossible;
pub mod json;
pub mod ser;
pub mod string;
pub mod visitor;

/// Implement `Deserialize` for a struct.
#[macro_export]
macro_rules! impl_deserialize_struct {
    ($struct:ident => { $($field:ident : $value:ty),* $(,)? }) => {
        impl $crate::serde::de::Deserialize for $struct {
            fn deserialize<D: $crate::serde::de::Deserializer>(
                deserializer: D,
            ) -> Result<Self, $crate::serde::de::Error> {
                struct StructVisitor;

                impl $crate::serde::visitor::Visitor for StructVisitor {
                    type Value = $struct;

                    fn expected(&self) -> &'static str {
                        "struct"
                    }

                    fn visit_map<Map: $crate::serde::visitor::MapAccess>(
                        self,
                        mut map: Map,
                    ) -> Result<Self::Value, $crate::serde::de::Error>  {
                        $(
                            let mut $field: Result<$value, $crate::serde::de::Error> = Err($crate::serde::de::Error::custom(concat!("missing field '", stringify!($field), "'")));
                        )*

                        while let Some(k) = map.next_key::<String>()?  {
                            match k.as_str() {
                                $(
                                    stringify!($field) => {
                                        $field = match map.next_value::<$value>()? {
                                            Some(x) => Ok(x),
                                            None => {
                                                return Err($crate::serde::de::Error::custom(concat!("missing field '", stringify!($field), "'")));
                                            }
                                        };
                                    }
                                )*

                                _ => {
                                    return Err($crate::serde::de::Error::custom(format!(
                                        "Unknown field '{k}'"
                                    )));
                                }
                            }
                        }

                        Ok($struct {
                            $(
                                $field: $field?
                            ),*
                        })
                    }
                }

                deserializer.deserialize_map(StructVisitor)
            }
        }
    };
}

/// Implement `Serialize` for a struct.
#[macro_export]
macro_rules! impl_serialize_struct {
    ($struct:ident => { $($field:ident : $value:ty),* $(,)? }) => {
        impl crate::serde::ser::Serialize for $struct {
            fn serialize<S: $crate::serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
                use $crate::serde::ser::MapSerializer;

                let mut map = serializer.serialize_map()?;

               $(
                    map.serialize_entry(&stringify!($field), &self.$field)?;
               )*

                map.end()
            }
        }
    };
}

/// Helper for implementing both `Serialize` and `Deserialize` for a struct.
#[macro_export]
macro_rules! impl_serde_struct {
    ($struct:ident => { $($field:ident : $value:ty),* $(,)? }) => {
        impl_deserialize_struct!($struct => { $($field: $value),* });
        impl_serialize_struct!($struct => { $($field: $value),* });
    };
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    struct MyStruct {
        unsigned: u64,
        signed: i64,
        float: f64,
        boolean: bool,
        str: String,
        items: Vec<MyStruct>,
        map: HashMap<String, MyStruct>,
    }

    impl_serialize_struct!(MyStruct => {
        unsigned: u64,
        signed: i64,
        float: f64,
        boolean: bool,
        str: String,
        items: Vec<MyStruct>,
        map: HashMap<String, MyStruct>
    });

    impl_deserialize_struct!(MyStruct => {
        unsigned: u64,
        signed: i64,
        float: f64,
        boolean: bool,
        str: String,
        items: Vec<MyStruct>,
        map: HashMap<String, MyStruct>
    });

    struct OtherStruct {
        unsigned: u64,
        signed: i64,
        float: f64,
        boolean: bool,
        str: String,
        items: Vec<MyStruct>,
        map: HashMap<String, MyStruct>,
    }

    impl_serde_struct!(OtherStruct => {
        unsigned: u64,
        signed: i64,
        float: f64,
        boolean: bool,
        str: String,
        items: Vec<MyStruct>,
        map: HashMap<String, MyStruct>
    });
}
