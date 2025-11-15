use std::{
    boxed::Box,
    path::PathBuf,
    task::{Poll, ready},
    vec::Vec,
};

use super::{
    Body,
    resolver::{FileResolver, IntoResolverStream, ResolvedPath},
};
use bycat::Matcher;
use bycat_error::Error;
use bycat_package::{IntoPackage, Package};
use bycat_source::Source;
use futures::Stream;
use pin_project_lite::pin_project;

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
            stream: Wrap {
                stream: self.root.into_walkdir(),
                state: WrapState::Stream,
            },
        }
    }
}

pin_project! {

pub struct WalkDirStream {
    #[pin]
    stream: Wrap<IntoResolverStream<async_walkdir::WalkDir>>,
}
}

impl Stream for WalkDirStream {
    type Item = Result<Package<Body>, Error>;
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.project().stream.poll_next(cx)
    }
}

pub struct ReadDir {
    patterns: Vec<Box<dyn Matcher<ResolvedPath> + Send + Sync>>,
    root: FileResolver,
}

impl ReadDir {
    pub fn new(path: impl Into<PathBuf>) -> ReadDir {
        ReadDir {
            patterns: Default::default(),
            root: FileResolver::new(path.into()),
        }
    }
    pub fn pattern<T: Matcher<ResolvedPath> + Send + Sync + 'static>(mut self, pattern: T) -> Self {
        self.patterns.push(Box::new(pattern));
        self
    }
}

impl<C> Source<C> for ReadDir {
    type Item = Package<Body>;

    type Error = Error;

    type Stream<'a>
        = ReadDirPackageStream
    where
        Self: 'a,
        C: 'a;

    fn create_stream<'a>(self, _ctx: &'a C) -> Self::Stream<'a> {
        ReadDirPackageStream {
            stream: Wrap {
                stream: self.root.into_list_dir(),
                state: WrapState::Stream,
            },
        }
    }
}

pin_project! {

pub struct ReadDirPackageStream {
    #[pin]
    stream: Wrap<IntoResolverStream<super::resolver::ReadDirStream>>,
}
}

impl Stream for ReadDirPackageStream {
    type Item = Result<Package<Body>, Error>;
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.project().stream.poll_next(cx)
    }
}

pin_project! {
    #[project = WrapStateProj]
    enum WrapState<T> {
        Stream,
        Project {
            #[pin]
            future: T
        },
    }
}

pin_project! {
    struct Wrap<T> {
        #[pin]
        stream: T,
        #[pin]
        state: WrapState<<ResolvedPath as IntoPackage<Body>>::Future>,
    }
}

impl<T> Stream for Wrap<T>
where
    T: Stream<Item = Result<ResolvedPath, bycat_error::Error>>,
{
    type Item = Result<Package<Body>, Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                WrapStateProj::Stream => match ready!(this.stream.poll_next(cx)) {
                    Some(Ok(ret)) => {
                        this.state.set(WrapState::Project {
                            future: ret.into_package(),
                        });
                    }
                    Some(Err(err)) => return Poll::Ready(Some(Err(err))),
                    None => return Poll::Ready(None),
                },
                WrapStateProj::Project { future } => {
                    let ret = ready!(future.poll(cx));
                    this.state.set(WrapState::Stream);
                    return Poll::Ready(Some(ret));
                }
            }
        }
    }
}
