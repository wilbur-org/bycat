use std::{
    path::PathBuf,
    task::{Poll, ready},
};

use crate::{
    Body,
    resolver::{FileResolver, IntoResolverStream, ResolvedPath},
};
use bycat::Matcher;
use bycat_error::Error;
use bycat_package::Package;
use bycat_source::Source;
use futures::{FutureExt, Stream, future::BoxFuture};
use pin_project_lite::pin_project;
use relative_path::RelativePathBuf;

pub struct WalkDir {
    root: FileResolver,
}

impl Default for WalkDir {
    fn default() -> Self {
        WalkDir::new(std::env::current_dir().unwrap())
    }
}

impl WalkDir {
    pub fn new(root: PathBuf) -> WalkDir {
        WalkDir {
            root: FileResolver::new(root),
        }
    }

    pub fn pattern<T: Matcher<ResolvedPath> + Send + Sync + 'static>(self, pattern: T) -> Self {
        Self {
            root: self.root.pattern(pattern),
        }
    }
}

impl<C> Source<C> for WalkDir {
    type Item = Package<Body>;

    type Error = Error;

    type Stream<'a>
        = WalkDirStream
    where
        Self: 'a,
        C: 'a;

    fn create_stream<'a>(self, _ctx: &'a C) -> Self::Stream<'a> {
        WalkDirStream {
            stream: self.root.into_find(),
        }
    }
}

pin_project! {

pub struct WalkDirStream {
    #[pin]
    stream: IntoResolverStream,
}
}

impl Stream for WalkDirStream {
    type Item = Result<Package<Body>, Error>;
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let next = match ready!(self.project().stream.poll_next(cx)) {
            Some(Ok(ret)) => ret,
            Some(Err(err)) => return Poll::Ready(Some(Err(Error::new(err)))),
            None => return Poll::Ready(None),
        };

        let full_path = next.full_path();
        let mime = mime_guess::from_path(&full_path).first_or_octet_stream();

        Poll::Ready(Some(Ok(Package::new(
            next.path().clone(),
            mime,
            Body::Path(full_path),
        ))))
    }
}

pub struct ReadDir {
    patterns: Vec<Box<dyn Matcher<ResolvedPath> + Send + Sync>>,
    root: PathBuf,
}

impl ReadDir {
    pub fn new(path: impl Into<PathBuf>) -> ReadDir {
        ReadDir {
            patterns: Default::default(),
            root: path.into(),
        }
    }
    pub fn pattern<T: Matcher<ResolvedPath> + Send + Sync + 'static>(mut self, pattern: T) -> Self {
        self.patterns.push(Box::new(pattern));
        self
    }
}

impl<C> Source<C> for ReadDir {
    type Item = ResolvedPath;

    type Error = Error;

    type Stream<'a>
        = ReadDirStream
    where
        Self: 'a,
        C: 'a;

    fn create_stream<'a>(self, _ctx: &'a C) -> Self::Stream<'a> {
        ReadDirStream {
            patterns: self.patterns,
            state: ReadDirState::ReadDir {
                future: tokio::fs::read_dir(self.root.clone()).boxed(),
            },
            root: self.root.clone(),
        }
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
        patterns: Vec<Box<dyn Matcher<ResolvedPath> + Send + Sync>>,
        #[pin]
        state: ReadDirState,
        root: PathBuf
    }

}

impl Stream for ReadDirStream {
    type Item = Result<ResolvedPath, Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                ReadDirProj::ReadDir { future } => {
                    //
                    match ready!(future.poll(cx)) {
                        Ok(ret) => {
                            let stream = tokio_stream::wrappers::ReadDirStream::new(ret);
                            this.state.set(ReadDirState::Stream { stream });
                        }
                        Err(err) => {
                            this.state.set(ReadDirState::Done);
                            return Poll::Ready(Some(Err(Error::new(err))));
                        }
                    }
                }
                ReadDirProj::Stream { stream } => {
                    //
                    match ready!(stream.poll_next(cx)) {
                        Some(Ok(next)) => {
                            let path = match pathdiff::diff_paths(next.path(), &this.root) {
                                Some(path) => path,
                                None => continue,
                            };

                            let Ok(path) = RelativePathBuf::from_path(path) else {
                                continue;
                            };

                            let path = ResolvedPath {
                                root: this.root.clone(),
                                path: path,
                            };

                            return Poll::Ready(Some(Ok(path)));
                        }
                        Some(Err(err)) => return Poll::Ready(Some(Err(Error::new(err)))),
                        None => return Poll::Ready(None),
                    }
                }
                ReadDirProj::Done => return Poll::Ready(None),
            }
        }
    }
}
