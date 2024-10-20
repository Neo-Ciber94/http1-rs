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
        impl $crate::serde::ser::Serialize for $struct {
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
        $crate::impl_deserialize_struct!($struct => { $($field: $value),* });
        $crate::impl_serialize_struct!($struct => { $($field: $value),* });
    };
}

/// Implement `Serialize` for a enum with unit variants.
#[macro_export]
macro_rules! impl_serialize_enum_str {
    ($enum:ident => { $($variant:ident),* $(,)? }) => {
        impl crate::serde::ser::Serialize for $enum {
            fn serialize<S: $crate::serde::ser::Serializer>(
                &self,
                serializer: S,
            ) -> Result<S::Ok, S::Err> {
                match self {
                    $(
                        $enum :: $variant => {
                            serializer.serialize_str(stringify!($variant))
                        }
                    )*
                }
            }
        }
    };
}

/// Implement `Deserialize` for an enum with unit variants.
#[macro_export]
macro_rules! impl_deserialize_enum_str {
    ($enum:ident => { $($variant:ident),* $(,)? }) => {
        impl $crate::serde::de::Deserialize for $enum {
            fn deserialize<D: $crate::serde::de::Deserializer>(
                deserializer: D,
            ) -> Result<Self, $crate::serde::de::Error> {
                static KNOWN_VARIANTS: &[&str] = &[
                    $(
                        stringify!($variant)
                    ),*
                ];

                let variant = deserializer.deserialize_string($crate::serde::de::StringVisitor)?;

                match variant.as_str() {
                    $(
                        stringify!(stringify!($variant)) => Ok($enum :: $variant),
                    )*
                    v => {
                        let unknown_variant = format!("{}::{v}", stringify!($enum));

                        Err($crate::serde::de::Error::custom(format!(
                            "Unknown enum variant `{unknown_variant}`, valid variants: {KNOWN_VARIANTS:?}"
                        )))
                    },
                }
            }
        }
    };
}

/// Helper for implementing both `Serialize` and `Deserialize` for an enum.
#[macro_export]
macro_rules! impl_serde_enum_str {
    ($enum:ident => { $($variant:ident),* $(,)? }) => {
        $crate::impl_serialize_enum_str!($enum => { $($variant),* });
        $crate::impl_deserialize_enum_str!($enum => { $($variant),* });
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

    enum Color {
        Red,
        Blue,
        Green,
    }

    impl_serialize_enum_str!(Color => {
        Red, Blue, Green
    });

    impl_deserialize_enum_str!(Color => {
        Red, Blue, Green
    });

    enum Fruits {
        Apple,
        Pear,
        Grape,
    }

    impl_serde_enum_str!(Fruits => { Apple, Pear, Grape });
}
