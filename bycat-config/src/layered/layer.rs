use std::collections::HashMap;

use bycat_error::Error;
use bycat_value::{Map, String, Value};
use futures::future::BoxFuture;

use crate::layered::source::ConfigSource;

pub struct Layer {
    pub name: String,
    pub config: Map,
    pub source: Box<dyn ConfigSource>,
}

impl Layer {
    pub async fn load(&mut self) -> Result<(), Error> {
        self.config = self.source.load_config().await?;
        Ok(())
    }

    pub async fn save(&self) -> Result<(), Error> {
        self.source.save_config(&self.config).await
    }
}

impl Layer {
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
    pub fn merge_into(self, map: &mut Map) {
        map.extend(self.config);
    }
}
