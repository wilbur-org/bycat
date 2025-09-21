use std::path::Path;

use bycat_fs::ResolvedPath;
use bycat_package::match_glob;
use futures::StreamExt;

use crate::{Paths, paths::Dir};

pub struct ConfigBuilder<'a> {
    pub local: Option<&'a str>,
    pub pattern: Option<&'a str>,
}

impl<'a> ConfigBuilder<'a> {
    pub fn build(self) -> SettingsFactory {
        SettingsFactory {
            local: self.local.map(|m| m.to_string()),
            pattern: self.pattern.map(|m| m.to_string()),
        }
    }
}

pub struct SettingsFactory {
    local: Option<String>,
    pattern: Option<String>,
}

impl SettingsFactory {
    pub async fn load(&self, paths: &Paths, cwd: Dir<'_>) -> Settings {
        let mut stream = Vec::new();
        if let Some(local) = &self.pattern {
            stream.push(cwd.find(match_glob(local.clone())));
        }

        if let Some(local) = &self.local {
            stream.push(cwd.find(match_glob(local.clone())));
        }

        let mut files = futures::stream::select_all(stream);

        while let Some(next) = files.next().await {}

        Settings {}
    }
}

pub struct LoadFuture {}

#[derive(Default)]
pub struct Settings {}
