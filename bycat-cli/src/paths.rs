use std::path::{Path, PathBuf};

use bycat::{Matcher, Work};
use bycat_error::Error;
use bycat_fs::{Body, FsDest, ReadDir, ReadDirStream, ResolvedPath, WalkDir, WalkDirStream};
use bycat_package::{IntoPackage, Package};
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
    pub fn config(&self) -> Dir<'_> {
        Dir { path: &self.config }
    }

    pub fn cache(&self) -> Dir<'_> {
        Dir { path: &self.cache }
    }

    pub fn data(&self) -> Dir<'_> {
        Dir { path: &self.data }
    }

    pub fn home(&self) -> Dir<'_> {
        Dir { path: &self.home }
    }
}

#[derive(Debug)]
pub struct Dir<'a> {
    pub(crate) path: &'a Path,
}

impl<'a> Dir<'a> {
    pub fn find<T: Matcher<ResolvedPath> + Send + Sync + 'static>(
        &self,
        matcher: T,
    ) -> WalkDirStream {
        let resolver = WalkDir::new(self.path.to_path_buf()).pattern(matcher);
        resolver.create_stream(&())
    }

    pub fn list(&self, path: impl AsRef<RelativePath>) -> ReadDirStream {
        let resolver = ReadDir::new(path.as_ref().to_logical_path(self.path));
        resolver.create_stream(&())
    }

    pub async fn read(&self, path: impl AsRef<RelativePath>) -> Result<Package<Body>, Error> {
        let path = ResolvedPath::new(
            self.path.to_path_buf(),
            path.as_ref().to_relative_path_buf(),
        );
        path.into_package().await
    }

    pub async fn write(&self, file: Package<Body>) -> Result<(), Error> {
        FsDest::new(self.path).call(&(), file).await?;
        Ok(())
    }
}
