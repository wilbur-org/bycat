use std::{
    path::{Path, PathBuf},
    task::{Poll, ready},
};

use async_walkdir::WalkDir;
use bycat_package::{IntoPackage, Package, WithPath};
use bycat_source::Source;
use futures::{Stream, future::BoxFuture};

pub use async_walkdir::Error as WalkDirError;
use bycat::Matcher;
use pin_project_lite::pin_project;
use relative_path::RelativePathBuf;

use crate::Body;

pub struct ResolvedPath {
    root: PathBuf,
    path: RelativePathBuf,
}

impl ResolvedPath {
    pub fn path(&self) -> &RelativePathBuf {
        &self.path
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn full_path(&self) -> PathBuf {
        self.path.to_logical_path(&self.root)
    }
}

impl WithPath for ResolvedPath {
    fn path(&self) -> &relative_path::RelativePath {
        &self.path
    }
}

impl IntoPackage<Body> for ResolvedPath {
    type Future = BoxFuture<'static, Result<Package<Body>, Self::Error>>;

    type Error = std::io::Error;

    fn into_package(self) -> Self::Future {
        Box::pin(async move {
            let full_path = self.path.to_logical_path(&self.root);

            let meta = tokio::fs::metadata(&full_path).await?;
            if meta.is_dir() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::IsADirectory,
                    "Package cannot be a directory",
                ));
            }

            let mime = mime_guess::from_path(&full_path).first_or_octet_stream();

            Ok(Package::new(self.path, mime, Body::Path(full_path)))
        })
    }
}

pub struct FileResolver {
    patterns: Vec<Box<dyn Matcher<ResolvedPath> + Send + Sync>>,
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
    pub fn pattern<M: Matcher<ResolvedPath> + Send + Sync + 'static>(mut self, pattern: M) -> Self {
        self.patterns.push(Box::new(pattern));
        self
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn find<'a>(&'a self) -> ResolverStream<'a> {
        ResolverStream {
            stream: WalkDir::new(&self.root),
            resolver: self,
        }
    }

    pub fn into_find(self) -> IntoResolverStream {
        IntoResolverStream {
            stream: WalkDir::new(&self.root),
            resolver: self,
        }
    }
}

impl<C> Source<C> for FileResolver {
    type Item = ResolvedPath;

    type Error = WalkDirError;

    type Stream<'a>
        = IntoResolverStream
    where
        Self: 'a,
        C: 'a;

    fn create_stream<'a>(self, _ctx: &'a C) -> Self::Stream<'a> {
        self.into_find()
    }
}

pin_project! {
    pub struct ResolverStream<'a> {
        #[pin]
        stream: WalkDir,
        resolver: &'a FileResolver,
    }
}

impl<'a> Stream for ResolverStream<'a> {
    type Item = Result<ResolvedPath, WalkDirError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            let this = self.as_mut().project();

            let next = match ready!(this.stream.poll_next(cx)) {
                Some(Ok(ret)) => ret,
                Some(Err(err)) => return Poll::Ready(Some(Err(err))),
                None => return Poll::Ready(None),
            };

            let path = match pathdiff::diff_paths(next.path(), &this.resolver.root) {
                Some(path) => path,
                None => continue,
            };

            let Ok(path) = RelativePathBuf::from_path(path) else {
                continue;
            };

            let path = ResolvedPath {
                root: this.resolver.root.clone(),
                path: path,
            };

            if this.resolver.patterns.is_empty() {
                return Poll::Ready(Some(Ok(path)));
            } else {
                for pattern in &this.resolver.patterns {
                    if pattern.is_match(&path) {
                        return Poll::Ready(Some(Ok(path)));
                    }
                }
            }
        }
    }
}

pin_project! {
    pub struct IntoResolverStream {
        #[pin]
        stream: WalkDir,
        resolver: FileResolver,
    }
}

impl Stream for IntoResolverStream {
    type Item = Result<ResolvedPath, WalkDirError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            let this = self.as_mut().project();

            let next = match ready!(this.stream.poll_next(cx)) {
                Some(Ok(ret)) => ret,
                Some(Err(err)) => return Poll::Ready(Some(Err(err))),
                None => return Poll::Ready(None),
            };

            let path = match pathdiff::diff_paths(next.path(), &this.resolver.root) {
                Some(path) => path,
                None => continue,
            };

            let Ok(path) = RelativePathBuf::from_path(path) else {
                continue;
            };

            let path = ResolvedPath {
                root: this.resolver.root.clone(),
                path: path,
            };

            if this.resolver.patterns.is_empty() {
                return Poll::Ready(Some(Ok(path)));
            } else {
                for pattern in &this.resolver.patterns {
                    if pattern.is_match(&path) {
                        return Poll::Ready(Some(Ok(path)));
                    }
                }
            }
        }
    }
}
