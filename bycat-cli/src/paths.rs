use std::path::{Path, PathBuf};

use bycat::Matcher;
use bycat_error::Error;
use bycat_fs::{Body, Fs, ReadDirStream, ResolvedPath, WalkDirStream};
use bycat_package::Package;
use bycat_source::Source;
use directories::{BaseDirs, ProjectDirs};
use relative_path::RelativePath;

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
    pub fn config(&self) -> Dir {
        Dir {
            fs: Fs::new(&self.config),
        }
    }

    pub fn cache(&self) -> Dir {
        Dir {
            fs: Fs::new(&self.cache),
        }
    }

    pub fn data(&self) -> Dir {
        Dir {
            fs: Fs::new(&self.data),
        }
    }

    pub fn home(&self) -> Dir {
        Dir {
            fs: Fs::new(&self.home),
        }
    }
}

#[derive(Debug)]
pub struct Dir {
    pub(crate) fs: Fs,
}

impl Dir {
    pub fn path(&self) -> &Path {
        self.fs.path()
    }

    pub fn find<T: Matcher<ResolvedPath> + Send + Sync + 'static>(
        &self,
        matcher: T,
    ) -> WalkDirStream {
        self.fs.find(matcher).create_stream(&())
    }

    pub fn list(&self, path: impl AsRef<RelativePath>) -> ReadDirStream {
        self.fs.list(path).create_stream(&())
    }

    pub async fn read(&self, path: impl AsRef<RelativePath>) -> Result<Package<Body>, Error> {
        self.fs.read(path).await
    }

    pub async fn write(&self, file: Package<Body>) -> Result<(), Error> {
        self.fs.write(file).await
    }
}
