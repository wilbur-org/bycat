use std::sync::Arc;

use bycat_config::Config;

use bycat_service::Shutdown;

use crate::{Builder, paths::Paths};

pub(crate) struct AppInner {
    pub paths: Paths,
    pub settings: Config,
    pub args: Vec<String>,
    pub shutdown: Shutdown,
}

#[derive(Clone)]
pub struct App(pub(crate) Arc<AppInner>);

impl App {
    pub fn new<'a>(app: &'a str) -> Builder<'a> {
        Builder::new(app)
    }
}

impl App {
    pub fn paths(&self) -> &Paths {
        &self.0.paths
    }

    pub fn args(&self) -> &[String] {
        &self.0.args
    }

    pub fn settings(&self) -> &Config {
        &self.0.settings
    }
}
