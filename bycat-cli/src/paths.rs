use bycat_fs::Fs;
use directories::{BaseDirs, ProjectDirs};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Paths {
    config: PathBuf,
    cache: PathBuf,
    data: PathBuf,
    home: PathBuf,
}

impl From<(BaseDirs, ProjectDirs)> for Paths {
    fn from((base, proj): (BaseDirs, ProjectDirs)) -> Self {
        Paths {
            config: proj.config_local_dir().to_path_buf(),
            cache: proj.cache_dir().to_path_buf(),
            data: proj.data_local_dir().to_path_buf(),
            home: base.home_dir().to_path_buf(),
        }
    }
}

impl Paths {
    pub fn config(&self) -> Fs {
        Fs::new(&self.config)
    }

    pub fn cache(&self) -> Fs {
        Fs::new(&self.cache)
    }

    pub fn data(&self) -> Fs {
        Fs::new(&self.data)
    }

    pub fn home(&self) -> Fs {
        Fs::new(&self.home)
    }
}
