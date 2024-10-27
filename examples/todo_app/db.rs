use http1_web::serde::{de::Deserialize, json::value::JsonValue, ser::Serialize};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct SetValueError;

impl Display for SetValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to set value")
    }
}

#[derive(Debug, Clone)]
pub struct KeyValueDatabase(PathBuf);

impl KeyValueDatabase {
    pub fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let cwd = std::env::current_dir()?;
        let file_path = cwd.join(path);

        if !file_path.exists() {
            let mut ancestors = file_path.ancestors();
            ancestors.next();

            if let Some(dir) = ancestors.next() {
                std::fs::create_dir_all(dir)?;
            }

            std::fs::write(&file_path, "{}")?;
            log::debug!("Created kv database file: {file_path:?}");
        } else {
            log::debug!("kv database file exists: {file_path:?}");
        }

        Ok(KeyValueDatabase(file_path))
    }

    fn tap<F, R>(&self, f: F) -> std::io::Result<R>
    where
        F: FnOnce(&mut JsonValue) -> std::io::Result<R>,
    {
        let bytes = std::fs::read(self.0.as_path())?;
        let mut json = if bytes.is_empty() {
            JsonValue::Object(Default::default())
        } else {
            http1_web::serde::json::from_bytes::<JsonValue>(bytes)
                .map_err(|err| std::io::Error::other(err))?
        };

        let result = f(&mut json);
        std::fs::write(self.0.as_path(), json.to_string())?;
        result
    }

    pub fn set<T: Serialize>(&self, key: impl AsRef<str>, value: T) -> std::io::Result<()> {
        self.tap(|json| {
            let new_value = http1_web::serde::json::to_value(&value)
                .map_err(|err| std::io::Error::other(err))?;
            json.try_insert(key.as_ref(), new_value)
                .map_err(|err| std::io::Error::other(err))?;
            Ok(())
        })
    }

    pub fn get<T: Deserialize + 'static>(
        &self,
        key: impl AsRef<str>,
    ) -> std::io::Result<Option<T>> {
        self.tap(|json| {
            let value = match json.get(key.as_ref()) {
                Some(x) => x,
                None => return Ok(None),
            };

            let value = http1_web::serde::json::from_value::<T>(value.clone())
                .map_err(|err| std::io::Error::other(err))?;
            Ok(Some(value))
        })
    }

    pub fn scan<T: Deserialize>(&self, pattern: impl AsRef<str>) -> std::io::Result<Vec<T>> {
        self.tap(|json| {
            let pattern = pattern.as_ref();
            let mut values = Vec::new();

            match json {
                JsonValue::Object(ordered_map) => {
                    for (k, v) in ordered_map.iter() {
                        if !k.starts_with(pattern) {
                            continue;
                        }

                        match http1_web::serde::json::from_value::<T>(v.clone()) {
                            Ok(x) => values.push(x),
                            Err(err) => {
                                log::warn!(
                                    "failed to scan value as `{}`: {err}",
                                    std::any::type_name::<T>()
                                );
                            }
                        };
                    }
                }
                v => panic!("expected json object but was `{}`", v.variant()),
            }

            Ok(values)
        })
    }

    pub fn incr(&self, key: impl AsRef<str>) -> std::io::Result<u64> {
        self.tap(|json| {
            let key = key.as_ref();
            match json.get(key) {
                Some(x) => {
                    if !x.is_number() {
                        return Err(std::io::Error::other(format!("`{key}` is not a number")));
                    }

                    let value = x.as_number().unwrap().as_u64().unwrap_or(0) + 1;
                    let new_value = JsonValue::from(value);
                    json.try_insert(key, new_value)
                        .map_err(|err| std::io::Error::other(err))?;
                    Ok(value)
                }
                None => {
                    json.try_insert(key, JsonValue::from(0))
                        .map_err(|err| std::io::Error::other(err))?;
                    Ok(0)
                }
            }
        })
    }

    pub fn contains(&self, key: impl AsRef<str>) -> std::io::Result<bool> {
        self.tap(|json| match json.get(key.as_ref()) {
            Some(_) => Ok(true),
            None => Ok(false),
        })
    }

    pub fn del(&self, key: impl AsRef<str>) -> std::io::Result<bool> {
        self.tap(|json| {
            let deleted = json.remove(key.as_ref()).is_some();
            Ok(deleted)
        })
    }

    pub fn retain(&self, f: impl Fn(&str) -> bool) -> std::io::Result<usize> {
        self.tap(|json| {
            let mut deleted_count = 0;

            match json {
                JsonValue::Object(ordered_map) => {
                    ordered_map.retain(|k, _| {
                        let should_remove = !f(k);

                        if should_remove {
                            deleted_count += 1;
                        }

                        should_remove
                    });
                }
                v => panic!("expected json object but was `{}`", v.variant()),
            }

            Ok(deleted_count)
        })
    }
}
