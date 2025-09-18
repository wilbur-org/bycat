use std::path::{Path, PathBuf};

use async_walkdir::WalkDir;
use futures::{Stream, TryStreamExt};

pub use async_walkdir::Error as WalkDirError;
use bycat::Matcher;
use relative_path::RelativePathBuf;

pub struct FileResolver {
    patterns: Vec<Box<dyn Matcher<RelativePathBuf> + Send + Sync>>,
    root: PathBuf,
}

impl FileResolver {
    pub fn new(path: PathBuf) -> FileResolver {
        FileResolver {
            patterns: Default::default(),
            root: path,
        }
    }
}

impl FileResolver {
    pub fn pattern<M: Matcher<RelativePathBuf> + Send + Sync + 'static>(
        mut self,
        pattern: M,
    ) -> Self {
        self.patterns.push(Box::new(pattern));
        self
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn find(&self) -> impl Stream<Item = Result<RelativePathBuf, WalkDirError>> {
        async_stream::try_stream! {

            let mut stream = WalkDir::new(&self.root);

            'main_loop:
            while let Some(next) = stream.try_next().await? {
                // Get relative path
                let path = match pathdiff::diff_paths(next.path(), &self.root) {
                    Some(path) => path,
                    None => {
                        continue 'main_loop;
                    }
                };

                let Ok(path) = RelativePathBuf::from_path(path) else {
                    continue
                };

                if self.patterns.is_empty() {
                    yield path;
                } else {
                    for pattern in &self.patterns {
                        if pattern.is_match(&path) {
                            yield path;
                            continue 'main_loop;
                        }
                    }
                }


            }

        }
    }
}
