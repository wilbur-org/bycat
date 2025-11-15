use async_walkdir::WalkDir;
use bycat_error::BoxError;
use bycat_futures::IntoResult;
use bycat_package::{IntoPackage, Package, WithPath};
use bycat_source::Source;
use futures::{FutureExt, Stream, future::BoxFuture};
use std::{
    boxed::Box,
    format,
    path::{Path, PathBuf},
    task::{Poll, ready},
    vec::Vec,
};

use bycat::Matcher;
use pin_project_lite::pin_project;
use relative_path::{RelativePath, RelativePathBuf};

use crate::fs::Body;

pub struct ResolvedPath {
    pub(crate) root: PathBuf,
    pub(crate) path: RelativePathBuf,
}

impl ResolvedPath {
    pub fn new(root: PathBuf, path: RelativePathBuf) -> ResolvedPath {
        ResolvedPath { root, path }
    }

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

impl AsRef<RelativePath> for ResolvedPath {
    fn as_ref(&self) -> &RelativePath {
        &self.path
    }
}

impl WithPath for ResolvedPath {
    fn path(&self) -> &relative_path::RelativePath {
        &self.path
    }
}

impl IntoPackage<Body> for ResolvedPath {
    type Future = BoxFuture<'static, Result<Package<Body>, Self::Error>>;

    type Error = bycat_error::Error;

    fn into_package(self) -> Self::Future {
        Box::pin(async move {
            let full_path = self.path.to_logical_path(&self.root);

            let meta = tokio::fs::metadata(&full_path)
                .await
                .map_err(bycat_error::Error::new)?;
            if meta.is_dir() {
                return Err(bycat_error::Error::new(std::io::Error::new(
                    std::io::ErrorKind::IsADirectory,
                    format!("Package cannot be a directory: {:?}", full_path),
                )));
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

    pub fn walkdir<'a>(&'a self) -> ResolverWalkStream<'a> {
        ResolverStream {
            stream: WalkDir::new(&self.root),
            resolver: self,
        }
    }

    pub fn into_walkdir(self) -> IntoResolverWalkStream {
        IntoResolverStream {
            stream: WalkDir::new(&self.root),
            resolver: self,
        }
    }

    pub fn list_dir<'a>(&'a self) -> ResolverListStream<'a> {
        ResolverStream {
            stream: ReadDirStream {
                state: ReadDirState::ReadDir {
                    future: tokio::fs::read_dir(self.root.clone()).boxed(),
                },
                root: self.root.clone(),
            },
            resolver: self,
        }
    }

    pub fn into_list_dir(self) -> IntoResolverListStream {
        IntoResolverStream {
            stream: ReadDirStream {
                state: ReadDirState::ReadDir {
                    future: tokio::fs::read_dir(self.root.clone()).boxed(),
                },
                root: self.root.clone(),
            },
            resolver: self,
        }
    }
}

pub type ResolverWalkStream<'a> = ResolverStream<'a, WalkDir>;

pub type IntoResolverWalkStream = IntoResolverStream<WalkDir>;

pub type ResolverListStream<'a> = ResolverStream<'a, ReadDirStream>;

pub type IntoResolverListStream = IntoResolverStream<ReadDirStream>;

impl<C> Source<C> for FileResolver {
    type Item = ResolvedPath;

    type Error = bycat_error::Error;

    type Stream<'a>
        = IntoResolverStream<WalkDir>
    where
        Self: 'a,
        C: 'a;

    fn create_stream<'a>(self, _ctx: &'a C) -> Self::Stream<'a> {
        self.into_walkdir()
    }
}

pin_project! {
    pub struct ResolverStream<'a, S> {
        #[pin]
        stream: S,
        resolver: &'a FileResolver,
    }
}

impl<'a, S> Stream for ResolverStream<'a, S>
where
    S: Stream,
    S::Item: IntoResult,
    <S::Item as IntoResult>::Output: DirEntryTrait,
    <S::Item as IntoResult>::Error: Into<BoxError>,
{
    type Item = Result<ResolvedPath, bycat_error::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            let this = self.as_mut().project();

            let next = match ready!(this.stream.poll_next(cx)).map(|m| m.into_result()) {
                Some(Ok(ret)) => ret,
                Some(Err(err)) => return Poll::Ready(Some(Err(bycat_error::Error::new(err)))),
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
    pub struct IntoResolverStream<S> {
        #[pin]
        stream: S,
        resolver: FileResolver,
    }
}

impl<S> Stream for IntoResolverStream<S>
where
    S: Stream,
    S::Item: IntoResult,
    <S::Item as IntoResult>::Output: DirEntryTrait,
    <S::Item as IntoResult>::Error: Into<BoxError>,
{
    type Item = Result<ResolvedPath, bycat_error::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            let this = self.as_mut().project();

            let next = match ready!(this.stream.poll_next(cx)).map(|m| m.into_result()) {
                Some(Ok(ret)) => ret,
                Some(Err(err)) => return Poll::Ready(Some(Err(bycat_error::Error::new(err)))),
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

trait DirEntryTrait {
    fn path(&self) -> PathBuf;
}

impl DirEntryTrait for tokio::fs::DirEntry {
    fn path(&self) -> PathBuf {
        self.path()
    }
}

impl DirEntryTrait for async_walkdir::DirEntry {
    fn path(&self) -> PathBuf {
        self.path()
    }
}

pin_project! {
    #[project= ReadDirProj]
    enum ReadDirState {
        ReadDir {
            #[pin]
            future: BoxFuture<'static, Result<tokio::fs::ReadDir, std::io::Error>>,
        },
        Stream {
            #[pin]
            stream: tokio_stream::wrappers::ReadDirStream,
        },
        Done
    }

}

pin_project! {
    pub struct ReadDirStream {
        #[pin]
        state: ReadDirState,
        root: PathBuf
    }

}

impl Stream for ReadDirStream {
    type Item = Result<tokio::fs::DirEntry, tokio::io::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                ReadDirProj::ReadDir { future } => match ready!(future.poll(cx)) {
                    Ok(ret) => {
                        let stream = tokio_stream::wrappers::ReadDirStream::new(ret);
                        this.state.set(ReadDirState::Stream { stream });
                    }
                    Err(err) => {
                        this.state.set(ReadDirState::Done);
                        return Poll::Ready(Some(Err(err)));
                    }
                },
                ReadDirProj::Stream { stream } => {
                    return stream.poll_next(cx);
                }
                ReadDirProj::Done => return Poll::Ready(None),
            }
        }
    }
}
