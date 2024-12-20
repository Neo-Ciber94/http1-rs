use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt::Display,
    ops::Index,
};

use orderedmap::OrderedMap;

use crate::{
    de::{Deserialize, Deserializer, Error},
    ser::{MapSerializer, SequenceSerializer, Serialize, Serializer},
    visitor::{BytesAccess, MapAccess, SeqAccess, Visitor},
};

use super::{
    number::Number,
    ser::{JsonBytesSerializer, JsonSerializationError},
};

/// A JSON value, which can represent various types of data such as numbers, strings,
/// booleans, arrays, objects, or null values.
#[derive(Default, Debug, Clone)]
pub enum JsonValue {
    /// A numeric value (can be a float, integer, or unsigned integer).
    Number(Number),
    /// A string value.
    String(String),
    /// A boolean value.
    Bool(bool),
    /// An array of JSON values.
    Array(Vec<JsonValue>),
    /// An object (a map from strings to JSON values).
    Object(OrderedMap<String, JsonValue>),
    /// A null value.
    #[default]
    Null,
}

impl JsonValue {
    /// Returns `true` if the value is `JsonValue::Null`.
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    /// Returns `true` if the value is `JsonValue::Number`.
    pub fn is_number(&self) -> bool {
        matches!(self, JsonValue::Number(_))
    }

    /// Returns `true` if the value is `JsonValue::Bool`.
    pub fn is_bool(&self) -> bool {
        matches!(self, JsonValue::Bool(_))
    }

    /// Returns `true` if the value is `JsonValue::String`.
    pub fn is_string(&self) -> bool {
        matches!(self, JsonValue::String(_))
    }

    /// Returns `true` if the value is `JsonValue::Array`.
    pub fn is_array(&self) -> bool {
        matches!(self, JsonValue::Array(_))
    }

    /// Returns `true` if the value is `JsonValue::Object`.
    pub fn is_object(&self) -> bool {
        matches!(self, JsonValue::Object(_))
    }

    /// Returns `true` if the value is a floating-point number (`JsonValue::Number::Float`).
    pub fn is_f64(&self) -> bool {
        matches!(self, JsonValue::Number(Number::Float(_)))
    }

    /// Returns `true` if the value is an unsigned integer (`JsonValue::Number::UInteger`).
    pub fn is_u128(&self) -> bool {
        matches!(self, JsonValue::Number(Number::UInteger(_)))
    }

    /// Returns `true` if the value is a signed integer (`JsonValue::Number::Integer`).
    pub fn is_i128(&self) -> bool {
        matches!(self, JsonValue::Number(Number::Integer(_)))
    }

    /// If the value is a `JsonValue::String`, returns the string as a `&str`.
    /// Otherwise, returns `None`.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// If the value is a `JsonValue::String`, returns the string as a mutable reference.
    /// Otherwise, returns `None`.
    pub fn as_string_mut(&mut self) -> Option<&mut String> {
        match self {
            JsonValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// If the value is a `JsonValue::Number`, returns the number.
    /// Otherwise, returns `None`.
    pub fn as_number(&self) -> Option<Number> {
        match self {
            JsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// If the value is a `JsonValue::Bool`, returns the boolean.
    /// Otherwise, returns `None`.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// If the value is a `JsonValue::Array`, returns the array as a slice of JSON values.
    /// Otherwise, returns `None`.
    pub fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// If the value is a `JsonValue::Array`, returns the array as a mutable reference
    /// to a vector of JSON values. Otherwise, returns `None`.
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<JsonValue>> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// If the value is a `JsonValue::Object`, returns the object as a reference to a map.
    /// Otherwise, returns `None`.
    pub fn as_map(&self) -> Option<&OrderedMap<String, JsonValue>> {
        match self {
            JsonValue::Object(map) => Some(map),
            _ => None,
        }
    }

    /// If the value is a `JsonValue::Object`, returns the object as a mutable reference
    /// to a map. Otherwise, returns `None`.
    pub fn as_map_mut(&mut self) -> Option<&mut OrderedMap<String, JsonValue>> {
        match self {
            JsonValue::Object(map) => Some(map),
            _ => None,
        }
    }

    /// Takes the current value and replaces it with `JsonValue::Null`.
    ///
    /// This can be used to extract the current value while leaving a null in its place.
    pub fn take(&mut self) -> JsonValue {
        std::mem::take(self)
    }

    /// Returns a reference element to the given index.
    pub fn get<I: JsonIndex>(&self, index: I) -> Option<&JsonValue> {
        index.get(self)
    }

    /// Returns a reference mutable element to the given index.
    pub fn get_mut<I: JsonIndex>(&mut self, index: I) -> Option<&mut JsonValue> {
        index.get_mut(self)
    }

    /// Removes the element at the given index.
    ///
    /// # Panics
    /// - This element is not a json
    pub fn remove<I: JsonIndexMut>(&mut self, index: I) -> Option<JsonValue> {
        index.remove(self)
    }

    /// Insert or replace the element at the given index and returns the previous value if any.
    ///
    /// # Panics
    /// - This element is not a json
    /// - This element is an array and the index is out of bounds
    pub fn insert<I: JsonIndexMut>(&mut self, index: I, new_value: JsonValue) -> Option<JsonValue> {
        index.insert(self, new_value)
    }

    /// Removes the element at the given index.
    pub fn try_remove<I: JsonIndexMut>(
        &mut self,
        index: I,
    ) -> Result<Option<JsonValue>, TryRemoveError> {
        index.try_remove(self)
    }

    /// Insert or replace the element at the given index and returns the previous value if any.
    pub fn try_insert<I: JsonIndexMut>(
        &mut self,
        index: I,
        new_value: JsonValue,
    ) -> Result<Option<JsonValue>, TryInsertError> {
        index.try_insert(self, new_value)
    }

    /// Selects a nested `JsonValue` based on a dot-separated path.
    ///
    /// The path can specify nested fields or indices using dot-separated components.
    /// For example:
    /// - `"students.0.name"` will access the name of the first student in an array.
    /// - `"skills.2"` will access the third skill in the skills array.
    ///
    /// # Parameters
    /// - `path`: A dot-separated string representing the nested path to traverse. If the path is empty or `"."`, the method returns the current value.
    ///
    /// # Returns
    /// - `Some(&JsonValue)` if the path is valid and the corresponding value exists.
    /// - `None` if the path is invalid or the value does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde::json;
    ///
    /// let jjk = json!({
    ///     name: "Satoru Gojo",
    ///     age: 28,
    ///     is_active: true,
    ///     skills: [
    ///         "Infinity",
    ///         "Limitless",
    ///         "Domain Expansion: Unlimited Void"
    ///     ],
    ///     students: [
    ///         json!({
    ///             name: "Yuji Itadori",
    ///             age: 15
    ///         }),
    ///         json!({
    ///             name: "Megumi Fushiguro",
    ///             age: 16
    ///         })
    ///     ]
    /// });
    ///
    /// assert_eq!(jjk.select("students.0.name"), Some(&json!("Yuji Itadori")));
    /// assert_eq!(jjk.select("skills.2"), Some(&json!("Domain Expansion: Unlimited Void")));
    /// assert_eq!(jjk.select("age"), Some(&json!(28)));
    /// assert_eq!(jjk.select("students.1.age"), Some(&json!(16)));
    /// ```
    pub fn select(&self, path: &str) -> Option<&JsonValue> {
        if path.is_empty() || path == "." {
            return Some(self);
        }

        match path.split_once(".") {
            None => match self {
                JsonValue::Array(vec) => {
                    let index: usize = path.parse().ok()?;
                    vec.get(index)
                }
                JsonValue::Object(map) => map.get(path),
                _ => None,
            },
            Some((sub_path, rest)) => match self {
                JsonValue::Array(vec) => {
                    let index: usize = sub_path.parse().ok()?;
                    vec.get(index).and_then(|v| v.select(rest))
                }
                JsonValue::Object(map) => map.get(sub_path).and_then(|v| v.select(rest)),
                _ => None,
            },
        }
    }

    /// Selects a mutable reference to a nested `JsonValue` based on a dot-separated path.
    ///
    /// The path can specify nested fields or indices using dot-separated components.
    /// For example:
    /// - `"students.0.name"` will access the name of the first student in an array.
    /// - `"skills.2"` will access the third skill in the skills array.
    ///
    /// # Parameters
    /// - `path`: A dot-separated string representing the nested path to traverse. If the path is empty or `"."`, the method returns the current value.
    ///
    /// # Returns
    /// - `Some(&mut JsonValue)` if the path is valid and the corresponding value exists.
    /// - `None` if the path is invalid or the value does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde::json;
    /// let mut jjk = json!({
    ///     name: "Satoru Gojo",
    ///     age: 28,
    ///     is_active: true,
    ///     skills: [
    ///         "Infinity",
    ///         "Limitless",
    ///         "Domain Expansion: Unlimited Void"
    ///     ],
    ///     students: [
    ///         json!({
    ///             name: "Yuji Itadori",
    ///             age: 15
    ///         }),
    ///         json!({
    ///             name: "Megumi Fushiguro",
    ///             age: 16
    ///         })
    ///     ]
    /// });
    ///
    /// if let Some(age) = jjk.select_mut("age") {
    ///     *age = json!(29);
    /// }
    ///
    /// assert_eq!(jjk.select("age"), Some(&json!(29)));
    ///
    /// if let Some(name) = jjk.select_mut("students.0.name") {
    ///     *name = json!("Yuji Itadori (updated)");
    /// }
    ///
    /// assert_eq!(jjk.select("students.0.name"), Some(&json!("Yuji Itadori (updated)")));
    /// ```
    pub fn select_mut(&mut self, path: &str) -> Option<&mut JsonValue> {
        if path.is_empty() || path == "." {
            return Some(self);
        }

        match path.split_once(".") {
            None => match self {
                JsonValue::Array(vec) => {
                    let index: usize = path.parse().ok()?;
                    vec.get_mut(index)
                }
                JsonValue::Object(map) => map.get_mut(path),
                _ => None,
            },
            Some((sub_path, rest)) => match self {
                JsonValue::Array(vec) => {
                    let index: usize = sub_path.parse().ok()?;
                    vec.get_mut(index).and_then(|v| v.select_mut(rest))
                }
                JsonValue::Object(map) => map.get_mut(sub_path).and_then(|v| v.select_mut(rest)),
                _ => None,
            },
        }
    }

    /// Returns the string representation of the type of the `JsonValue` (e.g., "string", "array").
    pub fn variant(&self) -> &'static str {
        match self {
            JsonValue::Number(_) => "number",
            JsonValue::String(_) => "string",
            JsonValue::Bool(_) => "boolean",
            JsonValue::Array(_) => "array",
            JsonValue::Object(_) => "object",
            JsonValue::Null => "null",
        }
    }
}

/// Helper for creating `JsonValue`
#[macro_export]
macro_rules! json {
    () => {
        $crate::json::value::JsonValue::Object(Default::default())
    };

    (null) =>  {
        $crate::json::value::JsonValue::Null
    };

    ($value:literal) => {
        $crate::json::value::JsonValue::from($value)
    };

    ([$($item:expr),* $(,)?]) => {
        {
            $crate::json::value::JsonValue::Array(vec![
                $(
                    json!($item)
                ),*
            ])
        }
    };

    ({ $($key:ident : $value:expr),* $(,)?}) => {
        {
            #[allow(unused_mut)]
           let mut map = $crate::re_exports::OrderedMap::<String, $crate::json::value::JsonValue>::new();
           $(
                map.insert({ stringify!($key) }.into(), $crate::json::value::JsonValue::from($value));
           )*

           $crate::json::value::JsonValue::Object(map)
        }
    }
}

impl Display for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = super::to_pretty_string(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{s}")
    }
}

impl PartialEq for JsonValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JsonValue::Number(a), JsonValue::Number(b)) => a == b,
            (JsonValue::String(a), JsonValue::String(b)) => a == b,
            (JsonValue::Bool(a), JsonValue::Bool(b)) => a == b,
            (JsonValue::Array(a), JsonValue::Array(b)) => a == b,
            (JsonValue::Object(a), JsonValue::Object(b)) => a.iter().eq(b.iter()),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl<I: JsonIndex> Index<I> for JsonValue {
    type Output = JsonValue;

    fn index(&self, index: I) -> &Self::Output {
        index.index(self)
    }
}

#[derive(Debug)]
pub enum TryInsertError {
    ExpectedArrayOrMap,
    ExpectedMap,
    OutOfBounds,
}

impl std::error::Error for TryInsertError {}

impl std::fmt::Display for TryInsertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TryInsertError::ExpectedArrayOrMap => {
                write!(f, "failed to insert value, expected array or map")
            }
            TryInsertError::ExpectedMap => write!(f, "failed to insert value, expected map"),
            TryInsertError::OutOfBounds => write!(f, "failed to insert, index out of bounds"),
        }
    }
}

#[derive(Debug)]
pub enum TryRemoveError {
    ExpectedArrayOrMap,
    ExpectedMap,
}

impl std::error::Error for TryRemoveError {}

impl std::fmt::Display for TryRemoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TryRemoveError::ExpectedArrayOrMap => {
                write!(f, "failed to remove value, expected array or map")
            }
            TryRemoveError::ExpectedMap => write!(f, "failed to remove value, expected map"),
        }
    }
}

#[doc(hidden)]
pub trait JsonIndexMut: Sized + Copy + std::fmt::Display {
    fn try_insert(
        self,
        json: &mut JsonValue,
        new_value: JsonValue,
    ) -> Result<Option<JsonValue>, TryInsertError>;

    fn try_remove(self, json: &mut JsonValue) -> Result<Option<JsonValue>, TryRemoveError>;

    fn insert(self, json: &mut JsonValue, new_value: JsonValue) -> Option<JsonValue> {
        match self.try_insert(json, new_value) {
            Ok(x) => x,
            Err(TryInsertError::ExpectedArrayOrMap) => {
                panic!(
                    "cannot insert value in a json `{}`, expected a array or map",
                    json.variant()
                )
            }
            Err(TryInsertError::ExpectedMap) => {
                panic!(
                    "cannot insert value in a json `{}`, expected a map",
                    json.variant()
                )
            }
            Err(TryInsertError::OutOfBounds) => {
                panic!("index `{self}` is out of bounds")
            }
        }
    }

    fn remove(self, json: &mut JsonValue) -> Option<JsonValue> {
        match self.try_remove(json) {
            Ok(x) => x,
            Err(TryRemoveError::ExpectedArrayOrMap) => {
                panic!(
                    "cannot remove `{self}` from a json `{}`, expected array or map",
                    json.variant()
                )
            }
            Err(TryRemoveError::ExpectedMap) => {
                panic!(
                    "cannot remove `{self}` from a json `{}`, expected a map",
                    json.variant()
                )
            }
        }
    }
}

impl JsonIndexMut for usize {
    fn try_insert(
        self,
        json: &mut JsonValue,
        new_value: JsonValue,
    ) -> Result<Option<JsonValue>, TryInsertError> {
        match json {
            JsonValue::Array(vec) => match vec.get_mut(self) {
                Some(x) => Ok(Some(std::mem::replace(x, new_value))),
                None => Err(TryInsertError::OutOfBounds),
            },
            JsonValue::Object(map) => match map.get_index_mut(self) {
                Some(x) => Ok(Some(std::mem::replace(x, new_value))),
                None => Err(TryInsertError::OutOfBounds),
            },
            _ => Err(TryInsertError::ExpectedArrayOrMap),
        }
    }

    fn try_remove(self, json: &mut JsonValue) -> Result<Option<JsonValue>, TryRemoveError> {
        match json {
            JsonValue::Array(vec) => {
                if self > vec.len() {
                    Ok(None)
                } else {
                    Ok(Some(vec.remove(self)))
                }
            }
            JsonValue::Object(map) => {
                if self > map.len() {
                    Ok(None)
                } else {
                    Ok(map.remove_index(self))
                }
            }
            _ => Err(TryRemoveError::ExpectedArrayOrMap),
        }
    }
}

impl<'a> JsonIndexMut for &'a str {
    fn try_insert(
        self,
        json: &mut JsonValue,
        new_value: JsonValue,
    ) -> Result<Option<JsonValue>, TryInsertError> {
        match json {
            JsonValue::Object(map) => match map.get_mut(self) {
                Some(x) => Ok(Some(std::mem::replace(x, new_value))),
                None => {
                    map.insert(self.to_owned(), new_value);
                    Ok(None)
                }
            },
            _ => Err(TryInsertError::ExpectedMap),
        }
    }

    fn try_remove(self, json: &mut JsonValue) -> Result<Option<JsonValue>, TryRemoveError> {
        match json {
            JsonValue::Object(map) => Ok(map.remove(self)),
            _ => Err(TryRemoveError::ExpectedMap),
        }
    }
}

/// Allow to index a json value object or array.
#[doc(hidden)]
pub trait JsonIndex {
    fn get(self, json: &JsonValue) -> Option<&JsonValue>;
    fn get_mut(self, json: &mut JsonValue) -> Option<&mut JsonValue>;
    fn index(self, json: &JsonValue) -> &JsonValue;
    fn index_mut(self, json: &mut JsonValue) -> &mut JsonValue;
}

impl JsonIndex for usize {
    fn get(self, json: &JsonValue) -> Option<&JsonValue> {
        match json {
            JsonValue::Array(vec) => vec.get(self),
            JsonValue::Object(map) => map.get_index(self),
            _ => None,
        }
    }

    fn get_mut(self, json: &mut JsonValue) -> Option<&mut JsonValue> {
        match json {
            JsonValue::Array(vec) => vec.get_mut(self),
            JsonValue::Object(map) => map.get_index_mut(self),
            _ => None,
        }
    }

    fn index(self, json: &JsonValue) -> &JsonValue {
        match json {
            JsonValue::Array(vec) => &vec[self],
            JsonValue::Object(map) => map
                .get_index(self)
                .unwrap_or_else(|| panic!("index {self} is out of range: {self} > {}", map.len())),
            _ => panic!("cannot index a json `{}` as an array", json.variant()),
        }
    }

    fn index_mut(self, json: &mut JsonValue) -> &mut JsonValue {
        match json {
            JsonValue::Array(vec) => &mut vec[self],
            _ => panic!("cannot index a `{}` as an array", json.variant()),
        }
    }
}

impl<'a> JsonIndex for &'a str {
    fn get(self, json: &JsonValue) -> Option<&JsonValue> {
        match json {
            JsonValue::Object(map) => map.get(self),
            _ => None,
        }
    }

    fn get_mut(self, json: &mut JsonValue) -> Option<&mut JsonValue> {
        match json {
            JsonValue::Object(map) => map.get_mut(self),
            _ => None,
        }
    }

    fn index(self, json: &JsonValue) -> &JsonValue {
        match json {
            JsonValue::Object(map) => map
                .get(self)
                .unwrap_or_else(|| panic!("no value found for key `{self}`")),
            _ => panic!("cannot index a json `{}` as a map", json.variant()),
        }
    }

    fn index_mut(self, json: &mut JsonValue) -> &mut JsonValue {
        match json {
            JsonValue::Object(map) => map
                .get_mut(self)
                .unwrap_or_else(|| panic!("no value found for key `{self}`")),
            _ => panic!("cannot index a json `{}` as a map", json.variant()),
        }
    }
}

impl From<()> for JsonValue {
    fn from(_value: ()) -> Self {
        JsonValue::Null
    }
}

impl From<String> for JsonValue {
    fn from(value: String) -> Self {
        JsonValue::String(value)
    }
}

impl<'a> From<&'a str> for JsonValue {
    fn from(value: &'a str) -> Self {
        JsonValue::String(value.into())
    }
}

impl From<bool> for JsonValue {
    fn from(value: bool) -> Self {
        JsonValue::Bool(value)
    }
}

impl From<u8> for JsonValue {
    fn from(value: u8) -> Self {
        JsonValue::Number(Number::from(value))
    }
}

impl From<u16> for JsonValue {
    fn from(value: u16) -> Self {
        JsonValue::Number(Number::from(value))
    }
}

impl From<u32> for JsonValue {
    fn from(value: u32) -> Self {
        JsonValue::Number(Number::from(value))
    }
}

impl From<u64> for JsonValue {
    fn from(value: u64) -> Self {
        JsonValue::Number(Number::from(value))
    }
}

impl From<u128> for JsonValue {
    fn from(value: u128) -> Self {
        JsonValue::Number(Number::from(value))
    }
}

impl From<i8> for JsonValue {
    fn from(value: i8) -> Self {
        JsonValue::Number(Number::from(value))
    }
}

impl From<i16> for JsonValue {
    fn from(value: i16) -> Self {
        JsonValue::Number(Number::from(value))
    }
}

impl From<i32> for JsonValue {
    fn from(value: i32) -> Self {
        JsonValue::Number(Number::from(value))
    }
}

impl From<i64> for JsonValue {
    fn from(value: i64) -> Self {
        JsonValue::Number(Number::from(value))
    }
}

impl From<i128> for JsonValue {
    fn from(value: i128) -> Self {
        JsonValue::Number(Number::from(value))
    }
}

impl From<f32> for JsonValue {
    fn from(value: f32) -> Self {
        JsonValue::Number(Number::from(value))
    }
}

impl From<f64> for JsonValue {
    fn from(value: f64) -> Self {
        JsonValue::Number(Number::from(value))
    }
}

impl From<isize> for JsonValue {
    fn from(value: isize) -> Self {
        JsonValue::Number(Number::from(value))
    }
}

impl From<usize> for JsonValue {
    fn from(value: usize) -> Self {
        JsonValue::Number(Number::from(value))
    }
}

impl<T: Into<JsonValue>> From<Vec<T>> for JsonValue {
    fn from(value: Vec<T>) -> Self {
        let arr = value.into_iter().map(|x| x.into()).collect::<Vec<_>>();
        JsonValue::Array(arr)
    }
}

impl<T: Into<JsonValue>> From<HashSet<T>> for JsonValue {
    fn from(value: HashSet<T>) -> Self {
        let arr = value.into_iter().map(|x| x.into()).collect::<Vec<_>>();
        JsonValue::Array(arr)
    }
}

impl<T: Into<JsonValue>> From<BTreeSet<T>> for JsonValue {
    fn from(value: BTreeSet<T>) -> Self {
        let arr = value.into_iter().map(|x| x.into()).collect::<Vec<_>>();
        JsonValue::Array(arr)
    }
}

impl<T: Into<JsonValue>> From<Option<T>> for JsonValue {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(x) => x.into(),
            None => JsonValue::Null,
        }
    }
}

impl<V: Into<JsonValue>> From<HashMap<String, V>> for JsonValue {
    fn from(value: HashMap<String, V>) -> Self {
        let mut map = OrderedMap::new();

        for (k, v) in value {
            map.insert(k, v.into());
        }

        JsonValue::Object(map)
    }
}

impl<V: Into<JsonValue>> From<BTreeMap<String, V>> for JsonValue {
    fn from(value: BTreeMap<String, V>) -> Self {
        let mut map = OrderedMap::new();

        for (k, v) in value {
            map.insert(k, v.into());
        }

        JsonValue::Object(map)
    }
}

impl<V: Into<JsonValue>> From<OrderedMap<String, V>> for JsonValue {
    fn from(value: OrderedMap<String, V>) -> Self {
        let mut map = OrderedMap::new();

        for (k, v) in value {
            map.insert(k, v.into());
        }

        JsonValue::Object(map)
    }
}

impl<T: Into<JsonValue>, const N: usize> From<[T; N]> for JsonValue {
    fn from(value: [T; N]) -> Self {
        let arr = value.into_iter().map(|x| x.into()).collect::<Vec<_>>();
        JsonValue::Array(arr)
    }
}

impl Serialize for Number {
    fn serialize<S: crate::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        match self {
            Number::Float(f) => serializer.serialize_f64(*f),
            Number::UInteger(u) => serializer.serialize_u128(*u),
            Number::Integer(i) => serializer.serialize_i128(*i),
        }
    }
}

impl Serialize for JsonValue {
    fn serialize<S: crate::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Err> {
        match self {
            JsonValue::Number(number) => number.serialize(serializer),
            JsonValue::String(s) => serializer.serialize_str(s),
            JsonValue::Bool(b) => serializer.serialize_bool(*b),
            JsonValue::Array(vec) => serializer.serialize_vec(vec),
            JsonValue::Object(obj) => {
                let mut map = serializer.serialize_map()?;

                for (key, value) in obj.iter() {
                    map.serialize_entry(key, value)?;
                }

                map.end()
            }
            JsonValue::Null => serializer.serialize_none(),
        }
    }
}

pub struct JsonValueSerializer;
impl Serializer for JsonValueSerializer {
    type Ok = JsonValue;
    type Err = JsonSerializationError;
    type Bytes = JsonBytesSerializer;
    type Seq = JsonArraySerializer;
    type Map = JsonObjectSerializer;

    fn serialize_unit(self) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::Null)
    }

    fn serialize_i128(self, value: i128) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::Number(Number::from(value)))
    }

    fn serialize_u128(self, value: u128) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::Number(Number::from(value)))
    }

    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::Number(Number::from(value)))
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::Number(Number::from(value)))
    }

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::Bool(value))
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::String(value.to_string()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::Null)
    }

    fn serialize_sequence(self) -> Result<Self::Seq, Self::Err> {
        Ok(Default::default())
    }

    fn serialize_map(self) -> Result<Self::Map, Self::Err> {
        Ok(Default::default())
    }

    fn serialize_byte_seq(self) -> Result<Self::Bytes, Self::Err> {
        Ok(Default::default())
    }
}

#[derive(Default)]
pub struct JsonArraySerializer(Vec<JsonValue>);
impl SequenceSerializer for JsonArraySerializer {
    type Ok = JsonValue;
    type Err = JsonSerializationError;

    fn serialize_element<T: Serialize>(&mut self, value: &T) -> Result<(), Self::Err> {
        let mut arr = vec![];
        let json_value = value.serialize(JsonValueSerializer)?;
        arr.push(json_value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::Array(self.0))
    }
}

#[derive(Default)]
pub struct JsonObjectSerializer(OrderedMap<String, JsonValue>);
impl MapSerializer for JsonObjectSerializer {
    type Ok = JsonValue;
    type Err = JsonSerializationError;

    fn serialize_entry<K: Serialize, V: Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Err> {
        let k = key.serialize(JsonValueSerializer)?;
        let v = value.serialize(JsonValueSerializer)?;

        let JsonValue::String(s) = k else {
            return Err(JsonSerializationError::Other(
                "json object keys should be strings".into(),
            ));
        };

        self.0.insert(s, v);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Err> {
        Ok(JsonValue::Object(self.0))
    }
}

impl Deserializer for JsonValue {
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: crate::visitor::Visitor,
    {
        match self {
            JsonValue::Null => visitor.visit_unit(),
            _ => Err(Error::other("expected unit")),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: crate::visitor::Visitor,
    {
        match self {
            JsonValue::Bool(value) => visitor.visit_bool(value),
            _ => Err(Error::other("expected boolean")),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: crate::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value.as_u8().ok_or_else(|| Error::other("expected u8"))?;
                visitor.visit_u8(n)
            }
            _ => Err(Error::other("expected u8")),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: crate::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value.as_u16().ok_or_else(|| Error::other("expected u16"))?;
                visitor.visit_u16(n)
            }
            _ => Err(Error::other("expected u16")),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: crate::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value.as_u32().ok_or_else(|| Error::other("expected u32"))?;
                visitor.visit_u32(n)
            }
            _ => Err(Error::other("expected u32")),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: crate::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value.as_u64().ok_or_else(|| Error::other("expected u64"))?;
                visitor.visit_u64(n)
            }
            _ => Err(Error::other("expected u64")),
        }
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: crate::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value
                    .as_u128()
                    .ok_or_else(|| Error::other("expected u128"))?;
                visitor.visit_u128(n)
            }
            _ => Err(Error::other("expected u128")),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: crate::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value.as_i8().ok_or_else(|| Error::other("expected i8"))?;
                visitor.visit_i8(n)
            }
            _ => Err(Error::other("expected i8")),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: crate::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value.as_i16().ok_or_else(|| Error::other("expected i16"))?;
                visitor.visit_i16(n)
            }
            _ => Err(Error::other("expected i16")),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: crate::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value.as_i32().ok_or_else(|| Error::other("expected i32"))?;
                visitor.visit_i32(n)
            }
            _ => Err(Error::other("expected i32")),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: crate::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value.as_i64().ok_or_else(|| Error::other("expected i64"))?;
                visitor.visit_i64(n)
            }
            _ => Err(Error::other("expected i64")),
        }
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: crate::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value
                    .as_i128()
                    .ok_or_else(|| Error::other("expected i128"))?;
                visitor.visit_i128(n)
            }
            _ => Err(Error::other("expected i128")),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: crate::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value.as_f32().ok_or_else(|| Error::other("expected f32"))?;
                visitor.visit_f32(n)
            }
            _ => Err(Error::other("expected f32")),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: crate::visitor::Visitor,
    {
        match self {
            JsonValue::Number(value) => {
                let n = value.as_f64().ok_or_else(|| Error::other("expected f64"))?;
                visitor.visit_f64(n)
            }
            _ => Err(Error::other("expected f64")),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: crate::visitor::Visitor,
    {
        match self {
            JsonValue::String(mut value) => {
                if value.is_empty() {
                    return Err(Error::other("expected char but was empty string"));
                }

                if value.len() > 1 {
                    return Err(Error::other("expected char but was string"));
                }

                let c = value.pop().unwrap();
                visitor.visit_char(c)
            }
            _ => Err(Error::other("expected char")),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: crate::visitor::Visitor,
    {
        match self {
            JsonValue::String(value) => visitor.visit_string(value),
            _ => Err(Error::other("expected string")),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: crate::visitor::Visitor,
    {
        match self {
            JsonValue::Array(value) => visitor.visit_seq(JsonSeqAccess(value.into_iter())),
            _ => Err(Error::other("expected array")),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, crate::de::Error>
    where
        V: crate::visitor::Visitor,
    {
        match self {
            JsonValue::Object(value) => visitor.visit_map(JsonObjectAccess::new(value.into_iter())),
            _ => Err(Error::other("expected object")),
        }
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor,
    {
        match self {
            JsonValue::Number(number) => match number {
                Number::Float(f) => visitor.visit_f64(f),
                Number::UInteger(u) => visitor.visit_u128(u),
                Number::Integer(i) => visitor.visit_i128(i),
            },
            JsonValue::String(s) => visitor.visit_string(s),
            JsonValue::Bool(value) => visitor.visit_bool(value),
            JsonValue::Array(vec) => visitor.visit_seq(JsonSeqAccess(vec.into_iter())),
            JsonValue::Object(ordered_map) => {
                visitor.visit_map(JsonObjectAccess::new(ordered_map.into_iter()))
            }
            JsonValue::Null => visitor.visit_none(),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor,
    {
        match self {
            JsonValue::Null => visitor.visit_none(),
            s => visitor.visit_some(s),
        }
    }

    fn deserialize_bytes_buf<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor,
    {
        match self {
            JsonValue::String(s) => {
                let bytes = s.into_bytes();
                visitor.visit_bytes_buf(bytes)
            }
            _ => Err(Error::mismatch(crate::de::Unexpected::Bytes, "json")),
        }
    }

    fn deserialize_bytes_seq<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor,
    {
        match self {
            JsonValue::String(value) => {
                let bytes = value.into_bytes();
                visitor.visit_bytes_seq(JsonBytesAccess(bytes))
            }
            _ => Err(Error::other("expected bytes")),
        }
    }
}

pub struct JsonSeqAccess(pub std::vec::IntoIter<JsonValue>);
impl SeqAccess for JsonSeqAccess {
    fn next_element<T: crate::de::Deserialize>(&mut self) -> Result<Option<T>, Error> {
        match self.0.next() {
            Some(x) => {
                let value = T::deserialize(x)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}

pub struct JsonObjectAccess<I: Iterator<Item = (String, JsonValue)>> {
    iter: I,
    value: Option<JsonValue>,
}

impl<I: Iterator<Item = (String, JsonValue)>> JsonObjectAccess<I> {
    pub fn new(iter: I) -> Self {
        JsonObjectAccess { iter, value: None }
    }
}

impl<I: Iterator<Item = (String, JsonValue)>> MapAccess for JsonObjectAccess<I> {
    fn next_key<K: Deserialize>(&mut self) -> Result<Option<K>, Error> {
        match self.iter.next() {
            Some((k, v)) => {
                self.value = Some(v);
                let key = K::deserialize(JsonValue::String(k))?;
                Ok(Some(key))
            }
            None => Ok(None),
        }
    }

    fn next_value<V: Deserialize>(&mut self) -> Result<Option<V>, Error> {
        match self.value.take() {
            Some(v) => {
                let value = V::deserialize(v)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}

pub struct JsonBytesAccess(Vec<u8>);
impl JsonBytesAccess {
    pub fn new(bytes: Vec<u8>) -> Self {
        JsonBytesAccess(bytes)
    }
}
impl BytesAccess for JsonBytesAccess {
    fn next_bytes<W: std::io::Write>(&mut self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(self.0.as_slice()).map_err(Error::other)
    }
}

impl Deserialize for JsonValue {
    fn deserialize<D: Deserializer>(deserializer: D) -> Result<Self, Error> {
        struct ValueVisitor;
        impl Visitor for ValueVisitor {
            type Value = JsonValue;

            fn expected(&self) -> &'static str {
                "json value"
            }

            fn visit_unit(self) -> Result<Self::Value, Error> {
                Ok(JsonValue::Null)
            }

            fn visit_bool(self, value: bool) -> Result<Self::Value, Error> {
                Ok(JsonValue::Bool(value))
            }

            fn visit_u128(self, value: u128) -> Result<Self::Value, Error> {
                Ok(JsonValue::Number(value.into()))
            }

            fn visit_i128(self, value: i128) -> Result<Self::Value, Error> {
                Ok(JsonValue::Number(value.into()))
            }

            fn visit_f64(self, value: f64) -> Result<Self::Value, Error> {
                Ok(JsonValue::Number(value.into()))
            }

            fn visit_none(self) -> Result<Self::Value, Error> {
                Ok(JsonValue::Null)
            }

            fn visit_some<D: Deserializer>(self, deserializer: D) -> Result<Self::Value, Error> {
                deserializer.deserialize_any(ValueVisitor)
            }

            fn visit_string(self, value: String) -> Result<Self::Value, Error> {
                Ok(JsonValue::String(value))
            }

            fn visit_seq<Seq: SeqAccess>(self, mut seq: Seq) -> Result<Self::Value, Error> {
                let mut items = vec![];

                while let Some(x) = seq.next_element()? {
                    items.push(x);
                }

                Ok(JsonValue::Array(items))
            }

            fn visit_map<Map: MapAccess>(self, mut map: Map) -> Result<Self::Value, Error> {
                let mut obj: OrderedMap<String, JsonValue> = OrderedMap::new();

                while let Some((key, value)) = map.next_entry()? {
                    obj.insert(key, value);
                }

                Ok(JsonValue::Object(obj))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

#[cfg(test)]
mod tests {
    use orderedmap::OrderedMap;

    use crate::json::value::JsonValue;

    #[test]
    fn should_build_json_using_macro() {
        // null
        assert_eq!(json! { null }, JsonValue::Null);

        // number
        assert_eq!(json! { 23 }, JsonValue::from(23));

        // boolean
        assert_eq!(json! { true }, JsonValue::from(true));

        // string
        assert_eq!(json! { "Pineapple" }, JsonValue::from("Pineapple"));

        // array
        assert_eq!(
            json! { [0.5f32, true, "Makoto"] },
            JsonValue::Array(vec![
                JsonValue::from(0.5f32),
                JsonValue::from(true),
                JsonValue::from("Makoto")
            ])
        );

        // empty object
        assert_eq!(json!(), JsonValue::Object(Default::default()));

        // object
        let mut map = OrderedMap::default();
        map.insert(String::from("name"), JsonValue::from("Makoto Hanaoka"));
        map.insert(String::from("age"), JsonValue::from(17));
        map.insert(String::from("in_love"), JsonValue::from(true));
        map.insert(
            String::from("friends"),
            JsonValue::from([JsonValue::from("Ryuji Taiga"), JsonValue::from("Saki Aoi")]),
        );

        assert_eq!(
            json! { {
                name: "Makoto Hanaoka",
                age: 17,
                in_love: true,
                friends: ["Ryuji Taiga", "Saki Aoi"]
            }},
            JsonValue::Object(map)
        )
    }

    #[test]
    fn should_select_json_value() {
        let jjk = json!({
            name: "Satoru Gojo",
            age: 28,
            is_active: true,
            skills: [
                "Infinity",
                "Limitless",
                "Domain Expansion: Unlimited Void"
            ],
            students: [
                json!({
                    name: "Yuji Itadori",
                    age: 15,
                }),
                json!({
                    name: "Megumi Fushiguro",
                    age: 16
                })
            ]
        });

        // Select first
        assert_eq!(jjk.select("name").unwrap(), &JsonValue::from("Satoru Gojo"));
        assert_eq!(jjk.select("age").unwrap(), &JsonValue::from(28));
        assert_eq!(jjk.select("is_active").unwrap(), &JsonValue::from(true));
        assert_eq!(
            jjk.select("skills").unwrap(),
            &JsonValue::from(vec![
                JsonValue::from("Infinity"),
                JsonValue::from("Limitless"),
                JsonValue::from("Domain Expansion: Unlimited Void"),
            ])
        );

        // Select array item
        assert_eq!(
            jjk.select("skills.0").unwrap(),
            &JsonValue::from("Infinity")
        );
        assert_eq!(
            jjk.select("skills.1").unwrap(),
            &JsonValue::from("Limitless")
        );
        assert_eq!(
            jjk.select("skills.2").unwrap(),
            &JsonValue::from("Domain Expansion: Unlimited Void")
        );

        assert_eq!(jjk.select("skills.6"), None);

        // Select array object
        assert_eq!(
            jjk.select("students.0.name").unwrap(),
            &JsonValue::from("Yuji Itadori")
        );
        assert_eq!(jjk.select("students.0.age").unwrap(), &JsonValue::from(15));

        assert_eq!(
            jjk.select("students.1.name").unwrap(),
            &JsonValue::from("Megumi Fushiguro")
        );
        assert_eq!(jjk.select("students.1.age").unwrap(), &JsonValue::from(16));
    }
}
