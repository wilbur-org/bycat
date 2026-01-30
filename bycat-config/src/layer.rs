use std::path::{Path, PathBuf};

use bycat_value::{Map, Value, merge};

use crate::layer;

pub struct Layer {
    source: PathBuf, // TODO ConfigSource
    config: Map,
}

impl Layer {
    pub fn path(&self) -> &Path {
        &self.source
    }

    pub fn config(&self) -> &Map {
        &self.config
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.config.get(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.config.contains_key(key)
    }
}

impl Layer {
    fn merge_into(self, map: &mut Map) {
        map.extend(self.config);
    }
}

pub struct Config {
    layer: Vec<Layer>,
    overrides: Map,
}

impl Config {
    pub fn get(&self, key: &str) -> Option<&Value> {
        if let Some(value) = self.overrides.get(key) {
            Some(value)
        } else {
            for item in self.layer.iter().rev() {
                if let Some(found) = item.get(key) {
                    return Some(found);
                }
            }
            None
        }
    }

    pub fn try_get<'a, S: serde::Deserialize<'a>>(
        &self,
        name: &str,
    ) -> Result<S, bycat_value::serde::DeserializerError> {
        if let Some(v) = self.get(name).cloned() {
            bycat_value::from_value(v)
        } else {
            Err(bycat_value::serde::DeserializerError::Custom(format!(
                "field not found: {}",
                name
            )))
        }
    }

    pub fn try_set<S: serde::Serialize>(
        &mut self,
        name: &str,
        value: S,
    ) -> Result<Option<Value>, bycat_value::serde::SerializerError> {
        Ok(self
            .overrides
            .insert(name, bycat_value::serde::to_value(value)?))
    }

    pub fn set(&mut self, name: impl ToString, value: impl Into<Value>) -> Option<Value> {
        self.overrides.insert(name.to_string(), value.into())
    }

    pub fn contains(&self, name: impl AsRef<str>) -> bool {
        if self.overrides.contains_key(name.as_ref()) {
            return true;
        }

        for item in self.layer.iter().rev() {
            if item.contains(name.as_ref()) {
                return true;
            }
        }

        false
    }

    pub fn extend(&mut self, config: Config) {
        let map = config.into_merged();
        for (key, value) in map.into_iter() {
            if !self.overrides.contains_key(&key) {
                self.overrides.insert(key, value);
            } else {
                let prev = self.overrides.get_mut(&key).unwrap();
                merge(prev, value);
            }
        }
    }

    pub fn try_into<'de, T: serde::Deserialize<'de>>(
        self,
    ) -> Result<T, bycat_value::serde::DeserializerError> {
        bycat_value::from_value(Value::Map(self.into_merged()))
    }

    pub fn layers(&self) -> &[Layer] {
        &self.layer
    }

    pub fn overrides(&self) -> &Map {
        &self.overrides
    }

    fn into_merged(self) -> Map {
        let mut map = Map::default();
        for layer in self.layer {
            layer.merge_into(&mut map);
        }
        map.extend(self.overrides);
        map
    }
}
