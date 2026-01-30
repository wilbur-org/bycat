use std::collections::HashMap;

use bycat_error::Error;
use bycat_value::{Map, String};
use futures::future::BoxFuture;

use crate::layered::layer::Layer;

use super::{config::Config, source::ConfigSource};

#[derive(Default)]
pub struct ConfigBuilder {
    sources: Vec<(String, Box<dyn ConfigSource>)>,
}

impl ConfigBuilder {
    pub fn with_source<S: ConfigSource + 'static>(mut self, name: &str, source: S) -> Self {
        self.sources.push((name.into(), Box::new(source)));
        self
    }

    pub fn add_source<S: ConfigSource + 'static>(&mut self, name: &str, source: S) -> &mut Self {
        self.sources.push((name.into(), Box::new(source)));
        self
    }

    pub async fn build(self) -> Result<Config, Error> {
        let mut layers = Vec::with_capacity(self.sources.len());

        for (k, v) in self.sources {
            let config = v.load_config().await?;
            layers.push(Layer {
                name: k,
                config,
                source: v,
            });
        }

        Ok(Config {
            layers,
            overrides: Default::default(),
        })
    }
}
