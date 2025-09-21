use std::{borrow::Cow, marker::PhantomData, path::Path};

use bycat_config::{Config, ConfigFactory, Locator, Mode};
use bycat_error::Error;
use directories::{BaseDirs, ProjectDirs};
use tracing::debug;

use crate::{Paths, cli::Cli};

struct LocatorBuilder {
    pub pattern: Option<String>,
    pub mode: Mode,
}

impl LocatorBuilder {
    pub fn create_locator(&self, path: &Path) -> Locator {
        let mut locator = Locator::new(path).mode(self.mode);
        if let Some(pattern) = &self.pattern {
            locator = locator.pattern(pattern);
        }

        locator
    }
}

impl<'a> Default for LocatorBuilder {
    fn default() -> Self {
        LocatorBuilder {
            pattern: None,
            mode: Mode::Single,
        }
    }
}

pub struct ConfigBuilder {
    global: Option<LocatorBuilder>,
    local: Option<LocatorBuilder>,
    config: Config,
}

impl ConfigBuilder {
    pub fn set_local(&mut self, mode: Mode, pattern: impl Into<Option<String>>) {
        self.local = Some(LocatorBuilder {
            pattern: pattern.into(),
            mode,
        })
    }

    pub fn set_global(&mut self, mode: Mode, pattern: impl Into<Option<String>>) {
        self.global = Some(LocatorBuilder {
            pattern: pattern.into(),
            mode,
        })
    }
}

impl ConfigBuilder {
    pub(crate) fn create_factory(&self, paths: &Paths, cwd: &Path) -> ConfigFactory {
        let mut factory = ConfigFactory::default();

        if let Some(global) = &self.global {
            debug!(path = ?paths.config().path, "Add config lookup path");
            factory.add_locator(global.create_locator(paths.config().path));
        }

        if let Some(local) = &self.local {
            debug!(path = ?cwd, "Add config lookup path");
            factory.add_locator(local.create_locator(cwd));
        }

        factory
    }
}

pub struct Builder<'a> {
    name: &'a str,
    config: ConfigBuilder,
}

impl<'a> Builder<'a> {
    pub fn new(name: &'a str) -> Builder<'a> {
        Builder {
            name,
            config: ConfigBuilder {
                local: Default::default(),
                global: Some(LocatorBuilder {
                    pattern: Some(format!("{}.{}", name, "{ext}").into()),
                    mode: Mode::Single,
                }),
                config: Config::default(),
            },
        }
    }

    pub fn config<T: FnOnce(&mut ConfigBuilder)>(mut self, func: T) -> Self {
        (func)(&mut self.config);
        self
    }

    pub fn build<C, T>(self, work: T) -> Result<Cli<C, T>, Error> {
        let base = BaseDirs::new().ok_or_else(|| Error::new("Could not acquire home directory"))?;
        let project = ProjectDirs::from("", "", &self.name)
            .ok_or_else(|| Error::new("Could not acquire home directory"))?;

        Ok(Cli {
            work,
            paths: (base, project).into(),
            config: self.config,
            ctx: PhantomData,
        })
    }
}
